use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::f32::consts::E;
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::ptr::null;
use std::{fs::File, path::Path};

use crate::common::utils;
use crate::models::cpu::Cpu;
use crate::models::file_info::{FileInfo, FileType, ThreadsInfo};
use crate::models::memory::{self, MemoryInfo};
use crate::models::thread::{Thread, ThreadStatus};

pub trait FileIndex<T> {
    fn read_index(path: &str) -> std::io::Result<Vec<T>>;
    fn write_index(files: &Vec<FileInfo>, path: &str);
    fn exist_index(path: &str) -> bool;
}

fn read(path: &str, file_name: &str) -> std::io::Result<Vec<String>> {
    let target_path = Path::new(path).join(file_name);
    if target_path.exists() {
        let file = File::open(&target_path)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line?;
            lines.push(line);
        }
        return Ok(lines);
    }
    Err(std::io::Error::new(ErrorKind::NotFound, "不存在索引文件"))
}
fn exist(path: &str, name: &str) -> bool {
    let target_dir = Path::new(path).join(name);
    target_dir.exists()
}

fn write(lines: &Vec<String>, path: &str, file_name: &str) -> std::io::Result<()> {
    if !exist(path, &file_name) {
        let file = File::create(Path::new(path).join(file_name))?;
        let mut writer = BufWriter::new(file);
        for line in lines {
            // 写入 JSON 字符串到文件，每行一个 JSON 对象
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}

pub struct SourceIndex;
pub struct ThreadsIndex;
pub struct StackIndex;
pub struct CpuIndex;
pub struct MemoryIndex;

#[derive(Serialize, Deserialize)]
pub struct DumpInfo{
    start_time: NaiveDateTime,
    time_cycle: i64
}

impl FileIndex<FileInfo> for SourceIndex {
    fn read_index(path: &str) -> io::Result<Vec<FileInfo>> {
        read(path, "f_idx")?
            .into_iter()
            .map(|line| {
                from_str::<FileInfo>(&line).map_err(|err| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("无法解析:{}", &line))
                })
            })
            .collect()
    }

    fn write_index(file_info: &Vec<FileInfo>, path: &str) {
        let lines: Vec<String> = file_info
            .iter()
            .map(|file| to_string(&file).expect(&format!("序列化错误：{:?}", file)))
            .collect();
        let _ = write(&lines, path, "f_idx");
    }

    fn exist_index(path: &str) -> bool {
        exist(path, "f_idx")
    }
}

impl FileIndex<Cpu> for CpuIndex {
    fn read_index(path: &str) -> std::io::Result<Vec<Cpu>> {
        read(path, "cpu_idx")?
            .into_iter()
            .map(|line| {
                from_str::<Cpu>(&line).map_err(|err| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("无法解析:{}", &line))
                })
            })
            .collect()
    }

    fn write_index(file_info: &Vec<FileInfo>, path: &str) {
        let cpu_file: Vec<FileInfo> = file_info
            .iter()
            .filter(|f| f.file_type == FileType::CpuTop)
            .cloned()
            .collect();
        let mut cpu_lines = Vec::with_capacity(cpu_file.len());

        for file in cpu_file {
            let file = fs::File::open(file.path).unwrap();
            let reader = io::BufReader::new(file);
            let lines_storage: Vec<String> = reader
                .lines()
                .take(4) // 仅读取前三行
                .map(|line| line.unwrap())
                .collect();
            let cpu = Cpu::new(lines_storage);
            let line = to_string(&cpu).expect("解析CPU信息错误");
            cpu_lines.push(line);
        }
        write(&cpu_lines, path, "cpu_idx");
    }

    fn exist_index(path: &str) -> bool {
        exist(path, "cpu_idx")
    }
}

impl FileIndex<MemoryInfo> for MemoryIndex {
    fn read_index(path: &str) -> std::io::Result<Vec<MemoryInfo>> {
        read(path, "mem_idx")?
        .into_iter()
        .map(|line| {
            from_str::<MemoryInfo>(&line).map_err(|err| {
                io::Error::new(io::ErrorKind::InvalidData, format!("无法解析:{}", &line))
            })
        })
        .collect()
    }

