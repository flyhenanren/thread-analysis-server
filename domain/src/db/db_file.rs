use chrono::NaiveDateTime;
use common::model::file_info::FileInfo;
use serde::Serialize;
use sqlx::FromRow;
use common::error::DBError;
use sqlx::{SqlitePool, Transaction};
use crate::db::{db::ModelTransfer, db_thread::DBThread};



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
  fn new(file: &FileInfo, _file_id: &str, work_space: &str) -> Self {
      Self{
          id: file.id.clone(),
          workspace: work_space.into(),
          file_path: file.path.clone(),
          file_type: file.file_type.clone().try_into().unwrap(),
          exe_time: file.time.clone(),
      }
  }
}


pub async fn batch_add(pool: &SqlitePool, file_infos: Vec<DBSourceFile>) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for file_info in file_infos {
        sqlx::query(
            r#"INSERT INTO FILE_INFO (id, workspace, file_path, file_type, exe_time) VALUES (?,?,?,?,?) "#)
            .bind(file_info.id)
            .bind(file_info.workspace)
            .bind(file_info.file_path)
            .bind(file_info.file_type)
            .bind(file_info.exe_time)
            .execute(pool)
            .await?;    
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<DBSourceFile>, DBError> {
    let work_space = sqlx::query_as::<_, DBSourceFile>("SELECT * FROM FILE_INFO WHERE WORKSPACE = ?")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}

pub async fn delete_all(pool: &SqlitePool) -> Result<(), DBError> {
    sqlx::query("DELETE FROM FILE_INFO")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_file_by_thread(pool: &SqlitePool, id: &str) -> Result<DBThread, DBError> {
    let file_info = sqlx::query_as::<_, DBThread>(r#"SELECT T.ID, F.FILE_PATH, T.THREAD_NAME, T.THREAD_STATUS, T.START_LINE, T.END_LINE FROM FILE_INFO F 
                                LEFT JOIN THREAD_INFO T 
                                ON F.ID = T.FILE_ID 
                                WHERE T.ID = ?"#)
    .bind(id)
    .fetch_one(pool)
    .await?;
Ok(file_info)
}