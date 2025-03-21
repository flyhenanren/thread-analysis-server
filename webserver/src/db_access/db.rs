use std::env;

use chrono::{NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};

use crate::{common::utils, models::{cpu::Cpu, file_info::FileInfo, memory::MemoryValue, thread::{Thread}}};

pub async fn establish_connection() -> Pool<Sqlite> {
    let database_url = env::var("DATABASE_URL").expect("无法获取数据库链接");
    SqlitePool::connect(&database_url)
        .await
        .expect("无法连接数据库")
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct StatusInfo {
    #[sqlx(rename = "FILE_PATH")]
    pub file_name: String,
    #[sqlx(rename = "THREAD_NAME")]
    pub thread_name: String,
    #[sqlx(rename = "THREAD_STATUS")]
    pub thread_status: i8,
}

pub trait ModelTransfer<T, U> {
    fn new(file: &T, file_id: &str, work_space: &str) -> U;
}
