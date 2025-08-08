use chrono::NaiveDateTime;
use common::string_utils::rand_id;
use serde::Serialize;
use sqlx::FromRow;
use common::error::DBError;
use sqlx::{SqlitePool, Transaction};

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


pub async fn batch_add(pool: &SqlitePool, mem_infos: &Vec<DBMemory>) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for mem_info in mem_infos {
        sqlx::query(
            r#"INSERT INTO MEMORY_INFO (ID, WORK_SPACE, FILE_ID, S0C, S0U, S1C, S1U, EC, EU, OC, OU, MC, MU, CCSC, CCSU, YGC, YGCT, FGC, FGCT, CGC, CGCT, GCT, EXE_TIME)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#)
             .bind(mem_info.id.clone())
             .bind(mem_info.work_space.clone())
            .bind(mem_info.file_id.clone())
            .bind(mem_info.s0c)
            .bind(mem_info.s0u)
            .bind(mem_info.s1c)
            .bind(mem_info.s1u)
            .bind(mem_info.ec)
            .bind(mem_info.eu)
            .bind(mem_info.oc)
            .bind(mem_info.ou)
            .bind(mem_info.mc)
            .bind(mem_info.mu)
            .bind(mem_info.ccsc)
            .bind(mem_info.ccsu)
            .bind(mem_info.ygc)
            .bind(mem_info.ygct)
            .bind(mem_info.fgc)
            .bind(mem_info.fgct)
            .bind(mem_info.cgc)
            .bind(mem_info.cgct)
            .bind(mem_info.gct)
            .bind(mem_info.exe_time)
            .execute(pool)
            .await?;    
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<DBMemory>, DBError> {
    let work_space = sqlx::query_as::<_, DBMemory>("SELECT * FROM MEMORY_INFO where WORKSPACE = ?")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}

pub async fn delete_all(pool: &SqlitePool) -> Result<(), DBError> {
    sqlx::query("DELETE FROM MEMORY_INFO")
        .execute(pool)
        .await?;
    Ok(())
}