use crate::{
    error::AnalysisError,
    models::thread::{StatusCount, StatusQuery, Thread},
};


/// 获取线程详情
pub fn get_thread_detail(path: &str, file_name: &str) -> Result<Vec<Thread>, AnalysisError> {
    Ok(vec![])
}

pub fn count_dumps_info(
    path: &str,
    count_query: &StatusQuery,
) -> Result<Vec<StatusCount>, AnalysisError> {
    Ok(vec![])
}

/// 获取活跃的堆栈信息
pub fn count_threads_info(
    path: &str,
    count_query: &StatusQuery,
) -> Result<Vec<StatusCount>, AnalysisError> {
    Ok(vec![])
}
