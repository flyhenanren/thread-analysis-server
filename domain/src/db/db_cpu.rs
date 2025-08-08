use chrono::NaiveTime;
use common::string_utils::rand_id;
use serde::Serialize;
use sqlx::FromRow;
use common::error::DBError;
use sqlx::{SqlitePool, Transaction};

use crate::{db::db::ModelTransfer, model::cpu::Cpu};


#[derive(Serialize, Debug, Clone, FromRow)]
pub struct DBCpu {
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
pub struct DBCpuCount {
    pub exe_time: NaiveTime,
    pub us: f64,
    pub sy: f64,
    pub ids: f64
}


impl ModelTransfer<Cpu, DBCpu> for DBCpu {
  fn new(file: &Cpu, _file_id: &str, work_space: &str) -> DBCpu {
      DBCpu {
          id: rand_id(),
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


pub async fn batch_add(pool: &SqlitePool, cpu_infos: Vec<DBCpu>) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for info in cpu_infos {
        sqlx::query(
            r#"INSERT INTO CPU_INFO (id,workspace, exe_time, us, sy, ids, tasks, running, sleeping, mem_total, mem_free, mem_used)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?) "#)
             .bind(info.id)
             .bind(info.workspace)
            .bind(info.exe_time)
            .bind(info.us)
            .bind(info.sy)
            .bind(info.ids)
            .bind(info.tasks)
            .bind(info.running)
            .bind(info.sleeping)
            .bind(info.mem_total)
            .bind(info.mem_free)
            .bind(info.mem_used)
            .execute(pool)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<DBCpu>, DBError> {
    let work_space = sqlx::query_as::<_, DBCpu>("SELECT * FROM CPU_INFO WHERE WORKSPACE = ? ")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}


pub async fn delete_all(pool: &SqlitePool) -> Result<(), DBError> {
    sqlx::query("DELETE FROM CPU_INFO")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_info(pool: &SqlitePool, work_space: &str) -> Result<Vec<DBCpuCount>, DBError> {
    let work_space = sqlx::query_as::<_, DBCpuCount>("SELECT exe_time, us, sy, ids FROM CPU_INFO WHERE WORKSPACE = ? ")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}