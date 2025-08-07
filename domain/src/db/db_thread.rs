use crate::model::thread::{CallFrame, Thread, ThreadStatus};
use serde_json::from_str;

use common::string_utils::rand_id;
use serde::Serialize;
use serde_json::to_string;
use sqlx::FromRow;


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
    pub top_method: String,
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
          top_method: thread.frames.first().and_then(|frame| frame.signature.clone()).unwrap_or_default(),
          stack_info: thread.frames.iter().map(|frame| to_string(&frame.frame).unwrap()).collect::<Vec<String>>().join("\n"),
      }
  }
  pub fn to_thread(&self) -> Thread {
        let frames: Vec<CallFrame> = self.stack_info
            .lines()
            .filter_map(|line| from_str(line).ok())
            .collect();
        Thread {
            id: self.thread_id.clone(),
            name: self.thread_name.clone(),
            daemon: self.daemon,
            prio: self.prio,
            os_prio: self.os_prio,
            tid: self.tid.clone(),
            nid: self.nid.clone(),
            status: ThreadStatus::try_from(self.thread_status).unwrap_or(ThreadStatus::Unknown),
            address: self.address.clone(),
            frames,
            start: self.start_line,
            end: self.end_line,
        }
    }
}
