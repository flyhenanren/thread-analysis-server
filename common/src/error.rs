use actix_web::{error, http::StatusCode, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::error::Error as SQLxError;
use tantivy::{directory::error::OpenDirectoryError, TantivyError};
use std::{fmt, io::Error, num::ParseIntError};

#[derive(Debug, Serialize, Deserialize)]
pub enum AnalysisError {
    DBError(String),
    ActixError(String),
    NotFound(String),
    IoError(String),
    ParseError(String),
    RegError(String)
}


#[derive(Debug, Serialize, Deserialize)]
pub enum DBError {
    Execute(String),
}

#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    error_message: String,
}

#[derive(Debug)]
pub enum ThreadError {
    IllegalStatus(String),
    ParseError(String),
    RegexError(regex::Error),
    ParseIntError(ParseIntError),
    MissingField(String),
    ParseFrame(String),
    InvalidStatus,
}

#[derive(Debug)]
pub enum FrameError {
    Unknown(String),
}

impl AnalysisError {
    fn error_response(&self) -> String {
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
            AnalysisError::ParseError(msg) => {
                println!("Not found error occurred:{:?}", msg);
                msg.into()
            }
            AnalysisError::IoError(msg) => {
                println!("Io error:{:?}", msg);
                msg.into()
            },
            AnalysisError::RegError(msg) => {
                println!("Regex error:{:?}", msg);
                msg.into()
            }
        }
    }
}

impl error::ResponseError for AnalysisError {
    fn status_code(&self) -> StatusCode {
        match self {
            AnalysisError::NotFound(_msg) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AnalysisResponse {
            error_message: self.error_response(),
        })
    }
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl From<actix_web::error::Error> for AnalysisError {
    fn from(value: actix_web::error::Error) -> Self {
        AnalysisError::ActixError(value.to_string())
    }
}

impl From<SQLxError> for AnalysisError {
    fn from(value: SQLxError) -> Self {
        AnalysisError::DBError(value.to_string())
    }
}

// 实现 `fmt::Display` 为 `ThreadError`
impl fmt::Display for ThreadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ThreadError::IllegalStatus(s) => write!(f, "Illegal status: {}", s),
            ThreadError::ParseError(s) => write!(f, "Parse Error: {}", s),
            ThreadError::RegexError(e) => write!(f, "Regex Error:{}", e),
            ThreadError::ParseIntError(e) => write!(f, "ParseIntError:{}", e),
            ThreadError::MissingField(e) => write!(f, "MissingField:{}", e),
            ThreadError::InvalidStatus => write!(f, "InvalidStatus"),
            ThreadError::ParseFrame(e) => write!(f, "ParseFrame"),
        }
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Unknown(str) => write!(f, "Unknown frame: {}", str),
        }
    }
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBError::Execute(str) => write!(f, "sql execute error: {}", str),
        }
    }
}

// 实现 `std::error::Error` 为 `ThreadError`
impl std::error::Error for ThreadError {}

impl std::error::Error for FrameError {}

impl From<FrameError> for ThreadError {
    fn from(error: FrameError) -> ThreadError {
        ThreadError::ParseFrame(error.to_string())
    }
}

impl From<ThreadError> for AnalysisError {
    fn from(error: ThreadError) -> Self {
        AnalysisError::ParseError(error.to_string())
    }
}

impl From<DBError> for AnalysisError {
    fn from(error: DBError) -> Self {
        AnalysisError::DBError(error.to_string())
    }
}


impl From<SQLxError> for DBError {
    fn from(value: SQLxError) -> Self {
        DBError::Execute(value.to_string())
    }
}


impl From<Error> for AnalysisError {
    fn from(value: Error) -> Self {
        AnalysisError::ParseError(value.to_string())
    }
}


impl From<OpenDirectoryError> for AnalysisError {
    fn from(value: OpenDirectoryError) -> Self {
        AnalysisError::IoError(value.to_string())
    }
}


impl From<TantivyError> for AnalysisError {
    fn from(value: TantivyError) -> Self {
        AnalysisError::IoError(value.to_string())
    }
}