    fn write_index(file_info: &Vec<FileInfo>, path: &str) {
        let gc_file: Vec<FileInfo> = file_info
            .iter()
            .filter(|f| f.file_type == FileType::Gc)
            .cloned()
            .collect();
        // 处理两种情况：
        // 提取排序后的前两个元素
        let mut cycle: Option<i64> = None;
        let mut start_time = None;
        match read_dump(path) {
            Ok(dump) => {
                cycle = Some(dump.time_cycle);
                start_time = Some(dump.start_time);
            },
            Err(err) => println!("未取得dump信息"),
        }
        let mut memory_infos = Vec::new();
        let mut sorted_files: Vec<&FileInfo> = gc_file.iter().filter(|f| f.time.is_some()).collect();
        if sorted_files.len() > 1 && start_time.is_none() && cycle.is_none(){
            sorted_files.sort_by_key(|f| {
                NaiveDateTime::parse_from_str(f.time.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S").unwrap()
            });
            if let (Some(first), Some(second)) = (sorted_files.get(0), sorted_files.get(1)) {
                let first_time =
                    NaiveDateTime::parse_from_str(first.time.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S")
                        .unwrap();
                let second_time =
                    NaiveDateTime::parse_from_str(second.time.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S")
                        .unwrap();
                // 计算时间差值
                let time_difference: Duration = second_time - first_time;
                cycle = Some(time_difference.num_seconds());
                start_time = Some(first_time);
            }
        }

        for file in gc_file.iter() {
            let path = Path::new(&file.path);
            let parent = path.parent().unwrap();
            if parent.to_str().unwrap().contains("gc") {
                memory_infos =
                    memory::batch_crate_memory_info(file.path.as_str(), start_time.unwrap(), cycle.unwrap());
            } else {
                let memory_info = memory::create(&file.path);
                memory_infos.push(memory_info.0);
            }
        }
        let mut lines = Vec::new();
        for info in memory_infos {
            let line = to_string(&info).expect("解析gc信息错误");
            lines.push(line);
        }
        let _ = write(&lines, path, "mem_idx");
    }

    fn exist_index(path: &str) -> bool {
        exist(path, "mem_idx")
    }
}

impl FileIndex<ThreadsInfo> for ThreadsIndex {
    fn read_index(path: &str) -> io::Result<Vec<ThreadsInfo>> {
        read(path, "stack_idx")?
            .into_iter()
            .map(|line| {
                from_str::<ThreadsInfo>(&line).map_err(|err| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("无法解析:{}", &line))
                })
            })
            .collect()
    }

    fn write_index(files: &Vec<FileInfo>, path: &str) {
        let stack_file: Vec<FileInfo> = files
            .iter()
            .filter(|f| f.file_type == FileType::StackTrace)
            .cloned()
            .collect();
        let mut stack_lines = Vec::with_capacity(stack_file.len());
        let mut thread_file_lines = Vec::with_capacity(stack_file.len());
        
        let mut start_time: Option<NaiveDateTime> = None;
        let mut time_cycle: Option<i64> = None;

        for file in stack_file {
            let path = Path::new(&file.path);
            let file = fs::File::open(path).unwrap();
            let file_name = Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("无法获取文件名");

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
                                }else if time_cycle.is_none() {
                                    let time_difference: Duration = parsed_time - start_time.unwrap();
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

            let mut thread_info = ThreadsInfo::new(file_name, &time.unwrap(), thread_lines.len() as i32);
            for group in thread_lines {
                match Thread::new(group) {
                    Ok(thread) => {
                        let line = to_string(&thread).expect("解析thread信息错误");
                        stack_lines.push(line);
                        if thread.status == ThreadStatus::Blocked {
                            thread_info.increment_block();
                        } else if thread.status == ThreadStatus::Runnable {
                            thread_info.increment_run();
                        }
                    }
                    Err(e) => println!("解析失败: {}", e),
                }
            }
            thread_file_lines.push(to_string(&thread_info).expect("msg"));
        }
        let dump_info = DumpInfo{
            start_time: start_time.unwrap(),
            time_cycle: time_cycle.unwrap(),
        };
        let _ =write(&thread_file_lines, path, "thread_idx");
        write_dump(dump_info, path);
    }

    fn exist_index(path: &str) -> bool {
        exist(path, "thread_idx")
    }
}


fn write_dump(dump_info: DumpInfo, path: &str){
    let _ =write(&vec![to_string(&dump_info).expect("msg")], path, "dump_idx");
}

fn read_dump(path: &str) -> io::Result<DumpInfo> {
    read(path, "dump_idx")?
    .into_iter()
        .find_map(|line| {
            match from_str::<DumpInfo>(&line) {
                Ok(info) => Some(Ok(info)),  // 返回解析成功的 DumpInfo
                Err(_) => None,  // 如果解析失败，则返回 None，继续查找下一个
            }
        })
        .unwrap_or_else(|| {
            // 如果没有成功解析的条目，返回错误
            Err(io::Error::new(io::ErrorKind::InvalidData, "无法解析任何有效的 DumpInfo"))
        })
}
