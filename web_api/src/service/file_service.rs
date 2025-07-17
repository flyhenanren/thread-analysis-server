use std::{collections::HashMap};

use common::{error::AnalysisError};
use db::{db_access::{db_cpu, db_file, db_memeory, db_stack, db_thread, db_worksapce}, workspace::DBFileWorkSpace};
use domain::{db::{db_file::DBSourceFile, db_thread::DBThreadInfo}, model::thread::{StackDumpInfo, ThreadStatus}};
use itertools::Itertools;
use sqlx::{SqlitePool};

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
                    #[allow(deprecated)]
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

    #[test]
    fn test_zip_type() {
        let path = Path::new("D:\\dump\\b.txt");
        let _ = common::file_utils::get_file_type(path.as_str());
    }
}
