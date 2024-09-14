use chrono::{NaiveDateTime, ParseError};

pub fn is_valid_datetime(input: &str) -> bool {
  NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").is_ok()
}

pub fn parse_time(input: &str)-> Result<NaiveDateTime, ParseError> {
  NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
}