
use common::string_utils::rand_id;
use serde::Serialize;
use serde_json::to_string;
use sqlx::FromRow;

use crate::model::thread::Thread;


#[derive(Serialize, Debug, Clone, FromRow)]
pub struct DBThreadInfo {
    #[sqlx(rename = "ID")]
    pub id: String,
    #[sqlx(rename = "FILE_ID")]
    pub file_id: String,
    #[sqlx(rename = "THREAD_ID")]
    pub thread_id: Option<String>,
    #[sqlx(rename = "THREAD_NAME")]
    pub thread_name: String,
    #[sqlx(rename = "DAEMON")]
    pub daemon: bool,
    #[sqlx(rename = "PRIO")]
    pub prio: Option<u16>,
    #[sqlx(rename = "OS_PRIO")]
    pub os_prio: u32,
    #[sqlx(rename = "TID")]
    pub tid: String,
    #[sqlx(rename = "NID")]
    pub nid: String,
    #[sqlx(rename = "ADDRESS")]
    pub address: Option<String>,
    #[sqlx(rename = "THREAD_STATUS")]
    pub thread_status: i8,
    #[sqlx(rename = "START_LINE")]
    pub start_line: i64,
    #[sqlx(rename = "END_LINE")]
    pub end_line: i64,
    #[sqlx(rename = "TOP_METHOD")]
    pub method_name: String,
    #[sqlx(rename = "STACK_INFO")]
    pub stack_info: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DBThread {
    #[sqlx(rename = "ID")]
    pub id: String,
    #[sqlx(rename = "FILE_PATH")]
    pub file_path: String,
    #[sqlx(rename = "THREAD_NAME")]
    pub thread_name: String,
    #[sqlx(rename = "THREAD_STATUS")]
    pub thread_status: i8,
    #[sqlx(rename = "START_LINE")]
    pub start_line: i64,
    #[sqlx(rename = "END_LINE")]
    pub end_line: i64
}


impl DBThreadInfo{
  pub fn new(thread: &Thread, file_id: &str) -> Self{
      DBThreadInfo {
          id: rand_id(),
          file_id: file_id.into(),
          thread_id: thread.id.clone(),
          thread_name: thread.name.clone(),
          daemon: thread.daemon,
          prio: thread.prio.clone(),
          os_prio: thread.os_prio,
          tid: thread.tid.clone(),
          nid: thread.nid.clone(),
          address: thread.address.clone(),
          thread_status: thread.status.clone().into(),
          start_line: thread.start,
          end_line: thread.end,
          method_name: thread.frames.first().and_then(|frame| frame.method_name.clone()).unwrap_or_default(),
          stack_info: thread.frames.iter().map(|frame| to_string(&frame.frame).unwrap()).collect::<Vec<String>>().join("\n"),
      }
  }
}