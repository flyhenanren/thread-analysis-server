use chrono::NaiveDateTime;
use common::string_utils::rand_id;
use serde::Serialize;
use sqlx::FromRow;

use crate::model::memory::MemoryValue;

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct DBMemory {
    pub id: String,
    pub work_space: String,
    pub file_id: String,
    pub s0c: f64,
    pub s0u: f64,
    pub s1c: f64,
    pub s1u: f64,
    pub ec: f64,
    pub eu: f64,
    pub oc: f64,
    pub ou: f64,
    pub mc: f64,
    pub mu: f64,
    pub ccsc: f64,
    pub ccsu: f64,
    pub ygc: f64,
    pub ygct: f64,
    pub fgc: f64,
    pub fgct: f64,
    pub cgc: f64,
    pub cgct: f64,
    pub gct: f64,
    pub exe_time: Option<NaiveDateTime>,
}

impl DBMemory {
  pub fn new(memory: &MemoryValue, work_space: &str) -> Self{
      Self {
          id: rand_id(),
          work_space: work_space.into(),
          file_id: memory.file_id.clone(),
          s0c: memory.value[0],
          s0u: memory.value[1],
          s1c: memory.value[2],
          s1u: memory.value[3],
          ec: memory.value[4],
          eu: memory.value[5],
          oc: memory.value[6],
          ou: memory.value[7],
          mc: memory.value[8],
          mu: memory.value[9],
          ccsc: memory.value[10],
          ccsu: memory.value[11],
          ygc: memory.value[12],
          ygct: memory.value[13],
          fgc: memory.value[14],
          fgct: memory.value[15],
          cgc: memory.value[16],
          cgct: memory.value[17],
          gct: memory.value[18],
          exe_time: memory.time,
      }
  }
}