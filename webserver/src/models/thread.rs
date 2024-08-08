use std::str::FromStr;

use serde::Serialize;
// "JOB_BI_JOB_THREAD_8" #488 prio=5 os_prio=0 tid=0x0000fffb30011000 nid=0xf1d43 runnable [0x0000fff7575fd000]

#[derive(Serialize, Debug, Clone)]
pub struct Thread {
    id: String,
    name: String,
    prio: u16,
    os_prio: u32,
    tid: u64,
    nid: u64,
    status: ThreadStatus,
}

impl Thread {
    pub fn new(info: &str, status: &str) -> Self {
        let infos: Vec<&str> = info.split_whitespace().collect();
        Thread {
            id: infos[1].to_string(),
            name: infos[0].to_string(),
            prio: infos[2].split("=").nth(1).unwrap().parse::<u16>().unwrap(),
            os_prio: infos[3].split("=").nth(1).unwrap().parse::<u32>().unwrap(),
            tid: infos[4].split("=").nth(1).unwrap().parse::<u64>().unwrap(),
            nid: infos[5].split("=").nth(1).unwrap().parse::<u64>().unwrap(),
            status: ThreadStatus::from_str(status).unwrap(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum ThreadStatus {
    NEW,
    RUNNABLE,
    BLOCKED,
    WAITING,
    TIMED_WAITING,
    TERMINATED,
}

impl FromStr for ThreadStatus {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        if let Some((state, info)) = str.split_once('(') {
            let state = state.trim();
            let _info = info.trim_end_matches(')').trim();
            match state {
                "NEW" => Ok(ThreadStatus::NEW),
                "WAITING" => Ok(ThreadStatus::WAITING),
                "BLOCKED" => Ok(ThreadStatus::BLOCKED),
                "TIMED_WAITING" => Ok(ThreadStatus::TIMED_WAITING),
                "RUNNABLE" => Ok(ThreadStatus::RUNNABLE),
                "TERMINATED" => Ok(ThreadStatus::TERMINATED),
                _ => Err(()),
            }
        } else {
            match str.trim() {
                "NEW" => Ok(ThreadStatus::NEW),
                "WAITING" => Ok(ThreadStatus::WAITING),
                "BLOCKED" => Ok(ThreadStatus::BLOCKED),
                "TIMED_WAITING" => Ok(ThreadStatus::TIMED_WAITING),
                "RUNNABLE" => Ok(ThreadStatus::RUNNABLE),
                "TERMINATED" => Ok(ThreadStatus::TERMINATED),
                _ => Err(()),
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StackTrace {
    class_name: String,
    line: u8,
    method: String,
    next: Option<Box<StackTrace>>,
}

impl StackTrace {
  
    pub fn new(mut lines: Vec<&str>) -> Self {
        lines.reverse();
        let (class_name, method, line) = Self::parse_stack_trace(lines[0]);
        lines.remove(0);
        let next:StackTrace;
        let next = if !lines.is_empty() {
           Some(Box::new(StackTrace::new(lines)))
        }else {
          None
        };
        StackTrace {
            class_name,
            line,
            method,
            next,
        }
    }

    fn parse_stack_trace(s: &str) -> (String, String, u8) {
        // Trim the leading "at " and remove the trailing ")"
        let s = s.trim_start_matches("at ").trim_end_matches(')');

        // Find the position of the opening parenthesis
        if let Some((method_and_class, rest)) = s.split_once('(') {
            // Split the method_and_class part by the last '.' to separate method from class
            let (class, method) = if let Some((class, method)) = method_and_class.rsplit_once('.') {
                (class.to_string(), method.to_string())
            } else {
                (method_and_class.to_string(), String::new())
            };

            // Extract line number
            let line_number = if let Some((_, line)) = rest.split_once(':') {
                line.parse::<u8>().unwrap_or(0)
            } else {
                0
            };

            (format!("{}.{}", class, method), class, line_number)
        } else {
            // Handle the case where the format might be unexpected
            ("".to_string(), "".to_string(), 0)
        }
    }
}
