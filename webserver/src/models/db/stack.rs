
use serde::Serialize;
use serde_json::to_string;
use sqlx::FromRow;

use crate::{common::utils, models::thread::{Thread}};

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct ThreadStack {
    pub id: String,
    pub work_space: String,
    pub thread_id: String,
    pub class_name: String,
    pub method_name: Option<String>,
    pub method_lin: Option<u32>,
    pub stack_status: String,
}

impl ThreadStack{
  pub fn new(thread: &Thread, thread_id: &str, work_space: &str) -> Vec<ThreadStack> {
      thread.frames
      .clone()
      .into_iter()
      .map(|frame| {
          ThreadStack{
              id: utils::rand_id(),
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