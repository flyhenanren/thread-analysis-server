use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{FrameError, ThreadError};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Thread {
    pub id: Option<String>,
    pub name: String,
    pub daemon: bool,
    pub prio: Option<u16>,
    pub os_prio: u32,
    pub tid: u64,
    pub nid: u64,
    pub status: ThreadStatus,
    pub address: String,
    pub frames: Vec<CallFrame>,
}
lazy_static::lazy_static! {
    static ref REGEX_MAIN_INFO:Regex = Regex::new(
        r#"^(?P<name>"[^"]*")(?: #(?P<number>\d+))?(?: prio=(?P<prio>\d+))?(?: os_prio=(?P<os_prio>\d+))? tid=(?P<tid>0x[0-9a-fA-F]+) nid=(?P<nid>0x[0-9a-fA-F]+) (?P<state>[^\[]*)(?:\[(?P<hex_address>0x[0-9a-fA-F]+)\])?$"#
    ).unwrap();
    static ref REGEX_STATE:Regex = Regex::new(r"State:\s(\w+)").unwrap();
    static ref REGEX_FRAME:Regex = Regex::new(r"at\s+([\w.$]+)\.(<init>|[\w$]+(?:\$\$Lambda\$\d+/\d+)?)(?:\.(\w+))?\(([^:]+|Unknown Source)(?::(\d+))?\)").unwrap();
}

impl Thread {
    pub fn new(lines: Vec<String>) -> Result<Self, ThreadError> {
        let (name, id, daemon, prio, os_prio, tid, nid, address) =
            Self::parse_thread_info(&lines[0])?;
        let status = match ThreadStatus::from_str(&lines[1]) {
            Ok(status) => status,
            Err(e) => {
                return Err(ThreadError::ParseError(format!(
                    "解析数据行:{}\r\n时错误。\r\nerror:{}",
                    lines[1].to_string(),
                    e.to_string()
                )))
            }
        };
        let call_info = &lines[2..=lines.len() - 1];
        let mut frames: Vec<CallFrame> = Vec::with_capacity(call_info.len());
        for call in call_info {
            frames.push(CallFrame::new(&call)?);
        }

        Ok(Thread {
            id,
            name,
            daemon,
            prio,
            os_prio,
            tid,
            nid,
            status,
            frames,
            address,
        })
    }

    pub fn is_sys_thread(lines: &Vec<String>) -> bool {
        if lines.len() == 1 {
            return Self::is_jvm_thread(&lines[0]);
        }
        false
    }

    fn is_jvm_thread(line: &String) -> bool {
        line.contains("VM Thread")
            || line.contains("VM Periodic Task Thread")
            || line.contains("GC task thread")
    }

