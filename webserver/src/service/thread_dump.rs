use std::collections::HashMap;
use rayon::vec;
use sqlx::SqlitePool;

use crate::{
    db_access::{db_file, db_thread}, error::AnalysisError, file, models::thread::{PoolThreads, StatusCount, StatusQuery, ThreadContent, ThreadDetail, ThreadStatus, ThreadsQuery}
};


/// 获取线程详情
pub async fn get_thread_detail(pool: &SqlitePool, threads_query: &ThreadsQuery) -> Result<Vec<ThreadDetail>, AnalysisError> {
    match db_thread::list_threads(pool, &threads_query.file_id, &threads_query.status, &threads_query.thread_ids).await{
        Ok(thread_info) => {
            return Ok(thread_info.iter().map(|thread| ThreadDetail::new(thread)).collect());
        },
        Err(err) => Err(AnalysisError::DBError(format!("查询线程详情错误:{}", err))),
    }
}


/// 获取活跃的堆栈信息
pub async fn count_status_by_files(
    pool: &SqlitePool,
    count_query: &StatusQuery,
) -> Result<Vec<StatusCount>, AnalysisError> {
    match db_thread::count_threads_status(pool, count_query).await{
        Ok(threads_info) => {
            let mut counts: HashMap<String, StatusCount> = HashMap::new();
            for thread in &threads_info{
                let status = counts
                .entry(thread.file_name.clone())
                .or_insert(StatusCount{
                    name: thread.file_name.clone(),
                    runnable: 0,
                    waitting: 0,
                    time_watting: 0,
                    block: 0,
                });
                match thread.thread_status{
                    1 => status.runnable += 1,
                    3 => status.waitting += 1,
                    4 => status.time_watting += 1,
                    2 => status.block += 1,
                    _ => {}
                }
            }
            let mut order = Vec::new();
            let mut seen = HashMap::new();
            for thread in threads_info{
                if seen.insert(thread.file_name.clone(), true).is_none(){
                    if let Some(status) = counts.remove(&thread.file_name){
                        order.push(status);
                    }
                }
            }
            return Ok(order);
        },
        Err(err) => Err(AnalysisError::DBError(format!("对象转换错误:{}", err))),
    }
}

/**
 * 获取指定文件中的线程信息
 */
pub async fn count_status_by_file(
    pool: &SqlitePool,
    file_id: &str,
) -> Result<Vec<PoolThreads>, AnalysisError> {
    match db_thread::list_threads(pool, file_id, &None, &None).await{
        Ok(threads_info) => {
            let mut pool_map: HashMap<String, PoolThreads> = HashMap::new();
            for thread in &threads_info{
                let prefix = extract_prefix(&thread.thread_name);
                let entry = pool_map.entry(prefix.clone()).or_insert(PoolThreads{
                    name: prefix.clone(),
                    source_name: thread.thread_name.clone(),
                    count: 0,
                    runnable: 0,
                    waitting: 0,
                    time_waitting: 0,
                    block: 0,
                    thread_ids: vec![]
                });
                entry.thread_ids.push(thread.id.clone());
                entry.count += 1;
                match thread.thread_status{
                    1 => entry.runnable += 1,
                    3 => entry.waitting += 1,
                    4 => entry.time_waitting += 1,
                    2 => entry.block += 1,
                    _ => {}
                }
            }
            return Ok(pool_map.into_values().collect());
        },
        Err(err) => Err(AnalysisError::DBError(format!("对象转换错误:{}", err))),
    }
}



fn extract_prefix(name: &str) -> String {
    let mut chars = name.chars().rev().peekable();
    let mut end_idx = name.len();
    let mut num_count = 0;
    let mut split_idx = None;

    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            num_count += 1;
        } else {
            // 检查是否是两个数值之间的分隔符
            if num_count > 0 {
                if let Some(&next_c) = chars.peek() {
                    if !next_c.is_ascii_digit() {
                        split_idx = Some(end_idx);
                        break;
                    }
                }
            }
            num_count = 0; // 不是数字，重置计数
        }
        end_idx -= c.len_utf8();
    }

    match split_idx {
        Some(idx) => name[..idx - 1].to_string(), // 去掉分隔符
        None => name.trim_end_matches(|c: char| c.is_ascii_digit()).to_string(),
    }
}

/// 获取活跃的堆栈信息
pub async fn count_status_by_thread(
    pool: &SqlitePool,
    count_query: &StatusQuery,
) -> Result<Vec<StatusCount>, AnalysisError> {
    match db_thread::count_threads_status(pool, count_query).await{
        Ok(threads_info) => {
            let mut counts: HashMap<String, StatusCount> = HashMap::new();
            for thread in &threads_info{
                let status = counts
                .entry(thread.thread_name.clone())
                .or_insert(StatusCount{
                    name: thread.thread_name.clone(),
                    runnable: 0,
                    waitting: 0,
                    time_watting: 0,
                    block: 0,
                });
                match thread.thread_status{
                    1 => status.runnable += 1,
                    3 => status.waitting += 1,
                    4 => status.time_watting += 1,
                    2 => status.block += 1,
                    _ => {}
                }
            }
            return Ok(counts.into_values().collect());
        },
        Err(err) => Err(AnalysisError::DBError(format!("对象转换错误:{}", err))),
    }
}


// 读取线程详情内容
pub async fn get_thread_content(pool: &SqlitePool, thread_id: &str) -> Result<ThreadContent, AnalysisError> {
    match db_file::get_file_by_thread(pool, &thread_id).await{
        Ok(file_info) => {
            let content = if file_info.end_line - file_info.start_line > 2  {
                file::index::read_lines_from_file(&file_info.file_path, (file_info.start_line  + 2) as usize, file_info.end_line as usize)?
            }else{
                vec![]
            };
            Ok(
                ThreadContent{
                    id: file_info.id,
                    name: file_info.thread_name,
                    status: ThreadStatus::try_from(file_info.thread_status).unwrap(),
                    content: if content.len() > 2 { content } else { vec![] },
                }
            )
        },
        Err(err) =>{
            Err(AnalysisError::DBError(format!("没有获取到数据:{}", err)))
        } ,
    }
}