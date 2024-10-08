use std::path::Path;

use chrono::Utc;
use rayon::prelude::*;
use sqlx::{SqlitePool};

use crate::{
    common::file_utils::{self},
    db_access::{db_cpu, db_file, db_memeory, db_stack, db_thread, db_worksapce},
    error::AnalysisError,
    file::parse::{CpuParser, MemoryParser, ParseFile, ThreadParser},
    models::{file_info::FileInfo, stack, thread::StackDumpInfo},
    CpuInfo, FileWorkSpace, MemoryInfo, ModelTransfer, SourceFileInfo, ThreadInfo, ThreadStack,
};

/// 分析文件夹或者文件中的线程信息,并生成到数据库中
pub async fn analysis(pool: &SqlitePool, path: &str) -> Result<(), AnalysisError> {
    let start = Utc::now().timestamp_millis();
    let file_type: u8 =
        file_utils::get_file_type(path).map_err(|e| AnalysisError::ParseError(e.to_string()))?;
    let source_path = Path::new(path);
    // 读取或生成文件索引
    let work_space;
    let files: Vec<FileInfo> = match file_type {
        1 => {
            work_space = FileWorkSpace::new(&path);
            db_worksapce::add(pool, &work_space).await?;
            file_utils::extract_file(source_path, &work_space.id)
                .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
        }
        _ => {
            let unzip_path: &Path = source_path.parent().expect("获取压缩包的上级路径错误");
            work_space = FileWorkSpace::new(unzip_path.to_str().unwrap());
            db_worksapce::add(pool, &work_space).await?;
            file_utils::unzip_and_extract_file(source_path, &work_space.id)
                .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
        }
    };
    let finish_unzip = Utc::now().timestamp_millis();
    println!("finish_unzip:{}", finish_unzip - start);

    let cpu_info = CpuParser::parse(path, &files)?;
    let finish_parse_cpu = Utc::now().timestamp_millis();
    println!("finish_parse_cpu:{}", finish_parse_cpu - finish_unzip);

    let threads_map = ThreadParser::parse(path, &files)?;
    let finish_parse_thread = Utc::now().timestamp_millis();
    println!(
        "finish_parse_thread:{}",
        finish_parse_thread - finish_parse_cpu
    );

    let memory_info = MemoryParser::parse(path, &files)?;
    let finish_parse_memory = Utc::now().timestamp_millis();
    println!(
        "finish_parse_memory:{}",
        finish_parse_memory - finish_parse_thread
    );

    db_file::batch_add(
        pool,
        files
            .into_iter()
            .map(|f| SourceFileInfo::new(&f, "", &work_space.id))
            .collect(),
    )
    .await?;
    let finish_db_file = Utc::now().timestamp_millis();
    println!("finish_db_file:{}", finish_db_file - finish_parse_memory);

    db_cpu::batch_add(
        pool,
        cpu_info
            .into_iter()
            .map(|info| CpuInfo::new(&info, &info.file_id, &work_space.id))
            .collect(),
        path,
    )
    .await?;
    let finish_db_cpu = Utc::now().timestamp_millis();
    println!("finish_db_cpu:{}", finish_db_cpu - finish_db_file);

    let work_space_id = &work_space.id;
    let threads_count = threads_map
        .into_par_iter()
        .flat_map(|(key, value)| {
            value.into_par_iter().map(move |thread| {
                // 先将 ThreadStack 创建并收集
                let stack = ThreadStack::new(&thread, &key, &work_space_id);
                // 收集完所有的 stack_info，返回值包含 thread_info 和 stack
                (ThreadInfo::new(&thread, &key), stack)
            })
        })
        .collect::<Vec<(ThreadInfo, Vec<ThreadStack>)>>();

    let finish_count_thread = Utc::now().timestamp_millis();
    println!(
        "finish_count_thread:{}",
        finish_count_thread - finish_db_cpu
    );

    // 分离 ThreadInfo 和 ThreadStack 的集合
    let mut stack_info = Vec::new();
    let mut threads_info = Vec::new();
    for (thread_info, stack) in threads_count {
        stack_info.extend(stack); // 将所有的 stack 扩展到 stack_info 中
        threads_info.push(thread_info);
    }

    db_thread::batch_add(pool, threads_info, &work_space.id).await?;
    let finish_db_thread = Utc::now().timestamp_millis();
    println!(
        "finish_db_thread:{}",
        finish_db_thread - finish_count_thread
    );

    db_stack::batch_add(pool, &stack_info).await?;
    let finish_db_stack = Utc::now().timestamp_millis();
    println!("finish_db_stack:{}", finish_db_stack - finish_db_thread);

    db_memeory::batch_add(
        pool,
        &memory_info
            .into_iter()
            .map(|mem| MemoryInfo::new(&mem, &work_space.id))
            .collect(),
    )
    .await?;
    let finish_db_memory = Utc::now().timestamp_millis();
    println!("finish_db_memory:{}", finish_db_memory - finish_db_stack);
    Ok(())
}

/// 获取所有线程文件信息
pub fn list_dump_file(pool: &SqlitePool, path: &str) -> Result<Vec<StackDumpInfo>, AnalysisError> {
    Ok(vec![])
}

pub async fn exist_work_space(pool: &SqlitePool, path: &str) -> Result<bool, AnalysisError> {
    Ok(db_worksapce::get_by_path(pool, path).await?.is_some())
}

#[cfg(test)]
mod tests {

    use actix_web::dev::Path;

    use super::analysis;
    use crate::{common, db_access::db};
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
        let pool: sqlx::Pool<sqlx::Sqlite> = db::establish_connection().await;
        analysis(&pool, path.as_str());
    }

    #[actix_rt::test]
    async fn test_walk_dir_all() {
        let path = Path::new("D:\\dump\\20240726");
        dotenv().ok();
        let pool: sqlx::Pool<sqlx::Sqlite> = db::establish_connection().await;
        analysis(&pool, path.as_str());
    }
    #[actix_rt::test]
    async fn test_walk_dir() {
        let path = Path::new("D:\\dump\\20240809_1");
        dotenv().ok();
        let pool: sqlx::Pool<sqlx::Sqlite> = db::establish_connection().await;
        analysis(&pool, path.as_str());
    }
}
