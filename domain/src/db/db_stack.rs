
use common::string_utils::rand_id;
use serde::Serialize;
use serde_json::to_string;
use sqlx::FromRow;

use crate::model::thread::Thread;


#[derive(Serialize, Debug, Clone, FromRow)]
pub struct DBStack {
    pub id: String,
    pub work_space: String,
    pub thread_id: String,
    pub class_name: String,
    pub method_name: Option<String>,
    pub method_lin: Option<u32>,
    pub stack_status: String,
}

impl DBStack{
  pub fn new(thread: &Thread, thread_id: &str, work_space: &str) -> Vec<DBStack> {
      thread.frames
      .clone()
      .into_iter()
      .map(|frame| {
          DBStack{
              id: rand_id(),
              work_space: work_space.into(),
              thread_id: thread_id.into(),
              class_name: frame.class_name,
              method_name: frame.method_name,
              method_lin: frame.line_number,
              stack_status: to_string(&frame.frame).unwrap(),
          }
      })
      .collect()
  }
}