use chrono::{NaiveDateTime, NaiveTime, ParseError};
use rand::Rng;

pub fn is_valid_datetime(input: &str) -> bool {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").is_ok()
}


pub fn parse_data_time(input: &str) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
}
pub fn parse_time(input: &str) -> Result<NaiveTime, ParseError> {
    NaiveTime::parse_from_str(input, "%H:%M:%S")
}

pub fn parse_thread_time(input: &str) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::parse_from_str(input, "%Y%m%d_%H%M%S")
}

pub fn rand_id() -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
  abcdefghijklmnopqrstuvwxyz\
  0123456789";
  let mut rng = rand::thread_rng();

  // 使用自定义字符集生成指定长度的随机字符串
  (0..=16)
      .map(|_| charset[rng.gen_range(0..charset.len())] as char)
      .collect()
}
