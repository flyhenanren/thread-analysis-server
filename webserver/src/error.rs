use serde::Serialize;
use std::fmt;


#[derive(Debug, Serialize)]
pub enum FileError{
  NotFound(String),
  IllegalName(String)
}

#[derive(Debug, Serialize)]
pub enum ThreadError{
  IllegalStatus(String),
  ParseError(String),
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
      }
  }
}

// 实现 `std::error::Error` 为 `ThreadError`
impl std::error::Error for ThreadError {}