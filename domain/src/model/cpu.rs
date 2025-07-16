use std::str::FromStr;

use chrono::{NaiveTime};
use serde::{Deserialize, Serialize};
use common::time_utils;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Cpu {
    pub file_id: String,
    pub work_space: String,
    pub exe_time: NaiveTime,
    pub us: f64,
    pub sy: f64,
    pub ids: f64,
    pub tasks: u32,
    pub running: u32,
    pub sleeping: u32,
    pub mem_total: f64,
    pub mem_free: f64,
    pub mem_used: f64,
}

impl Cpu {
    pub fn new(lines: Vec<String>, file_id: &str, work_space: &str) -> Self {
        let time = Self::extract_main(lines.get(0).unwrap());
        let (tasks, running, sleeping) = Self::extract_threads(lines.get(1).unwrap());
        let (us, sy, id) = Self::extract_cpu(lines.get(2).unwrap());
        let (mem_total, mem_free, mem_used) = Self::extract_mem(lines.get(3).unwrap());
        // let process: Vec<Process> = Vec::with_capacity(0);
        // for i in 7..lines.len() {
        //     process.push(Process::from_str(&lines[i]).unwrap());
        // }
        Cpu {
            file_id: file_id.into(),
            work_space: work_space.into(),
            exe_time: time_utils::parse_time(&time).unwrap(),
            us,
            sy,
            ids:id,
            tasks,
            running,
            sleeping,
            mem_total,
            mem_free,
            mem_used,
        }
    }
    fn extract_main(line: &str) -> String {
        let infos: Vec<&str> = line.split(",").collect();
        let cpu_running: Vec<&str> = infos[0].split_whitespace().collect();
        cpu_running[2].to_string()
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
    fn extract_mem(line: &str) -> (f64, f64, f64) {
        let mem_line = line.split(":").nth(1).unwrap();
        let mem_vec: Vec<&str> = mem_line.split(",").collect();
        let total = mem_vec
            .get(0)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        let free = mem_vec
            .get(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        let used = mem_vec
            .get(2)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        (total, free, used)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Process {
    pid: u32,
    usr: String,
    cpu: f64,
    mem: f64,
    time: String,
    command: String,
}

impl FromStr for Process {
    type Err = ();
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let value: Vec<&str> = line.split_whitespace().collect();
        Ok(Process {
            pid: value[0].parse::<u32>().unwrap(),
            usr: value[1].to_string(),
            cpu: value[8].parse::<f64>().unwrap(),
            mem: value[9].parse::<f64>().unwrap(),
            time: value[10].to_string(),
            command: value[11].to_string(),
        })
    }
}


#[derive(Serialize)]
pub struct CpuCount {
    pub exe_time: Vec<NaiveTime>,
    pub us: Vec<f64>,
    pub sy: Vec<f64>,
    pub ids: Vec<f64>,
}

#[cfg(test)]
pub mod test {
    use super::Cpu;
    use std::{
        fs,
        io::{self, BufRead},
    };

    #[test]
    pub fn test_cpu() {
        let file = fs::File::open("D:\\dump\\20240809\\日志\\异步节点\\20240809.tar\\20240809\\20240809_22\\20240809_223714\\cpu_top_17606.log").unwrap();
        let reader = io::BufReader::new(file);
        let mut lines_storage: Vec<String> = Vec::new();

        for line in reader.lines() {
            lines_storage.push(line.unwrap());
        }

        let result = Cpu::new(lines_storage,"A","c");
        println!("{:?}", result);
    }
}
