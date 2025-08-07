
use std::collections::HashMap;

use common::{error::AnalysisError};
use db::db_access::{db_cpu, db_memeory, db_thread};
use domain::{db::{db::ModelTransfer, db_cpu::DBCpu, db_memory::DBMemory, db_thread::DBThreadInfo}, model::{cpu::Cpu, memory::MemoryValue, thread::Thread}};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::SqlitePool;


pub trait Writer{
   async fn write_threads(pool: &SqlitePool, workspace_id: &str, threads_map: &HashMap<String, Vec<Thread>>) -> Result<(),AnalysisError>;
   async fn write_cpu(pool: &SqlitePool, workspace_id: &str, cpus: &Vec<Cpu>) -> Result<(), AnalysisError>;
   async fn write_memory(pool: &SqlitePool, workspace_id: &str, memories: &Vec<MemoryValue>) -> Result<(),AnalysisError>;
}


pub struct LocalWriter;
struct DBWriter;

impl Writer for LocalWriter{
    async fn write_threads(pool: &SqlitePool, workspace_id: &str, threads_map: &HashMap<String, Vec<Thread>>) -> Result<(),AnalysisError> {
        DBWriter::write_threads(pool, workspace_id, threads_map).await?;
        Ok(())
      
    }

    async fn write_cpu(pool: &SqlitePool, workspace_id: &str, cpus: &Vec<Cpu>) -> Result<(), AnalysisError> {
        DBWriter::write_cpu(pool, workspace_id, cpus).await?;
        Ok(())
    }

    async fn write_memory(pool: &SqlitePool, workspace_id: &str, memories: &Vec<MemoryValue>) -> Result<(),AnalysisError> {
        DBWriter::write_memory(pool, workspace_id, memories).await?;
        Ok(())
    }
} 

impl Writer for DBWriter {

    async fn write_threads(pool: &SqlitePool, _workspace_id: &str, threads_map: &HashMap<String, Vec<Thread>>) -> Result<(), AnalysisError> {
      let db_threads = threads_map
        .into_par_iter()
        .flat_map(|(key, value)| {
            value.into_par_iter().map(move |thread| {
                DBThreadInfo::new(&thread, &key)
            })
        })
        .collect::<Vec<DBThreadInfo>>();
        db_thread::batch_add(pool, db_threads).await?;
      Ok(())
    }

    async fn write_cpu(pool: &SqlitePool,workspace_id: &str, cpus: &Vec<Cpu>) -> Result<(), AnalysisError> {
           db_cpu::batch_add(
        pool,
        cpus
            .into_iter()
            .map(|info| DBCpu::new(&info, &info.file_id, &workspace_id))
            .collect()
      )   
      .await
        .map_err(|e| AnalysisError::DBError(e.to_string()))?;
        Ok(())
    }

    async fn write_memory(pool: &SqlitePool,workspace_id: &str, memory_info: &Vec<MemoryValue>) -> Result<(), AnalysisError> {
        db_memeory::batch_add(
        pool,
        &memory_info
            .into_iter()
            .map(|mem| DBMemory::new(&mem,workspace_id))
            .collect(),
        )
      .await
      .map_err(|e| AnalysisError::DBError(e.to_string()))?;
        Ok(())
    }
    
}