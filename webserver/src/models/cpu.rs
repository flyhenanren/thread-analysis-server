pub struct Cpu {
    us: f64,
    sy: f64,
    id: f64,
    mem: f64,
    tasks: u32,
    running: u32,
    sleeping: u32,
    mem_total: f64,
    mem_free: f64,
    mem_used: f64,
    process: Vec<Process>,
}

impl Cpu {
    pub fn new(line: &str) -> Self {
        Cpu {
            us: todo!(),
            sy: todo!(),
            id: todo!(),
            mem: todo!(),
            tasks: todo!(),
            running: todo!(),
            sleeping: todo!(),
            mem_total: todo!(),
            mem_free: todo!(),
            mem_used: todo!(),
            process: todo!(),
        }
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


impl Process{
  pub fn new(lines: Vec<&str>) -> Self{
    Process{
        pid: todo!(),
        usr: todo!(),
        cpu: todo!(),
        mem: todo!(),
        time: todo!(),
        command: todo!(),
    }
  }
}