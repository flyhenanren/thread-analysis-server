use serde::{Deserialize, Serialize};
use std::{fmt, num::ParseIntError};
use actix_web::{error, http::StatusCode, HttpResponse, Result};

#[derive(Debug, Serialize, Deserialize)]
pub enum AnalysisError{
  DBError(String),
  ActixError(String),
  NotFound(String),
}

#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
  error_message: String,
}

impl AnalysisError{
  fn error_response(&self) -> String{
    match self {
      AnalysisError::DBError(msg) => {
        println!("Database error occurred:{:?}", msg);
        "Database error".into()
      }
      AnalysisError::ActixError(msg) => {
        println!("Server error occurred:{:?}", msg);
        "Internal server error".into()
      }
      AnalysisError::NotFound(msg) => {
        println!("Not found error occurred:{:?}", msg);
        msg.into()
      }
    }
  }
}

impl error::ResponseError for AnalysisError{
  fn status_code(&self) -> StatusCode{
    match self {
      AnalysisError::DBError(_msg) | AnalysisError::ActixError(_msg) => StatusCode::INTERNAL_SERVER_ERROR,
      AnalysisError::NotFound(_msg) => StatusCode::NOT_FOUND,
    }
  }

  fn error_response (&self) -> HttpResponse{
    HttpResponse::build(self.status_code()).json(AnalysisResponse{
      error_message: self.error_response(),
    })
  }
}

impl fmt::Display for AnalysisError{
    fn fmt(&self, f: &mut fmt::Formatter) ->  Result<(), fmt::Error>{
      write!(f, "{}", self)
    }
}

impl From<actix_web::error::Error> for AnalysisError{
  fn from(value: actix_web::error::Error) -> Self {
    AnalysisError::ActixError(value.to_string())
  }
}



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
  MissingField(String),
  InvalidStatus,
}

#[derive(Debug)]
pub enum FrameError{
  Unknown(String)
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
          ThreadError::MissingField(e) => write!(f, "MissingField:{}",e),
          ThreadError::InvalidStatus => write!(f, "InvalidStatus"),
      }
  }
}

impl fmt::Display for FrameError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Unknown(str) => write!(f, "Unknown frame: {}", str),
        }
    }
}

// 实现 `std::error::Error` 为 `ThreadError`
impl std::error::Error for ThreadError {}

impl std::error::Error for FrameError{}