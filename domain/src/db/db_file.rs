use chrono::NaiveDateTime;
use common::model::file_info::FileInfo;
use serde::Serialize;
use sqlx::FromRow;

use crate::db::db::ModelTransfer;



#[derive(Serialize, Debug, Clone, FromRow)]
pub struct DBSourceFile {
    #[sqlx(rename = "ID")]
    pub id: String,
    #[sqlx(rename = "WORKSPACE")]
    pub workspace: String,
    #[sqlx(rename = "FILE_PATH")]
    pub file_path: String,
    #[sqlx(rename = "FILE_TYPE")]
    pub file_type: i8,
    #[sqlx(rename = "EXE_TIME")]
    pub exe_time: Option<NaiveDateTime>,
}


impl ModelTransfer<FileInfo, DBSourceFile> for DBSourceFile{
  fn new(file: &FileInfo, file_id: &str, work_space: &str) -> Self {
      Self{
          id: file.id.clone(),
          workspace: work_space.into(),
          file_path: file.path.clone(),
          file_type: file.file_type.clone().try_into().unwrap(),
          exe_time: file.time.clone(),
      }
  }
}