use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{NaiveDateTime, Utc};
use crate::common::utils;

#[derive(Serialize,Deserialize, Debug, Clone, FromRow)]
pub struct FileWorkSpace {
    #[sqlx(rename = "ID")]
    pub id: String,
    #[sqlx(rename = "FILE_PATH")]
    pub file_path: String,
    #[sqlx(rename = "CREATE_TIME")]
    pub create_time: NaiveDateTime,
    #[sqlx(rename = "UPDATE_TIME")]
    pub update_time: NaiveDateTime,
}

impl FileWorkSpace{
  pub fn new(path: &str) -> Self{
      FileWorkSpace {
          id: utils::rand_id(),
          file_path: path.into(),
          create_time: Utc::now().naive_utc(),
          update_time: Utc::now().naive_utc()
      }
  }
}
