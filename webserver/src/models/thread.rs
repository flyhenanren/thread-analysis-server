use std::str::FromStr;

use regex::Regex;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
pub struct Thread {
    id: String,
    name: String,
    prio: u16,
    os_prio: u32,
    tid: u64,
    nid: u64,
    status: ThreadStatus,
    frames: Vec<CallFrame>,
}

impl Thread {
    pub fn new(lines: Vec<&str>) -> Self {
        let thread_info: Vec<&str> = lines[0].split_whitespace().collect();
        let status = ThreadStatus::from_str(lines[1]).unwrap();
        let call_info = &lines[2..=lines.len() - 1];
        let mut frames: Vec<CallFrame> = Vec::with_capacity(call_info.len());
        for call in call_info {
            frames.insert(0, CallFrame::new(&call));
        }
        let tid_str = thread_info[4]
            .split("=")
            .nth(1)
            .unwrap()
            .trim_start_matches("0x");
        let tid = u64::from_str_radix(tid_str, 16).unwrap();

        let nid_str = thread_info[5]
            .split("=")
            .nth(1)
            .unwrap()
            .trim_start_matches("0x");
        let nid = u64::from_str_radix(nid_str, 16).unwrap();
        Thread {
            id: thread_info[1].to_string(),
            name: thread_info[0].trim_matches('\"').to_string(),
            prio: thread_info[2]
                .split("=")
                .nth(1)
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            os_prio: thread_info[3]
                .split("=")
                .nth(1)
                .unwrap()
                .parse::<u32>()
                .unwrap(),
            tid,
            nid,
            status,
            frames,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub enum ThreadStatus {
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
    New,
}

impl FromStr for ThreadStatus {
    type Err = ();
    fn from_str(status: &str) -> Result<Self, Self::Err> {
        lazy_static::lazy_static! {
            static ref REGEX:Regex = Regex::new(r"State:\s(\w+)").unwrap();
        }

        if let Some(captures) = REGEX.captures(status) {
            match &captures[1] {
                "NEW" => Ok(ThreadStatus::New),
                "WAITING" => Ok(ThreadStatus::Waiting),
                "BLOCKED" => Ok(ThreadStatus::Blocked),
                "TIMED_WAITING" => Ok(ThreadStatus::TimedWaiting),
                "RUNNABLE" => Ok(ThreadStatus::Runnable),
                "TERMINATED" => Ok(ThreadStatus::Terminated),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CallFrame {
    class_name: String,
    method_name: Option<String>,
    line_number: Option<u32>,
    frame: Frame,
}

impl CallFrame {
    pub fn new(frame_info: &str) -> Self {
        let parts: Option<(&str, &str)>= frame_info.split_once(" ");
        let (method_name, class_name, line_number) = match parts {
            Some((key, method_info)) => {
                match key {
                    "-" => {
                        let (_, after_prefix) = method_info.split_once("(a ").unwrap_or(("",""));
                        let class_name = after_prefix.split(")").next().unwrap_or("Unknown Source").trim().to_string();
                        (None, class_name, None)
                     },
                     "at" => {
                         let method_parts: Vec<&str> = method_info.split('(').collect();
                         let method_name = Some(method_parts[0].to_string());
         
                         let file_info = method_parts.get(1).map(|s| s.trim_end_matches(')'));
                         let (class_name, line_number) = if let Some(file_info) = file_info {
                             let file_parts: Vec<&str> = file_info.split(':').collect();
                             let class_name = file_parts
                                 .get(0)
                                 .map(|s| s.split(".").next().unwrap().to_string())
                                 .unwrap();
                             let line_number: Option<u32> =
                                 file_parts.get(1).map(|s| s.parse().unwrap());
                             (class_name, line_number)
                         } else {
                             ("Unknown Source".to_string(), None)
                         };
                         (method_name, class_name, line_number)
                     },
                     _ => (None, "Unknown Source".to_string(), None)
                }
            },
            None => (None, "Unknown Source".to_string(), None)
        };
        CallFrame {
            class_name,
            line_number,
            method_name,
            frame: Frame::from_str(frame_info).unwrap(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub enum Frame {
    MethodCall,
    Lock {
        lock_address: u64,
    },
    Monitor {
        monitor_address: u64,
        action: MonitorAction,
    },
    Parking{
        parking_address: u64,
    },
    NativeMethod,
}

impl FromStr for Frame {
    type Err = ();

    fn from_str(frame_info: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = frame_info.split_whitespace().collect();
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
                    }
                }),
                "parking" => Ok(Frame::Parking { parking_address: extract_address(frame_info)}),
                _ => Err(()),
            },
            "at" => {
                if frame_info.contains("(Native Method)") {
                    Ok(Frame::NativeMethod)
                }else {
                    Ok(Frame::MethodCall)
                }
            },
            _ => Err(()),
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
#[derive(Serialize, Debug, PartialEq)]
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
          "\"Thread-1\" #1 prio=5 os_prio=0 tid=0x00007f3d70001800 nid=0x2f03 runnable [0x00007f3d80f21000]",
          "java.lang.Thread.State: RUNNABLE",
          "at com.example.MyClass.myMethod(MyClass.java:10)",
          "at com.example.MyClass.run(MyClass.java:5)",
          "at java.lang.Thread.run(Thread.java:748)"
    ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(5),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.myMethod".to_string()),
            line_number: Some(10),
            frame: Frame::MethodCall,
        });
        let thread = Thread {
            id: "#1".to_string(),
            name: "Thread-1".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70001800,
            nid: 0x2f03,
            status: ThreadStatus::Runnable,
            frames,
        };
        assert_eq!(result, thread)
    }

    #[test]
    pub fn test_blocked_thread() {
        let lines = vec![
          "\"Thread-2\" #2 prio=5 os_prio=0 tid=0x00007f3d70002800 nid=0x2f04 waiting for monitor entry [0x00007f3d80f22000]",
          "java.lang.Thread.State: BLOCKED (on object monitor)",
          "at com.example.MyClass.synchronizedMethod(MyClass.java:20)",
          "- waiting to lock <0x00000000c7c600d0> (a java.lang.Object)",
          "at com.example.MyClass.run(MyClass.java:15)",
          "at java.lang.Thread.run(Thread.java:748)"     
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(15),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Monitor { monitor_address: 0x00000000c7c600d0, action:  MonitorAction::WaitingToLock},
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.synchronizedMethod".to_string()),
            line_number: Some(20),
            frame: Frame::MethodCall,
        });
        let thread = Thread {
            id: "#2".to_string(),
            name: "Thread-2".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70002800,
            nid: 0x2f04,
            status: ThreadStatus::Blocked,
            frames,
        };
        assert_eq!(result, thread)
    }

    #[test]
    pub fn test_waiting_thread() {
        let lines = vec![
          "\"Thread-3\" #3 prio=5 os_prio=0 tid=0x00007f3d70003800 nid=0x2f05 in Object.wait() [0x00007f3d80f23000]",
          "java.lang.Thread.State: WAITING (on object monitor)",
          "at java.lang.Object.wait(Native Method)",
          "- waiting on <0x00000000c7c600d0> (a java.lang.Object)",
          "at java.lang.Object.wait(Object.java:502)",
          "at com.example.MyClass.waitMethod(MyClass.java:30)",
          "- locked <0x00000000c7c600d0> (a java.lang.Object)",
          "at com.example.MyClass.run(MyClass.java:25)",
          "at java.lang.Thread.run(Thread.java:748)",
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(25),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Lock { lock_address: 0x00000000c7c600d0 },
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.waitMethod".to_string()),
            line_number: Some(30),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "Object".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: Some(502),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Monitor { monitor_address: 0x00000000c7c600d0, action: MonitorAction::WaitingOn },
        });
        frames.push(CallFrame {
            class_name: "Native Method".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        let thread = Thread {
            id: "#3".to_string(),
            name: "Thread-3".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70003800,
            nid: 0x2f05,
            status: ThreadStatus::Waiting,
            frames,
        };
        assert_eq!(result, thread)
    }
    #[test]
    pub fn test_time_waiting_sleep_thread() {
        let lines = vec![
          "\"Thread-4\" #4 prio=5 os_prio=0 tid=0x00007f3d70004800 nid=0x2f06 waiting on condition [0x00007f3d80f24000]",
          "java.lang.Thread.State: TIMED_WAITING (sleeping)",
          "at java.lang.Thread.sleep(Native Method)",
          "at com.example.MyClass.sleepMethod(MyClass.java:40)",
          "at com.example.MyClass.run(MyClass.java:35)",
          "at java.lang.Thread.run(Thread.java:748)",
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(35),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.sleepMethod".to_string()),
            line_number: Some(40),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "Native Method".to_string(),
            method_name: Some("java.lang.Thread.sleep".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
       
        let thread = Thread {
            id: "#4".to_string(),
            name: "Thread-4".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70004800,
            nid: 0x2f06,
            status: ThreadStatus::TimedWaiting,
            frames,
        };
        assert_eq!(result, thread)
    }
    #[test]
    pub fn test_time_wating_monitor_thread() {
        let lines = vec![
          "\"Thread-7\" #7 prio=5 os_prio=0 tid=0x00007f3d70007800 nid=0x2f09 timed waiting on object monitor [0x00007f3d80f26000]",
          "java.lang.Thread.State: TIMED_WAITING (on object monitor)",
          "at java.lang.Object.wait(Native Method)",
          "- waiting on <0x00000000c7c600d0> (a java.lang.Object)",
          "at java.lang.Object.wait(Object.java:502)",
          "at com.example.MyClass.timedWaitMethod(MyClass.java:60)",
          "- locked <0x00000000c7c600d0> (a java.lang.Object)",
          "at com.example.MyClass.run(MyClass.java:55)",
          "at java.lang.Thread.run(Thread.java:748)"
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(55),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Lock { lock_address: 0x00000000c7c600d0 } ,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.timedWaitMethod".to_string()),
            line_number: Some(60),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "Object".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: Some(502),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.lang.Object".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Monitor { monitor_address: 0x00000000c7c600d0, action: MonitorAction::WaitingOn },
        });
        frames.push(CallFrame {
            class_name: "Native Method".to_string(),
            method_name: Some("java.lang.Object.wait".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        let thread = Thread {
            id: "#7".to_string(),
            name: "Thread-7".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70007800,
            nid: 0x2f09,
            status: ThreadStatus::TimedWaiting,
            frames,
        };
        assert_eq!(result, thread)
    }

    #[test]
    pub fn test_time_wating_condition_thread() {
        let lines = vec![
          "\"Thread-8\" #8 prio=5 os_prio=0 tid=0x00007f3d70008800 nid=0x2f0a waiting on condition [0x00007f3d80f27000]",
          "java.lang.Thread.State: TIMED_WAITING (on a condition)",
          "at sun.misc.Unsafe.park(Native Method)",
          "- parking to wait for  <0x00000002a5bdfc00> (a java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject)",
          "at java.util.concurrent.locks.LockSupport.parkNanos(LockSupport.java:215)",
          "at java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos(AbstractQueuedSynchronizer.java:2078)",
          "at com.example.MyClass.timedConditionMethod(MyClass.java:70)",
          "at com.example.MyClass.run(MyClass.java:65)",
          "at java.lang.Thread.run(Thread.java:748)",
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(65),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.timedConditionMethod".to_string()),
            line_number: Some(70),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "AbstractQueuedSynchronizer".to_string(),
            method_name: Some("java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos".to_string()),
            line_number: Some(2078),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "LockSupport".to_string(),
            method_name: Some("java.util.concurrent.locks.LockSupport.parkNanos".to_string()),
            line_number: Some(215),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Parking { parking_address: 0x00000002a5bdfc00 },
        });
        frames.push(CallFrame {
            class_name: "Native Method".to_string(),
            method_name: Some("sun.misc.Unsafe.park".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        let thread = Thread {
            id: "#8".to_string(),
            name: "Thread-8".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70008800,
            nid: 0x2f0a,
            status: ThreadStatus::TimedWaiting,
            frames,
        };
        assert_eq!(result, thread)
    }

    #[test]
    pub fn test_waiting_parking_thread() {
        let lines = vec![
          "\"Thread-6\" #6 prio=5 os_prio=0 tid=0x00007f3d70006800 nid=0x2f08 waiting on condition [0x00007f3d80f25000]",
          "java.lang.Thread.State: WAITING (parking)",
          "at sun.misc.Unsafe.park(Native Method)",
          "- parking to wait for  <0x00000002a5bdfc00> (a java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject)",
          "at java.util.concurrent.locks.LockSupport.park(LockSupport.java:175)",
          "at java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await(AbstractQueuedSynchronizer.java:2039)",
          "at com.example.MyClass.conditionMethod(MyClass.java:50)",
          "at com.example.MyClass.run(MyClass.java:45)",
          "at java.lang.Thread.run(Thread.java:748)"
        ];
        let result = Thread::new(lines);
        let mut frames: Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
            class_name: "Thread".to_string(),
            method_name: Some("java.lang.Thread.run".to_string()),
            line_number: Some(748),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.run".to_string()),
            line_number: Some(45),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "MyClass".to_string(),
            method_name: Some("com.example.MyClass.conditionMethod".to_string()),
            line_number: Some(50),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "AbstractQueuedSynchronizer".to_string(),
            method_name: Some("java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await".to_string()),
            line_number: Some(2039),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "LockSupport".to_string(),
            method_name: Some("java.util.concurrent.locks.LockSupport.park".to_string()),
            line_number: Some(175),
            frame: Frame::MethodCall,
        });
        frames.push(CallFrame {
            class_name: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject".to_string(),
            method_name: None,
            line_number: None,
            frame: Frame::Parking { parking_address: 0x00000002a5bdfc00 },
        });
        frames.push(CallFrame {
            class_name: "Native Method".to_string(),
            method_name: Some("sun.misc.Unsafe.park".to_string()),
            line_number: None,
            frame: Frame::NativeMethod,
        });
        let thread = Thread {
            id: "#6".to_string(),
            name: "Thread-6".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70006800,
            nid: 0x2f08,
            status: ThreadStatus::Waiting,
            frames,
        };
        assert_eq!(result, thread)
    }
}