    pub fn parse_thread_info(
        line: &str,
    ) -> Result<(String, Option<String>, bool, Option<u16>, u32, u64, u64, String), ThreadError> {
        if let Some(caps) = REGEX_MAIN_INFO.captures(line) {
            let name = caps
                .name("name")
                .ok_or(ThreadError::MissingField)?
                .as_str()
                .trim_matches('\"')
                .to_string();
             let id = match caps.name("number"){
                Some(number)  => Some(format!("#{}", number.as_str())),
                None => None,
            };
            let daemon = caps.name("daemon").is_some();
            let prio = caps
                .name("prio")
                .map(|m| {
                    m.as_str()
                        .parse::<u16>()
                        .map_err(ThreadError::ParseIntError)
                })
                .transpose()?;
            let os_prio = caps
                .name("os_prio")
                .map(|m| {
                    m.as_str()
                        .parse::<u32>()
                        .map_err(ThreadError::ParseIntError)
                })
                .transpose()?;
            let tid_str = caps
                .name("tid")
                .ok_or(ThreadError::MissingField)?
                .as_str()
                .strip_prefix("0x")
                .ok_or(ThreadError::InvalidStatus)?;
            let tid = u64::from_str_radix(tid_str, 16).map_err(ThreadError::ParseIntError)?;
            let nid_str = caps
                .name("nid")
                .ok_or(ThreadError::MissingField)?
                .as_str()
                .strip_prefix("0x")
                .ok_or(ThreadError::InvalidStatus)?;
            let nid = u64::from_str_radix(nid_str, 16).map_err(ThreadError::ParseIntError)?;

            let hex_address = caps
                .name("hex_address")
                .ok_or(ThreadError::MissingField)?
                .as_str()
                .to_string();
            Ok((
                name,
                id,
                daemon,
                prio,
                os_prio.unwrap_or_default(),
                tid,
                nid,
                hex_address,
            ))
        } else {
            Err(ThreadError::ParseError(format!(
                "无法解析堆栈信息:{}",
                line
            )))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ThreadStatus {
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
    New,
}

impl FromStr for ThreadStatus {
    type Err = ThreadError;
    fn from_str(status: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = REGEX_STATE.captures(status) {
            match &captures[1] {
                "NEW" => Ok(ThreadStatus::New),
                "WAITING" => Ok(ThreadStatus::Waiting),
                "BLOCKED" => Ok(ThreadStatus::Blocked),
                "TIMED_WAITING" => Ok(ThreadStatus::TimedWaiting),
                "RUNNABLE" => Ok(ThreadStatus::Runnable),
                "TERMINATED" => Ok(ThreadStatus::Terminated),
                _ => Err(ThreadError::IllegalStatus(captures[1].to_string())),
            }
        } else {
            Err(ThreadError::IllegalStatus(status.to_owned()))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CallFrame {
    pub class_name: String,
    pub method_name: Option<String>,
    pub line_number: Option<u32>,
    pub frame: Frame,
}

impl CallFrame {
    pub fn new(frame_info: &str) -> Result<Self, ThreadError> {
        let trim_info = frame_info.trim();
        if trim_info.starts_with("at") {
            if let Some(caps) = REGEX_FRAME.captures(frame_info) {
                let class_name = caps[1].to_string();
                let method_name = Some(format!("{}.{}", class_name, &caps[2]));
                let line_number = caps
                    .get(5)
                    .map(|m| m.as_str().parse().expect("行号解析失败"));

                let frame = if trim_info.contains("(Native Method)") {
                    Frame::NativeMethod
                } else {
                    Frame::MethodCall
                };

                return Ok(CallFrame {
                    class_name,
                    method_name,
                    line_number,
                    frame,
                });
            } else {
                return Err(ThreadError::ParseError(format!(
                    "无法解析方法：{}",
                    trim_info
                )));
            }
        };

        if trim_info.starts_with('-') {
            let parts: Vec<&str> = trim_info.split_whitespace().collect();
            let (_, after_prefix) = frame_info.split_once("(a ").unwrap_or(("", ""));
            let class_name = after_prefix
                .split(")")
                .next()
                .expect(format!("无法识别的class name:{}", after_prefix).as_str())
                .trim()
                .to_string();
            let frame = match Frame::from_str(parts, frame_info) {
                Ok(frame) => frame,
                Err(e) => return Err(ThreadError::ParseError(e.to_string())),
            };
            return Ok(CallFrame {
                class_name,
                method_name: None,
                line_number: None,
                frame,
            });
        }
        Err(ThreadError::ParseError("分割失败".to_string()))
    }
}
#[derive(Serialize,Deserialize, Debug, PartialEq)]
pub enum Frame {
    MethodCall,
    Lock {
        lock_address: u64,
    },
    Monitor {
        monitor_address: u64,
        action: MonitorAction,
    },
    Parking {
        parking_address: u64,
    },
    NativeMethod,
    Eliminated,
}

impl Frame {
    fn from_str(parts: Vec<&str>, frame_info: &str) -> Result<Self, FrameError> {
        match parts[0] {
            "-" => match parts[1] {
                "locked" => Ok(Frame::Lock {
                    lock_address: extract_address(frame_info),
                }),
                "waiting" => Ok(Frame::Monitor {
                    monitor_address: extract_address(frame_info),
                    action: if frame_info.contains("waiting to lock") {
                        MonitorAction::WaitingToLock
                    } else if frame_info.contains("waiting on") {
                        MonitorAction::WaitingOn
                    } else {
                        MonitorAction::Locked
                    },
                }),
                "parking" => Ok(Frame::Parking {
                    parking_address: extract_address(frame_info),
                }),
                "eliminated" => Ok(Frame::Eliminated),
                _ => Err(FrameError::Unknown(parts[1].to_string())),
            },
            "at" => {
                if frame_info.contains("(Native Method)") {
                    Ok(Frame::NativeMethod)
                } else {
                    Ok(Frame::MethodCall)
                }
            }
            _ => Err(FrameError::Unknown(parts[0].to_string())),
        }
    }
}

fn extract_address(input: &str) -> u64 {
    let parts: Vec<&str> = input.split('<').collect();
    for part in parts.iter().skip(1) {
        let content: Vec<&str> = part.split('>').collect();
        if let Some(content) = content.get(0) {
            let tid_str = content.trim_start_matches("0x");
            return u64::from_str_radix(tid_str, 16).unwrap();
        }
    }
    0x0000000000000000
}
#[derive(Serialize,Deserialize, Debug, PartialEq)]
pub enum MonitorAction {
    WaitingToLock,
    WaitingOn,
    Locked,
}

#[cfg(test)]
pub mod tests {

    use std::vec;

    use super::*;

    #[test]
    pub fn test_runnable_thread() {
        let lines = vec![
          "\"Thread-1\" #1 prio=5 os_prio=0 tid=0x00007f3d70001800 nid=0x2f03 runnable [0x00007f3d80f21000]".to_string(),
          "java.lang.Thread.State: RUNNABLE".to_string(),
          "at com.example.MyClass.myMethod(MyClass.java:10)".to_string(),
          "at com.example.MyClass.run(MyClass.java:5)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string()
    ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(5),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.myMethod".to_string()),
            line_number: Some(10),
            frame: Frame::MethodCall,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#1".to_string()),
            name: "Thread-1".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70001800,
            nid: 0x2f03,
            status: ThreadStatus::Runnable,
            frames,
            address: "0x00007f3d80f21000".to_owned(),
            daemon: false,
        };
        assert_eq!(result.unwrap(), thread)
    }

    #[test]
    pub fn test_blocked_thread() {
        let lines = vec![
          "\"Thread-2\" #2 prio=5 os_prio=0 tid=0x00007f3d70002800 nid=0x2f04 waiting for monitor entry [0x00007f3d80f22000]".to_string(),
          "java.lang.Thread.State: BLOCKED (on object monitor)".to_string(),
          "at com.example.MyClass.synchronizedMethod(MyClass.java:20)".to_string(),
          "- waiting to lock <0x00000000c7c600d0> (a java.lang.Object)".to_string(),
          "at com.example.MyClass.run(MyClass.java:15)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string()     
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(15),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Monitor {
                monitor_address: 0x00000000c7c600d0,
                action: MonitorAction::WaitingToLock,
            },
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.synchronizedMethod".to_string()),
            line_number: Some(20),
            frame: Frame::MethodCall,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#2".to_string()),
            name: "Thread-2".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70002800,
            nid: 0x2f04,
            status: ThreadStatus::Blocked,
            frames,
            daemon: false,
            address: "0x00007f3d80f22000".to_owned(),
        };
        assert_eq!(result.unwrap(), thread)
    }

    #[test]
    pub fn test_waiting_thread() {
        let lines = vec![
          "\"Thread-3\" #3 prio=5 os_prio=0 tid=0x00007f3d70003800 nid=0x2f05 in Object.wait() [0x00007f3d80f23000]".to_string(),
          "java.lang.Thread.State: WAITING (on object monitor)".to_string(),
          "at java.lang.Object.wait(Native Method)".to_string(),
          "- waiting on <0x00000000c7c600d0> (a java.lang.Object)".to_string(),
          "at java.lang.Object.wait(Object.java:502)".to_string(),
          "at com.example.MyClass.waitMethod(MyClass.java:30)".to_string(),
          "- locked <0x00000000c7c600d0> (a java.lang.Object)".to_string(),
          "at com.example.MyClass.run(MyClass.java:25)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string(),
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(25),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Lock {
                lock_address: 0x00000000c7c600d0,
            },
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.waitMethod".to_string()),
            line_number: Some(30),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: Some(502),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Monitor {
                monitor_address: 0x00000000c7c600d0,
                action: MonitorAction::WaitingOn,
            },
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#3".to_string()),
            name: "Thread-3".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70003800,
            nid: 0x2f05,
            status: ThreadStatus::Waiting,
            frames,
            daemon: false,
            address: "0x00007f3d80f23000".to_owned(),
        };
        assert_eq!(result.unwrap(), thread)
    }
    #[test]
    pub fn test_time_waiting_sleep_thread() {
        let lines = vec![
          "\"Thread-4\" #4 prio=5 os_prio=0 tid=0x00007f3d70004800 nid=0x2f06 waiting on condition [0x00007f3d80f24000]".to_string(),
          "java.lang.Thread.State: TIMED_WAITING (sleeping)".to_string(),
          "at java.lang.Thread.sleep(Native Method)".to_string(),
          "at com.example.MyClass.sleepMethod(MyClass.java:40)".to_string(),
          "at com.example.MyClass.run(MyClass.java:35)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string(),
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(35),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.sleepMethod".to_string()),
            line_number: Some(40),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.sleep".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#4".to_string()),
            name: "Thread-4".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70004800,
            nid: 0x2f06,
            status: ThreadStatus::TimedWaiting,
            frames,
            daemon: false,
            address: "0x00007f3d80f24000".to_owned(),
        };
        assert_eq!(result.unwrap(), thread)
    }
    #[test]
    pub fn test_time_wating_monitor_thread() {
        let lines = vec![
          "\"Thread-7\" #7 prio=5 os_prio=0 tid=0x00007f3d70007800 nid=0x2f09 timed waiting on object monitor [0x00007f3d80f26000]".to_string(),
          "java.lang.Thread.State: TIMED_WAITING (on object monitor)".to_string(),
          "at java.lang.Object.wait(Native Method)".to_string(),
          "- waiting on <0x00000000c7c600d0> (a java.lang.Object)".to_string(),
          "at java.lang.Object.wait(Object.java:502)".to_string(),
          "at com.example.MyClass.timedWaitMethod(MyClass.java:60)".to_string(),
          "- locked <0x00000000c7c600d0> (a java.lang.Object)".to_string(),
          "at com.example.MyClass.run(MyClass.java:55)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string()
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(55),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Lock {
                lock_address: 0x00000000c7c600d0,
            },
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.timedWaitMethod".to_string()),
            line_number: Some(60),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: Some(502),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Monitor {
                monitor_address: 0x00000000c7c600d0,
                action: MonitorAction::WaitingOn,
            },
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#7".to_string()),
            name: "Thread-7".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70007800,
            nid: 0x2f09,
            status: ThreadStatus::TimedWaiting,
            frames,
            daemon: false,
            address: "0x00007f3d80f26000".to_owned(),
        };
        assert_eq!(result.unwrap(), thread)
    }

    #[test]
    pub fn test_time_wating_condition_thread() {
        let lines = vec![
          "\"Thread-8\" #8 prio=5 os_prio=0 tid=0x00007f3d70008800 nid=0x2f0a waiting on condition [0x00007f3d80f27000]".to_string(),
          "java.lang.Thread.State: TIMED_WAITING (on a condition)".to_string(),
          "at sun.misc.Unsafe.park(Native Method)".to_string(),
          "- parking to wait for  <0x00000002a5bdfc00> (a java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject)".to_string(),
          "at java.util.concurrent.locks.LockSupport.parkNanos(LockSupport.java:215)".to_string(),
          "at java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos(AbstractQueuedSynchronizer.java:2078)".to_string(),
          "at com.example.MyClass.timedConditionMethod(MyClass.java:70)".to_string(),
          "at com.example.MyClass.run(MyClass.java:65)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string(),
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(65),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.timedConditionMethod".to_string()),
            line_number: Some(70),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject"
                .to_string(),
            method_name: Some(
                "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos"
                    .to_string(),
            ),
            line_number: Some(2078),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.LockSupport".to_string(),
            method_name: Some("java.util.concurrent.locks.LockSupport.parkNanos".to_string()),
            line_number: Some(215),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject"
                .to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Parking {
                parking_address: 0x00000002a5bdfc00,
            },
        });
        frames.push(CallFrame {
            class_name: "sun.misc.Unsafe".to_string(),
            method_name: Some("sun.misc.Unsafe.park".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#8".to_string()),
            name: "Thread-8".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70008800,
            nid: 0x2f0a,
            status: ThreadStatus::TimedWaiting,
            frames,
            daemon: false,
            address: "0x00007f3d80f27000".to_owned(),
        };
        assert_eq!(result.unwrap(), thread)
    }

    #[test]
    pub fn test_waiting_parking_thread() {
        let lines = vec![
          "\"Thread-6\" #6 prio=5 os_prio=0 tid=0x00007f3d70006800 nid=0x2f08 waiting on condition [0x00007f3d80f25000]".to_string(),
          "java.lang.Thread.State: WAITING (parking)".to_string(),
          "at sun.misc.Unsafe.park(Native Method)".to_string(),
          "- parking to wait for  <0x00000002a5bdfc00> (a java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject)".to_string(),
          "at java.util.concurrent.locks.LockSupport.park(LockSupport.java:175)".to_string(),
          "at java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await(AbstractQueuedSynchronizer.java:2039)".to_string(),
          "at com.example.MyClass.conditionMethod(MyClass.java:50)".to_string(),
          "at com.example.MyClass.run(MyClass.java:45)".to_string(),
          "at java.lang.Thread.run(Thread.java:748)".to_string()
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "java.lang.Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(45),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "com.example.MyClass".to_string(),
            method_name: Some("com.example.MyClass.conditionMethod".to_string()),
            line_number: Some(50),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject"
                .to_string(),
            method_name: Some(
                "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await"
                    .to_string(),
            ),
            line_number: Some(2039),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.LockSupport".to_string(),
            method_name: Some("java.util.concurrent.locks.LockSupport.park".to_string()),
            line_number: Some(175),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject"
                .to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Parking {
                parking_address: 0x00000002a5bdfc00,
            },
        });
        frames.push(CallFrame {
            class_name: "sun.misc.Unsafe".to_string(),
            method_name: Some("sun.misc.Unsafe.park".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        frames.reverse();
        let thread = Thread {
            id: Some("#6".to_string()),
            name: "Thread-6".to_string(),
            prio: Some(5),
            os_prio: 0,
            tid: 0x00007f3d70006800,
            nid: 0x2f08,
            status: ThreadStatus::Waiting,
            frames,
            daemon: false,
            address: "0x00007f3d80f25000".to_owned(),
        };
        assert_eq!(result.unwrap(), thread)
    }


    #[test]
    pub fn test_gc_thread() {
        let lines = vec![
          "\"GC task thread#0 (ParallelGC)\" os_prio=0 tid=0x0000ffff8c060800 nid=0xec316 runnable ".to_string(),
    ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        let thread = Thread {
            id: None,
            name: "GC task thread#0 (ParallelGC)".to_string(),
            prio: None,
            os_prio: 0,
            tid: 0x0000ffff8c060800,
            nid: 0xec316,
            status: ThreadStatus::Runnable,
            frames,
            address: "0x0000000000000000".to_owned(),
            daemon: false,
        };
        assert_eq!(result.unwrap(), thread)
    }

    #[test]
    pub fn test_vm_thread() {
        let lines = vec![
          "\"VM Thread\" os_prio=0 tid=0x0000ffff8c132000 nid=0xec341 runnable ".to_string(),
    ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        let thread = Thread {
            id: None,
            name: "VM Thread".to_string(),
            prio: None,
            os_prio: 0,
            tid: 0x0000ffff8c132000,
            nid: 0xec341,
            status: ThreadStatus::Runnable,
            frames,
            address: "0x0000000000000000".to_owned(),
            daemon: false,
        };
        assert_eq!(result.unwrap(), thread)
    }

    #[test]
    pub fn test_task_thread() {
        let lines = vec![
          "\"VM Periodic Task Thread\" os_prio=0 tid=0x0000ffff8c1ab000 nid=0xec358 waiting on condition ".to_string(),
    ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        let thread = Thread {
            id: None,
            name: "VM Periodic Task Thread".to_string(),
            prio: None,
            os_prio: 0,
            tid: 0x0000ffff8c1ab000,
            nid: 0xec358,
            status: ThreadStatus::Waiting,
            frames,
            address: "0x0000000000000000".to_owned(),
            daemon: false,
        };
        assert_eq!(result.unwrap(), thread)
    }
}
