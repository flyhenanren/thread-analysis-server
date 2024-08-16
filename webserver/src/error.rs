use serde::Serialize;
use std::{fmt, num::ParseIntError};


#[derive(Debug, Serialize)]
pub enum FileError{
  NotFound(String),
  IllegalName(String)
}

#[derive(Debug)]
pub enum ThreadError{
  IllegalStatus(String),
  ParseError(String),
  RegexError(regex::Error),
  ParseIntError(ParseIntError),
  MissingField,
  InvalidStatus,
}

impl fmt::Display for FileError{
    fn fmt(&self, f: &mut fmt::Formatter) ->  Result<(), fmt::Error>{
      write!(f, "{}", self)
    }
}

// 实现 `fmt::Display` 为 `ThreadError`
impl fmt::Display for ThreadError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
          ThreadError::IllegalStatus(s) => write!(f, "Illegal status: {}", s),
          ThreadError::ParseError(s) => write!(f, "Parse Error: {}", s),
          ThreadError::RegexError(e) => write!(f, "Regex Error:{}", e),
          ThreadError::ParseIntError(e) => write!(f, "ParseIntError:{}",e),
          ThreadError::MissingField => write!(f, "MissingField"),
          ThreadError::InvalidStatus => write!(f, "InvalidStatus"),
      }
  }
}

// 实现 `std::error::Error` 为 `ThreadError`
impl std::error::Error for ThreadError {}