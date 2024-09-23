use crate::{
    error::AnalysisError,
    files::*,
    models::thread::{StatusCount, StatusQuery, Thread, ThreadStatus},
};
use index::{FileIndex, StackIndex, ThreadsIndex};

/// 获取线程详情
pub fn get_thread_detail(path: &str, file_name: &str) -> Result<Vec<Thread>, AnalysisError> {
    // 读取索引并处理
    ThreadsIndex::read_index(path)
        .map_err(|_| AnalysisError::DBError("索引错误".to_string())) // 处理索引读取错误
        .and_then(|files| {
            // 找到对应文件，如果没有找到则返回错误
            files
                .into_iter()
                .find(|file| file.file_name == file_name)
                .ok_or_else(|| AnalysisError::NotFound("未找到匹配的文件".to_string()))
        })
        .and_then(|file| {
            // 根据文件的行号读取堆栈数据
            StackIndex::read_index_by_line(&path, file.start_line, file.end_line)
                .map_err(|err| AnalysisError::DBError(format!("读取索引失败:{}", err)))
        })
}

pub fn count_dumps_info(
    path: &str,
    count_query: &StatusQuery,
) -> Result<Vec<StatusCount>, AnalysisError> {
    file::list_dump_file(path)
        .map(|files| {
            files
                .iter()
                .filter(|file| count_query.files.contains(&file.file_name))
                .map(|file| {
                    // 获取线程详情，处理错误
                    match get_thread_detail(path, &file.file_name) {
                        Ok(threads) => {
                            // 按文件名统计状态
                            let mut status_count = StatusCount {
                                name: file.file_name.clone(), // 改为文件名
                                runnable: 0,
                                waitting: 0,
                                time_watting: 0,
                                block: 0,
                            };
                            // 累计文件中所有线程的状态
                            for thread in threads {
                                match thread.status {
                                    ThreadStatus::Runnable => status_count.runnable += 1,
                                    ThreadStatus::Blocked => status_count.block += 1,
                                    ThreadStatus::TimedWaiting => status_count.time_watting += 1,
                                    ThreadStatus::Waiting => status_count.waitting += 1,
                                    _ => {}
                                }
                            }
                            status_count // 返回每个文件的状态计数
                        }
                        Err(err) => {
                            // 记录错误并返回默认的状态计数
                            eprintln!(
                                "Error getting thread details for {}: {:?}",
                                file.file_name, err
                            );
                            StatusCount {
                                name: file.file_name.clone(),
                                runnable: 0,
                                waitting: 0,
                                time_watting: 0,
                                block: 0,
                            }
                        }
                    }
                })
                .collect::<Vec<StatusCount>>() // 收集文件状态计数
        })
        .map_err(|_| AnalysisError::ActixError("发生错误".to_string()))
}

/// 获取活跃的堆栈信息
pub fn count_threads_info(
    path: &str,
    count_query: &StatusQuery,
) -> Result<Vec<StatusCount>, AnalysisError> {
    file::list_dump_file(path)
        .map(|files| {
            files
                .iter()
                .filter(|file| count_query.files.contains(&file.file_name))
                .flat_map(|file| {
                    // 获取线程详情，处理错误
                    match get_thread_detail(path, &file.file_name) {
                        Ok(threads) => threads
                            .into_iter()
                            .filter(|thread| {
                                if let Some(exclude_list) = &count_query.exclude {
                                    !exclude_list.contains(&thread.name)
                                } else {
                                    true
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_iter(),
                        Err(err) => {
                            // 记录错误并返回空迭代器
                            eprintln!(
                                "Error getting thread details for {}: {:?}",
                                file.file_name, err
                            );
                            Vec::new().into_iter()
                        }
                    }
                })
                .map(|thread| {
                    // 创建或更新 ThreadCount
                    let mut thread_count = StatusCount {
                        name: thread.name,
                        runnable: 0,
                        waitting: 0,
                        time_watting: 0,
                        block: 0,
                    };
                    match thread.status {
                        ThreadStatus::Runnable => thread_count.runnable += 1,
                        ThreadStatus::Blocked => thread_count.block += 1,
                        ThreadStatus::TimedWaiting => thread_count.time_watting += 1,
                        ThreadStatus::Waiting => thread_count.waitting += 1,
                        _ => {}
                    }
                    thread_count
                })
                .fold(Vec::<StatusCount>::new(), |mut acc, count| {
                    // 更新或插入计数
                    let entry = acc.iter_mut().find(|e| e.name == count.name);
                    if let Some(e) = entry {
                        e.runnable += count.runnable;
                        e.block += count.block;
                        e.time_watting += count.time_watting;
                        e.waitting += count.waitting;
                    } else {
                        acc.push(count);
                    }
                    acc
                })
                .into_iter()
                .collect::<Vec<_>>() // 收集并创建 Vec
        })
        .map(|mut result| {
            // 排序并返回前十个
            result.sort_by(|a, b| {
                let a_total = match &count_query.status {
                    Some(statuses) => {
                        // 根据 `count_query.status` 来定义排序优先级
                        statuses
                            .iter()
                            .map(|status| match status {
                                ThreadStatus::Runnable => a.runnable,
                                ThreadStatus::Blocked => a.block,
                                ThreadStatus::TimedWaiting => a.time_watting,
                                ThreadStatus::Waiting => a.waitting,
                                ThreadStatus::Terminated => 0,
                                ThreadStatus::New => 0,
                                ThreadStatus::Unknown => 0,
                            })
                            .sum::<usize>()
                    }
                    None => a.runnable + a.block + a.time_watting + a.waitting, // 默认排序规则
                };

                let b_total = match &count_query.status {
                    Some(statuses) => statuses
                        .iter()
                        .map(|status| match status {
                            ThreadStatus::Runnable => b.runnable,
                            ThreadStatus::Blocked => b.block,
                            ThreadStatus::TimedWaiting => b.time_watting,
                            ThreadStatus::Waiting => b.waitting,
                            ThreadStatus::Terminated => 0,
                            ThreadStatus::New => 0,
                            ThreadStatus::Unknown => 0,
                        })
                        .sum::<usize>(),
                    None => b.runnable + b.block + b.time_watting + b.waitting,
                };

                b_total.cmp(&a_total) // 降序排序
            });
            result.truncate(match count_query.total {
                Some(total) if total > 0 => total,
                _ => 10
            }); // 保留前十个
            result
        })
        .map_err(|_| AnalysisError::ActixError("发生错误".to_string()))
}
