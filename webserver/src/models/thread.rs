use std::str::FromStr;

use serde::Serialize;
// "JOB_BI_JOB_THREAD_8" #488 prio=5 os_prio=0 tid=0x0000fffb30011000 nid=0xf1d43 runnable [0x0000fff7575fd000]

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
        let infos: Vec<&str> = lines[0].split_whitespace().collect();
        let status = ThreadStatus::from_str(lines[1]).unwrap();
        let call_info = &lines[2..=lines.len() - 1];
        let mut frames: Vec<CallFrame> = Vec::with_capacity(call_info.len());
        for call in call_info {
            frames.push(CallFrame::new(&call));
        }
        Thread {
            id: infos[1].to_string(),
            name: infos[0].to_string(),
            prio: infos[2].split("=").nth(1).unwrap().parse::<u16>().unwrap(),
            os_prio: infos[3].split("=").nth(1).unwrap().parse::<u32>().unwrap(),
            tid: infos[4].split("=").nth(1).unwrap().parse::<u64>().unwrap(),
            nid: infos[5].split("=").nth(1).unwrap().parse::<u64>().unwrap(),
            status,
            frames,
        }
    }
}

#[derive(Serialize, Debug,PartialEq)]
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
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        if let Some((state, info)) = str.split_once(':') {
            match info.trim() {
                "NEW" => Ok(ThreadStatus::New),
                "WAITING" => Ok(ThreadStatus::Waiting),
                "BLOCKED" => Ok(ThreadStatus::Blocked),
                "TIMED_WAITING" => Ok(ThreadStatus::TimedWaiting),
                "RUNNABLE" => Ok(ThreadStatus::Runnable),
                "TERMINATED" => Ok(ThreadStatus::Terminated),
                _ => Err(()),
            }
        } else {
            match str.trim() {
                "NEW" => Ok(ThreadStatus::New),
                "WAITING" => Ok(ThreadStatus::Waiting),
                "BLOCKED" => Ok(ThreadStatus::Blocked),
                "TIMED_WAITING" => Ok(ThreadStatus::TimedWaiting),
                "RUNNABLE" => Ok(ThreadStatus::Runnable),
                "TERMINATED" => Ok(ThreadStatus::Terminated),
                _ => Err(()),
            }
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CallFrame {
    class_name: String,
    method_name: String,
    line_number: Option<u32>,
    frame: Frame,
}

impl CallFrame {
    pub fn new(frame_info: &str) -> Self {
        let parts: Vec<&str> = frame_info.split_whitespace().collect();
        let method_info: &str = parts[1];

        let method_parts: Vec<&str> = method_info.split('(').collect();
        let method_name = method_parts[0].to_string();

        let file_info = method_parts.get(1).map(|s| s.trim_end_matches(')'));
        let (class_name, line_number) = if let Some(file_info) = file_info {
            let file_parts: Vec<&str> = file_info.split(':').collect();
            let class_name = file_parts.get(0).map(|s| s.split(".").collect());
            let line_number = file_parts.get(1).map(|s| s.parse().unwrap_or(0));
            (class_name, line_number)
        } else {
            (None, None)
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
        lock_address: String,
    },
    Monitor {
        monitor_address: String,
        action: MonitorAction,
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
                    },
                }),
                _ => Err(()),
            },
            "at" => match parts[2] {
                "Native Method" => Ok(Frame::NativeMethod),
                _ => Ok(Frame::MethodCall),
            },
            _ => Err(()),
        }
    }
}

fn extract_address(input: &str) -> String {
    let mut results = Vec::new();
    let parts: Vec<&str> = input.split('<').collect();

    for part in parts.iter().skip(1) {
        let content: Vec<&str> = part.split('>').collect();
        if let Some(content) = content.get(0) {
            results.push(content.trim().to_string());
        }
    }

    results.concat()
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
        let mut frames:Vec<CallFrame> = Vec::new();
        frames.push(CallFrame {
          class_name: "Thread".to_string(),
          method_name: "java.lang.Thread.run".to_string(),
          line_number: Some(748),
          frame: Frame::MethodCall,
        });
        let thread = Thread {
            id: "1".to_string(),
            name: "Thread-1".to_string(),
            prio: 5,
            os_prio: 0,
            tid: 0x00007f3d70001800,
            nid: 0x2f03,
            status: ThreadStatus::Runnable,
            frames,
        };
        println!("{:?}", result);
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
        print!("result:{:?}", result)
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
        print!("result:{:?}", result)
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
        print!("result:{:?}", result)
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
        print!("result:{:?}", result)
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
        print!("result:{:?}", result)
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
        print!("result:{:?}", result)
    }
}
