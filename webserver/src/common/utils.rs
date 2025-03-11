use chrono::{NaiveDateTime, NaiveTime, ParseError};
use rand::Rng;
use uuid::Uuid;

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
    Uuid::new_v4().to_string()
}
