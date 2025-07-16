use std::{collections::HashMap, path::Path};

use common::{error::AnalysisError, file_utils, model::file_info::FileInfo};
use db::{db_access::{db_cpu, db_file, db_memeory, db_stack, db_thread, db_worksapce}, workspace::DBFileWorkSpace};
use domain::{db::{db::ModelTransfer, db_cpu::DBCpu, db_file::DBSourceFile, db_memory::DBMemory, db_stack::DBStack, db_thread::DBThreadInfo}, model::thread::{StackDumpInfo, ThreadStatus}};
use itertools::Itertools;
use rayon::prelude::*;
use sqlx::{SqlitePool};
use task::async_task::ExecuteContext;

use crate::file::parse::{CpuParser, MemoryParser, ParseFile, ThreadParser};


/// 分析文件夹或者文件中的线程信息,并生成到数据库中
pub async fn analysis(pool: &SqlitePool, path: &str, context: &ExecuteContext) -> Result<String, AnalysisError> {
    let file_type: u8 =
        file_utils::get_file_type(path).map_err(|e| AnalysisError::ParseError(e.to_string()))?;
    let source_path = Path::new(path);
    // 读取或生成文件索引
    let work_space;
    context.update_progress(1.0, Some("读取文件".to_string())).await;
    let files: Vec<FileInfo> = match file_type {
        1 => {
            work_space = DBFileWorkSpace::new(&path);
            db_worksapce::add(pool, &work_space).await?;
            file_utils::extract_file(source_path, &work_space.id)
                .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
        }
        _ => {
            work_space = DBFileWorkSpace::new(path);
            db_worksapce::add(pool, &work_space).await?;
            context.update_progress(2.0, Some("解压".to_string())).await;
            file_utils::unzip_and_extract_file(source_path, &work_space.id)
                .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
        }
    };
    context.update_progress(10.0, Some("解析CPU文件".to_string())).await;
    let cpu_info = CpuParser::parse(path, &files)?;
    context.update_progress(15.0, Some("解析线程文件".to_string())).await;
    let threads_map = ThreadParser::parse(path, &files)?;
    context.update_progress(25.0, Some("解析内存文件".to_string())).await;
    let memory_info = MemoryParser::parse(path, &files)?;
    context.update_progress(30.0, Some("写入文件信息".to_string())).await;
    db_file::batch_add(
        pool,
        files
            .into_iter()
            .map(|f| DBSourceFile::new(&f, "", &work_space.id))
            .collect(),
    )
    .await?;
    context.update_progress(35.0, Some("写入CPU信息".to_string())).await;
    db_cpu::batch_add(
        pool,
        cpu_info
            .into_iter()
            .map(|info| DBCpu::new(&info, &info.file_id, &work_space.id))
            .collect(),
        path,
    )
    .await?;
    context.update_progress(40.0, Some("解析线程信息".to_string())).await;
    let work_space_id = &work_space.id;
    let threads_count = threads_map
        .into_par_iter()
        .flat_map(|(key, value)| {
            value.into_par_iter().map(move |thread| {
                let thread_info = DBThreadInfo::new(&thread, &key);
                // 先将 ThreadStack 创建并收集
                let stack = DBStack::new(&thread, &thread_info.id, &work_space_id);
                // 收集完所有的 stack_info，返回值包含 thread_info 和 stack
                (thread_info, stack)
            })
        })
        .collect::<Vec<(DBThreadInfo, Vec<DBStack>)>>();
    
    // 分离 ThreadInfo 和 ThreadStack 的集合
    let mut stack_info = Vec::new();
    let mut threads_info = Vec::new();
    for (thread_info, stack) in threads_count {
        stack_info.extend(stack); // 将所有的 stack 扩展到 stack_info 中
        threads_info.push(thread_info);
    }
    context.update_progress(50.0, Some("写入线程信息".to_string())).await;
    db_thread::batch_add(pool, threads_info, &work_space.id).await?;
    context.update_progress(65.0, Some("写入堆栈信息".to_string())).await;
    db_stack::batch_add(pool, &stack_info).await?;
    context.update_progress(95.0, Some("写入内存信息".to_string())).await;
    db_memeory::batch_add(
        pool,
        &memory_info
            .into_iter()
            .map(|mem| DBMemory::new(&mem, &work_space.id))
            .collect(),
    )
    .await?;
    context.update_progress(100.0, Some("解析完成".to_string())).await;
    Ok(work_space.id)
}

