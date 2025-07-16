use chrono::{Duration, NaiveDateTime, Utc};
use common::error::AnalysisError;
use common::model::file_info::{FileInfo, FileType};
use domain::model::cpu::Cpu;
use domain::model::memory::{self, MemoryValue};
use domain::model::thread::Thread;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::iter::Map;
use std::{fs, vec};
use std::{fs::File, path::Path};

pub trait ParseFile<T, U> {
    fn parse(path: &str, files: &Vec<U>) -> Result<T, AnalysisError>;
}

pub struct ThreadParser;
pub struct CpuParser;
pub struct MemoryParser;

impl ParseFile<Vec<Cpu>, FileInfo> for CpuParser {
    fn parse(_path: &str, files: &Vec<FileInfo>) -> Result<Vec<Cpu>, AnalysisError> {
        files
            .iter()
            .filter(|f| f.file_type == FileType::CpuTop)
            .map(|file_info| {
                let file = fs::File::open(&file_info.path)
                    .map_err(|err| AnalysisError::IoError(err.to_string()))?;
                let reader = io::BufReader::new(file);
                let lines_storage: Vec<String> = reader
                    .lines()
                    .take(4) // 仅读取前三行
                    .map(|line| line.map_err(|err| AnalysisError::ParseError(err.to_string())))
                    .collect::<Result<Vec<String>, _>>()?;
                Ok(Cpu::new(
                    lines_storage,
                    &file_info.id,
                    &file_info.work_space,
                ))
            })
            .collect::<Result<Vec<Cpu>, AnalysisError>>()
    }
}

impl ParseFile<Vec<MemoryValue>, FileInfo> for MemoryParser {
    fn parse(path: &str, files: &Vec<FileInfo>) -> Result<Vec<MemoryValue>, AnalysisError> {
        let gc_file: Vec<FileInfo> = files
            .iter()
            .filter(|f| f.file_type == FileType::Gc)
            .cloned()
            .collect();
        // 处理两种情况：
        // 提取排序后的前两个元素
        let mut cycle: Option<i64> = None;
        let mut start_time = None;
        let mut sorted_files: Vec<&FileInfo> =
            gc_file.iter().filter(|f| f.time.is_some()).collect();
        if sorted_files.len() > 1 && start_time.is_none() && cycle.is_none() {
            sorted_files.sort_by_key(|f| f.time.unwrap());
            if let (Some(first), Some(second)) = (sorted_files.get(0), sorted_files.get(1)) {
                let first_time = first.time.unwrap();
                let second_time = second.time.unwrap();
                // 计算时间差值
                let time_difference: Duration = second_time - first_time;
                cycle = Some(time_difference.num_seconds());
                start_time = Some(first_time);
            }
        }
        let mut memory_infos = Vec::new();
        for file in gc_file.iter() {
            let path = Path::new(&file.path);
            let parent = path.parent().unwrap();
            if parent.to_str().unwrap().contains("gc") {
                memory_infos = memory::batch_crate_memory_info(
                    file.path.as_str(),
                    start_time,
                    cycle,
                );
            } else {
                let memory_info = memory::create(&file.path);
                memory_infos.push(memory_info.0);
            }
        }
        Ok(memory_infos)
    }
}

impl ParseFile<HashMap<String, Vec<Thread>>, FileInfo> for ThreadParser {
    fn parse(
        _path: &str,
        files: &Vec<FileInfo>,
    ) -> Result<HashMap<String, Vec<Thread>>, AnalysisError> {
        let stack_file: Vec<FileInfo> = files
            .iter()
            .filter(|f| f.file_type == FileType::StackTrace)
            .cloned()
            .collect();
        let thread_map : HashMap<String, Vec<Thread>>= stack_file
        .par_iter()
        .filter_map(|file_info| {
            let path = Path::new(&file_info.path);
            let file = fs::File::open(path).unwrap();

            let reader = io::BufReader::new(file);
            let mut thread_lines: Vec<Vec<String>> = Vec::new();
            let mut current_group: Vec<String> = Vec::new();
            let mut start = false;
            let mut line_number:i64 = 0;
            let mut line_tag:Vec<(i64, i64)> = Vec::new();
            for line in reader.lines() {
                line_number+=1;
                match line {
                    Ok(line) => {
                        if line.is_empty() {
                            start = false;
                            continue;
                        }
                        if line.contains("nid=") {
                            start = true;
                            if let Some((last)) = line_tag.last_mut(){
                                last.1 = line_number - 2;
                            }
                            line_tag.push((line_number, line_number));
                            if !current_group.is_empty() {
                                thread_lines.push(current_group);
                                current_group = Vec::new();
                            }
                        }
                        if start {
                            current_group.push(line);
                        }
                    }
                    Err(_) => {}
                }
            }
            if !current_group.is_empty() {
                if let Some(last) = line_tag.last_mut() {
                    last.1 = line_number - 2;
                }
                thread_lines.push(current_group);
            }
            let file_thread_info: Vec<Thread> = thread_lines
                .par_iter()
                .enumerate()
                .filter_map(|(idx, group)| {
                    match Thread::new(group,line_tag[idx].0, line_tag[idx].1) {
                        Ok(thread) => Some(thread),
                        Err(err) => {
                            eprintln!("解析线程失败: {:?}", err); // 打印或记录错误
                            None // 失败时返回 None
                        }
                    }
                    
                })
                .collect();
            if !file_thread_info.is_empty() {
                Some((file_info.id.clone(), file_thread_info))   
            }else {
                None
            }
        })
        .collect();
        Ok(thread_map)
    }
}
