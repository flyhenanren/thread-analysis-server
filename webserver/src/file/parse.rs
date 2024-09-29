use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::iter::Map;
use std::{fs, vec};
use std::{fs::File, path::Path};

use crate::common::utils;
use crate::error::AnalysisError;
use crate::models::cpu::Cpu;
use crate::models::file_info::{FileInfo, FileType};
use crate::models::memory::{self, MemoryValue};
use crate::models::thread::{StackDumpInfo, Thread};
use crate::{CpuInfo, ThreadInfo};

use super::index::{self, DumpInfo};

pub trait ParseFile<T, U> {
    fn parse( path: &str, files: &Vec<U>) -> Result<T, AnalysisError>;
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
                Ok(Cpu::new(lines_storage,&file_info.id, &file_info.work_space))
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
        match index::read_dump(path) {
            Ok(dump) => {
                cycle = Some(dump.time_cycle);
                start_time = Some(dump.start_time);
            }
            Err(_err) => println!("未取得dump信息"),
        }

        let mut sorted_files: Vec<&FileInfo> =
            gc_file.iter().filter(|f| f.time.is_some()).collect();
        if sorted_files.len() > 1 && start_time.is_none() && cycle.is_none() {
            sorted_files.sort_by_key(|f| {
                f.time.unwrap()
            });
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
                    start_time.unwrap(),
                    cycle.unwrap(),
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
    fn parse(path: &str, files: &Vec<FileInfo>) -> Result<HashMap<String, Vec<Thread>>, AnalysisError> {
        let stack_file: Vec<FileInfo> = files
            .iter()
            .filter(|f| f.file_type == FileType::StackTrace)
            .cloned()
            .collect();
        let mut start_time: Option<NaiveDateTime> = None;
        let mut time_cycle: Option<i64> = None;
        let mut thread_map = HashMap::with_capacity(stack_file.len());
        for file_info in stack_file {
            let path = Path::new(&file_info.path);
            let file = fs::File::open(path).unwrap();

            let reader = io::BufReader::new(file);
            let mut thread_lines: Vec<Vec<String>> = Vec::new();
            let mut current_group: Vec<String> = Vec::new();
            let mut start = false;
            let mut time: Option<NaiveDateTime> = None;
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if line.is_empty() {
                            start = false;
                            continue;
                        }
                        if time.is_none() {
                            if utils::is_valid_datetime(&line) {
                                let parsed_time = utils::parse_time(&line).unwrap();
                                time = Some(parsed_time);
                                if start_time.is_none() {
                                    start_time = Some(parsed_time);
                                } else if time_cycle.is_none() {
                                    let time_difference: Duration =
                                        parsed_time - start_time.unwrap();
                                    time_cycle = Some(time_difference.num_seconds());
                                }
                            }
                        }
                        if line.contains("nid=") {
                            start = true;
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
                thread_lines.push(current_group);
            }
            let file_thread_info: Vec<Thread> = thread_lines
                .iter()
                .filter_map(|group| {
                    match Thread::new(group) {
                        Ok(thread) => Some(thread),
                        Err(err) => {
                            eprintln!("解析线程失败: {:?}", err); // 打印或记录错误
                            None // 失败时返回 None
                        }
                    }
                })
                .collect();
            thread_map.insert(file_info.id, file_thread_info);
        }
        index::write_dump(
            DumpInfo {
                start_time: start_time.unwrap(),
                time_cycle: time_cycle.unwrap(),
            },
            path,
        );
        Ok(thread_map)
    }
}
