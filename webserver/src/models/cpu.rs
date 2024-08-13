pub struct Cpu {
    time: String,
    running_time: u64,
    us: f64,
    sy: f64,
    id: f64,
    tasks: u32,
    running: u32,
    sleeping: u32,
    mem_total: f64,
    mem_free: f64,
    mem_used: f64,
    process: Vec<Process>,
}

impl Cpu {
    pub fn new(lines: Vec<&str>) -> Self {
      let (time, running_time) = Self::extract_cpu(lines.get(0).unwrap());
      let (tasks, running, sleeping) = Self::extract_task(lines.get(1).unwrap());
        Cpu {
            time,
            running_time,
            us: todo!(),
            sy: todo!(),
            id: todo!(),
            tasks,
            running,
            sleeping,
            mem_total: todo!(),
            mem_free: todo!(),
            mem_used: todo!(),
            process: todo!(),
        }
    }
    fn extract_cpu(line: &str) -> (String, u64) {
        let infos: Vec<&str> = line.split(",").collect();
        let cpu_running:Vec<&str> = infos[1].split_whitespace().collect();
        (cpu_running[2].to_string(), u64::from_str_radix(cpu_running[4], 16).unwrap())
    }

    fn extract_task(line: &str) -> (u32, u32, u32){
      let tasks:Vec<&str> = line.split(",").collect();
      let total_info:Vec<&str> = tasks.get(0).unwrap().split_whitespace().collect();
      let running_info:Vec<&str> = tasks.get(1).unwrap().split_whitespace().collect();
      let sleep_info:Vec<&str> = tasks.get(2).unwrap().split_whitespace().collect();
      let total = total_info.get(1).unwrap().parse::<u32>().unwrap();
      let running = running_info.get(0).unwrap().parse::<u32>().unwrap();
      let sleep = sleep_info.get(0).unwrap().parse::<u32>().unwrap();
      (total, running, sleep)
    }

}

pub struct Process {
    pid: u32,
    usr: String,
    cpu: f64,
    mem: f64,
    time: String,
    command: String,
}

impl Process {
    pub fn new(lines: Vec<&str>) -> Self {
        Process {
            pid: todo!(),
            usr: todo!(),
            cpu: todo!(),
            mem: todo!(),
            time: todo!(),
            command: todo!(),
        }
    }
}
