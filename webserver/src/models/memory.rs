use core::f64;

use chrono::NaiveDateTime;

#[derive(Debug, PartialEq)]
pub struct MemoryInfo {
    time: NaiveDateTime,
    value: Vec<f64>,
}
#[derive(Debug, PartialEq)]
pub struct MemoryPercent {
    time: NaiveDateTime,
    value: Vec<f64>,
}

impl MemoryInfo {
    pub fn new(time: NaiveDateTime, line: &str) -> Self{
        MemoryInfo{
          time,
          value: Self::split_value(line)
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
  pub fn new(time: NaiveDateTime, line: &str) -> Self{
    MemoryPercent{
        time,
        value: Self::split_value(line)
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
pub mod tests{

    use crate::models::memory;

    use super::*;

    #[test]
    pub fn test_gc(){
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
        let mut result: Vec<MemoryInfo> = Vec::new();
        for line in lines {
            if line.starts_with("S0C") {
                flag = true;
                continue;
            }
            if flag {
                let info: MemoryInfo = MemoryInfo::new(time, line);
                result.push(info);
                flag = false;     
            }
        }
        let memory = vec![MemoryInfo{ time, value: vec![0.0,
            786432.0,0.0,786432.0,5820416.0,625664.0,24850432.0,4080638.3,236180.0,225812.0,26332.0,24429.3,16.0,6.146,0.0,0.000,6.146] }];
        assert_eq!(result, memory);
    }
}