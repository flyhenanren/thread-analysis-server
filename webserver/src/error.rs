use actix_web::{error, http::StatusCode, HttpResponse, Result};
use serde::Serialize;
use std::fmt;


#[derive(Debug, Serialize)]
pub enum MyError{
  FileError(String),
  ActixError(String),
  NotFound(String),
}

#[derive(Debug, Serialize)]
pub struct MyErrorResponse {
  error_message: String,
}

impl MyError{
  fn error_response(&self) -> String{
    match self {
      MyError::FileError(msg) => {
        println!("File error occurred:{:?}", msg);
        "File error".into()
      }
      MyError::ActixError(msg) => {
        println!("Server error occurred:{:?}", msg);
        "Internal server error".into()
      }
      MyError::NotFound(msg) => {
        println!("Not found error occurred:{:?}", msg);
        msg.into()
      }
    }
  }
}

impl error::ResponseError for MyError{
  fn status_code(&self) -> StatusCode{
    match self {
      MyError::FileError(_msg) | MyError::ActixError(_msg) => StatusCode::INTERNAL_SERVER_ERROR,
      MyError::NotFound(_msg) => StatusCode::NOT_FOUND,
    }
  }

  fn error_response (&self) -> HttpResponse{
    HttpResponse::build(self.status_code()).json(MyErrorResponse{
      error_message: self.error_response(),
    })
  }
}

impl fmt::Display for MyError{
    fn fmt(&self, f: &mut fmt::Formatter) ->  Result<(), fmt::Error>{
      write!(f, "{}", self)
    }
}

impl From<actix_web::error::Error> for MyError{
  fn from(value: actix_web::error::Error) -> Self {
      MyError::ActixError(value.to_string())
  }
}

// impl From<SQLxError> for MyError {
//     fn from(value: SQLxError) -> Self {
//         MyError::FileError(value.to_string())
//     }
// }