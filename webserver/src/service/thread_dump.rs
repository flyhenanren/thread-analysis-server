use std::collections::HashMap;

use actix_web::cookie::time::ext;
use regex::Regex;
use sqlx::SqlitePool;

use crate::{
    db_access::db_thread, error::AnalysisError, handlers::thread, models::thread::{PoolThreads, StatusCount, StatusQuery, Thread}
};


/// 获取线程详情
pub fn get_thread_detail(path: &str, file_name: &str) -> Result<Vec<Thread>, AnalysisError> {
    Ok(vec![])
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
    match db_thread::list_threads_by_file(pool, file_id).await{
        Ok(threads_info) => {
            let mut pool_map: HashMap<String, PoolThreads> = HashMap::new();
            let total_count = threads_info.len() as f64; // 转换为 f64 进行计算
            for thread in &threads_info{
                let prefix = extract_prefix(&thread.thread_name);
                let entry = pool_map.entry(prefix.clone()).or_insert(PoolThreads{
                    name: prefix.clone(),
                    source_name: thread.thread_name.clone(),
                    count: 0,
                    runnable: 0,
                    waitting: 0,
                    time_waitting: 0,
                    block: 0
                });
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
