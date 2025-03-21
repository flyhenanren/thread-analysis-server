use chrono::NaiveTime;
use serde::Serialize;
use sqlx::FromRow;

use crate::{common::utils, db_access::db::ModelTransfer, models::cpu::Cpu};

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct CpuInfo {
    pub id: String,
    pub workspace: String,
    pub exe_time: NaiveTime,
    pub us: f64,
    pub sy: f64,
    pub ids: f64,
    pub tasks: u32,
    pub running: u32,
    pub sleeping: u32,
    pub mem_total: f64,
    pub mem_free: f64,
    pub mem_used: f64,
}


#[derive(Serialize, Debug, Clone, FromRow)]
pub struct CpuCountInfo {
    pub exe_time: NaiveTime,
    pub us: f64,
    pub sy: f64,
    pub ids: f64
}


impl ModelTransfer<Cpu, CpuInfo> for CpuInfo {
  fn new(file: &Cpu, _file_id: &str, work_space: &str) -> CpuInfo {
      CpuInfo {
          id: utils::rand_id(),
          workspace: work_space.into(),
          exe_time: file.exe_time,
          us: file.us,
          sy: file.sy,
          ids: file.ids,
          tasks: file.tasks,
          running: file.running,
          sleeping: file.sleeping,
          mem_total: file.mem_total,
          mem_free: file.mem_free,
          mem_used: file.mem_used,
      }
  }
}