/// 获取所有线程文件信息
pub async fn list_dump_file(pool: &SqlitePool, work_space_id: &str) -> Result<Vec<StackDumpInfo>, AnalysisError> {
    return match db_worksapce::get(pool, work_space_id).await? {
        Some(work_space) => {
            let fils: Vec<DBSourceFile> = db_file::list(pool, &work_space.id).await?;
            let infos: HashMap<String, Vec<DBThreadInfo>> = db_thread::list_by_work_space(pool, &work_space.id).await?
                            .into_iter()
                            .into_group_map_by(|info| info.file_id.clone());
            let mut result: Vec<StackDumpInfo> = Vec::new();
            for file in fils {
                if let Some(thread_status_list ) = infos.get(&file.id) {
                    let mut dump_info = StackDumpInfo {
                        file_id: file.id.clone(),
                        file_name: file.file_path.clone(),
                        time: file.exe_time.unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
                        run_threads:0,
                        block_threads: 0,
                        threads:0
                    };
                    for status_info in thread_status_list  {
                        dump_info.threads += 1;
                        if let Ok(status) = ThreadStatus::try_from(status_info.thread_status) {
                            match status {
                                ThreadStatus::Runnable => dump_info.run_threads += 1,
                                ThreadStatus::Blocked => dump_info.block_threads += 1,
                                _ => {}
                            }
                        }
                    }
                    result.push(dump_info);
                }
            }
            return Ok(result);
        },
        None => Ok(vec![])
    }
}

pub async fn exist_work_space(pool: &SqlitePool, path: &str) -> Result<bool, AnalysisError> {
    Ok(db_worksapce::get_by_path(pool, path).await?.is_some())
}

pub async fn list_work_space(pool: &SqlitePool) -> Result<Vec<DBFileWorkSpace>, AnalysisError> {
    let mut work_space = db_worksapce::list(pool).await?;
    work_space.sort_by(|a, b| b.create_time.cmp(&a.create_time));
    Ok(work_space)
}

pub async fn clean_work_space(pool: &SqlitePool) -> Result<bool, AnalysisError>{
    db_worksapce::delete_all(pool).await.unwrap_or_else(|err| log::error!("删除工作空间出错：{:?}", err));
    db_file::delete_all(pool).await.unwrap_or_else(|err| log::error!("删除文件信息出错：{:?}", err));
    db_memeory::delete_all(pool).await.unwrap_or_else(|err| log::error!("删除内存信息出错：{:?}", err));
    db_cpu::delete_all(pool).await.unwrap_or_else(|err| log::error!("删除CPU信息出错：{:?}", err));
    db_thread::delete_all(pool).await.unwrap_or_else(|err| log::error!("删除线程信息出错：{:?}", err));
    db_stack::delete_all(pool).await.unwrap_or_else(|err| log::error!("删除堆栈信息出错：{:?}", err));
    Ok(true)
}

#[cfg(test)]
mod tests {

    use actix_web::dev::Path;
    use db::connection::establish_connection;
    use task::async_task::ExecuteContext;

    use super::analysis;
    use dotenv::dotenv;

    #[test]
    fn test_zip_type() {
        let path = Path::new("D:\\dump\\b.txt");
        let _ = common::file_utils::get_file_type(path.as_str());
    }

    #[actix_rt::test]
    async fn test_unzip() {
        dotenv().ok();
        let path = Path::new("D:\\dump\\20240726XNJK[非涉密].zip");
        let pool: sqlx::Pool<sqlx::Sqlite> = establish_connection().await;
        analysis(&pool, path.as_str(), &ExecuteContext{ pool: todo!(), channel: todo!(), param: todo!() });
    }

    #[actix_rt::test]
    async fn test_walk_dir_all() {
        let path = Path::new("D:\\dump\\20240726");
        dotenv().ok();
        let pool: sqlx::Pool<sqlx::Sqlite> = establish_connection().await;
        analysis(&pool, path.as_str(),&ExecuteContext{ pool: todo!(), channel: todo!(), param: todo!() });
    }
    #[actix_rt::test]
    async fn test_walk_dir() {
        let path = Path::new("D:\\dump\\20240809_1");
        dotenv().ok();
        let pool: sqlx::Pool<sqlx::Sqlite> = establish_connection().await;
        analysis(&pool, path.as_str(),&ExecuteContext{ pool: todo!(), channel: todo!(), param: todo!() });
    }
}
