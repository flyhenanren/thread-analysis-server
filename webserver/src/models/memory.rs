use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use core::f64;
use std::io::{BufRead, BufWriter, Error, Read, Write};
use std::{fs, io, path::Path};

#[derive(Serialize,Deserialize,Debug, PartialEq)]
pub struct MemoryValue {
    pub file_id: String,
    pub time: NaiveDateTime,
    pub value: Vec<f64>,
}
#[derive(Serialize,Deserialize, Debug, PartialEq)]
pub struct MemoryPercent {
    time: NaiveDateTime,
    value: Vec<f64>,
}

impl MemoryValue {
    pub fn new(time: NaiveDateTime, filed_id: &str, line: &str) -> Self {
        MemoryValue {
            file_id: filed_id.into(),
            time,
            value: Self::split_value(line),
        }
    }

    fn split_value(line: &str) -> Vec<f64> {
        line.split_whitespace()
            .map(|l| {
                if l.eq("-") {
                    0.0;
                }
                match l.parse::<f64>() {
                    Ok(value) => value,
                    Err(e) => 0.0,
                }
            })
            .collect()
    }
}

impl MemoryPercent {
    pub fn new(time: NaiveDateTime, line: &str) -> Self {
        MemoryPercent {
            time,
            value: Self::split_value(line),
        }
    }

    fn split_value(line: &str) -> Vec<f64> {
        line.split_whitespace()
            .map(|l| {
                if l.eq("-") {
                    0.00;
                }
                match l.parse::<f64>() {
                    Ok(value) => value,
                    Err(e) => 0.00,
                }
            })
            .collect()
    }
}

pub fn create(path: &str) -> (MemoryValue, MemoryPercent) {
    let file_path = Path::new(path);
    let path_time = file_path.parent().expect("无法获取上级");
    let fmt = "%Y%m%d_%H%M%S";
    let time = NaiveDateTime::parse_from_str(path_time.file_name().unwrap().to_str().unwrap(), fmt)
        .unwrap();
    let file = fs::File::open(path).unwrap();
    let reader = io::BufReader::new(file);
    let mut idx = 0;
    let mut mem_info: Option<MemoryValue> = None;
    let mut mem_percent: Option<MemoryPercent> = None;
    for line in reader.lines() {
        match idx {
            1 => mem_percent = Some(MemoryPercent::new(time, &line.unwrap())),
            3 => mem_info = Some(MemoryValue::new(time,path, &line.unwrap())),
            _ => {}
        }
        idx += 1;
    }
    let mem_info = mem_info.expect("未找到MemoryInfo数据");
    let mem_percent = mem_percent.expect("未找到MemoryPercent数据");
    (mem_info, mem_percent)
}

pub fn batch_crate_memory_info(
    file_path: &str,
    start: NaiveDateTime,
    cycle: i64,
) -> Vec<MemoryValue> {
    let mut memory_info = Vec::new();
    let file = fs::File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);
    let mut flag = false;
    let mut current_time = start;
    for line in reader.lines() {
        let line = line.unwrap();
        if line.contains(S0C) {
            flag =true;
            continue;
        }
        if flag {
            let mem_info = MemoryValue::new(current_time, file_path, &line);
            memory_info.push(mem_info);
            current_time = current_time.checked_add_signed(Duration::seconds(cycle.into())).expect("时间转换失败");
            flag = false;
        }
    }
    memory_info
}

pub fn batch_crate_memory_percent(
    file_path: &str,
    start: NaiveDateTime,
    cycle: u32,
) -> Vec<MemoryPercent> {
    let mut memory_info = Vec::new();
    let file = fs::File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);
    let mut flag = false;
    let mut current_time = start;
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with(S0) {
            flag =true;
            continue;
        }
        if flag {
            let mem_info = MemoryPercent::new(current_time, &line);
            memory_info.push(mem_info);
            current_time = current_time.checked_add_signed(Duration::seconds(cycle.into())).expect("时间转换失败");
            flag = false;
        }
    }
    memory_info
}

pub static S0: &str = "S0";
pub static S0C: &str = "S0C";
pub static S1C: &str = "S1C";
pub static S0U: &str = "S0U";
pub static S1U: &str = "S1U";
pub static EC: &str = "EC";
pub static EU: &str = "EU";
pub static OC: &str = "OC";
pub static OU: &str = "OU";
pub static MC: &str = "MC";
pub static MU: &str = "MU";
pub static CCSC: &str = "CCSC";
pub static CCSU: &str = "CCSU";
pub static YGC: &str = "YGC";
pub static YGCT: &str = "YGCT";
pub static FGC: &str = "FGC";
pub static FGCT: &str = "FGCT";
pub static CGC: &str = "CGC";
pub static CGCT: &str = "CGCT";
pub static GCT: &str = "GCT";

#[cfg(test)]
pub mod tests {

    use crate::models::memory;

    use super::*;

    #[test]
    pub fn test_gc() {
        let lines = vec![
            "S0     S1     E      O      M     CCS    YGC     YGCT    FGC    FGCT     GCT   ",
            "0.00 100.00  10.38  16.41  95.61  92.77     16    6.146     0    0.000    6.146",
            "S0C    S1C    S0U    S1U      EC       EU        OC         OU       MC     MU    CCSC   CCSU   YGC     YGCT    FGC    FGCT     GCT   ",
            "0.0   786432.0  0.0   786432.0 5820416.0 625664.0 24850432.0 4080638.3  236180.0 225812.0 26332.0 24429.3     16    6.146   0      0.000    6.146"
        ];
        let file_name = "20240809_170136";
        let fmt = "%Y%m%d_%H%M%S";
        let time = NaiveDateTime::parse_from_str(&file_name, fmt).unwrap();
        let mut flag = false;
        let mut result: Vec<MemoryValue> = Vec::new();
        for line in lines {
            if line.starts_with("S0C") {
                flag = true;
                continue;
            }
            if flag {
                let info: MemoryValue = MemoryValue::new(time, "", line);
                result.push(info);
                flag = false;
            }
        }
        let memory = vec![MemoryValue {
            file_id:"".to_string(),
            time,
            value: vec![
                0.0, 786432.0, 0.0, 786432.0, 5820416.0, 625664.0, 24850432.0, 4080638.3, 236180.0,
                225812.0, 26332.0, 24429.3, 16.0, 6.146, 0.0, 0.000, 6.146,
            ],
        }];
        assert_eq!(result, memory);
    }
}
