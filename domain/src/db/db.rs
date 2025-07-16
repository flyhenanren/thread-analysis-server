use serde::{Serialize};
use sqlx::{FromRow};

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
