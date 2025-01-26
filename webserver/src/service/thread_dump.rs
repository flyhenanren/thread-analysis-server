use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::{
    db_access::db_thread, error::AnalysisError, models::thread::{StatusCount, StatusQuery, Thread}
};


/// 获取线程详情
pub fn get_thread_detail(path: &str, file_name: &str) -> Result<Vec<Thread>, AnalysisError> {
    Ok(vec![])
}


/// 获取活跃的堆栈信息
pub async fn count_status_by_file(
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
