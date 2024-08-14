use chrono::NaiveTime;

#[derive(Debug, PartialEq)]
pub struct Cpu {
    time: String,
    us: f64,
    sy: f64,
    id: f64,
    tasks: u32,
    running: u32,
    sleeping: u32,
    mem_total: u64,
    mem_free: u64,
    mem_used: u64,
    process: Vec<Process>,
}

impl Cpu {
    pub fn new(lines: Vec<&str>) -> Self {
        let time = Self::extract_main(lines.get(0).unwrap());
        let (tasks, running, sleeping) = Self::extract_threads(lines.get(1).unwrap());
        let (us, sy, id) = Self::extract_cpu(lines.get(2).unwrap());
        let (mem_total, mem_free, mem_used) = Self::extract_mem(lines.get(3).unwrap());
        let mut process:Vec<Process> = Vec::with_capacity(lines.len() - 6);
        for i in 7..lines.len() {
            process.push(Process::new(lines[i]));
        }
        Cpu {
            time,
            us,
            sy,
            id,
            tasks,
            running,
            sleeping,
            mem_total,
            mem_free,
            mem_used,
            process,
        }
    }
    fn extract_main(line: &str) -> String {
        let infos: Vec<&str> = line.split(",").collect();
        let cpu_running: Vec<&str> = infos[0].split_whitespace().collect();
        cpu_running[cpu_running.len() - 2].to_string()
    }

    fn extract_threads(line: &str) -> (u32, u32, u32) {
        let tasks: Vec<&str> = line.split(",").collect();
        let total_info: Vec<&str> = tasks[0].split_whitespace().collect();
        let running_info: Vec<&str> = tasks[1].split_whitespace().collect();
        let sleep_info: Vec<&str> = tasks[2].split_whitespace().collect();

        let total = total_info.get(1).unwrap().parse::<u32>().unwrap();
        let running = running_info.get(0).unwrap().parse::<u32>().unwrap();
        let sleep = sleep_info.get(0).unwrap().parse::<u32>().unwrap();
        (total, running, sleep)
    }

    fn extract_cpu(line: &str) -> (f64, f64, f64) {
        let cpu_line = line.split(":").nth(1).unwrap();
        let cpu_vec: Vec<&str> = cpu_line.split(",").collect();
        let us = cpu_vec
            .get(0)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        let sy = cpu_vec
            .get(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        let id = cpu_vec
            .get(3)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        (us, sy, id)
    }
    fn extract_mem(line: &str) -> (u64, u64, u64) {
        let mem_line = line.split(":").nth(1).unwrap();
        let mem_vec: Vec<&str> = mem_line.split(",").collect();
        let total = mem_vec
            .get(0)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let free = mem_vec
            .get(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let used = mem_vec
            .get(2)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        (total, free, used)
    }
}

#[derive(Debug, PartialEq)]
pub struct Process {
    pid: u32,
    usr: String,
    cpu: f64,
    mem: f64,
    time: String,
    command: String,
}

impl Process {
    pub fn new(line: &str) -> Self {
        let value:Vec<&str> = line.split_whitespace().collect();
        Process {
            pid: value[0].parse::<u32>().unwrap(),
            usr: value[1].to_string(),
            cpu: value[8].parse::<f64>().unwrap(),
            mem: value[9].parse::<f64>().unwrap(),
            time:value[10].to_string(),
            command: value[11].to_string(),
        }
    }
}


#[cfg(test)]
pub mod test{
    use std::{fs, io::{self, BufRead}};
    use super::Cpu;


    #[test]
    pub fn test_cpu(){
        let file = fs::File::open("D:\\dump\\20240809\\日志\\异步节点\\20240809.tar\\20240809\\20240809_22\\20240809_223714\\cpu_top_17606.log").unwrap();
        let reader = io::BufReader::new(file);
        let mut lines_str:Vec<&str> = Vec::new();
        let mut lines_storage:Vec<String> = Vec::new();

        for line in reader.lines() {
            lines_storage.push(line.unwrap());
        }

        for s in &lines_storage {
            lines_str.push(s);
        }
        let result = Cpu::new(lines_str);
        println!("{:?}", result);
    }
}