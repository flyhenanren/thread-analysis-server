use chrono::{Duration, NaiveDateTime};
use serde_json::{from_str, to_string};
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Error, Read, Write};
use std::path::Path;

use crate::error::AnalysisError;
use crate::models::cpu::{Cpu};
use crate::models::file_info::{DumpFile, FileType};
use crate::models::memory::{self};
use crate::models::thread::Thread;
use crate::{files::zip_extract, models::file_info::FileInfo};

use super::file_index::{CpuFile, FileIndex, MemoryFile, SourceFile, StackFile};

pub fn analysis(path: &str) {
    let exist_source_idx = SourceFile::exist_index(path);
    let exist_cpu_idx = CpuFile::exist_index(path);
    let exist_memory_idx = MemoryFile::exist_index(path);
    let exist_stack_idx = StackFile::exist_index(path);
    if exist_source_idx && exist_cpu_idx && exist_memory_idx && exist_stack_idx {
        return;
    }
    let file_type = get_file_type(path).unwrap_or_else(|e| {
        panic!("文件类型校验时发生错误：{}", e);
    });
    let source_path = Path::new(path);
    // 提取文件
    let extract_files = || -> Vec<FileInfo> {
        zip_extract::extract_file(source_path)
            .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
    };
    let unzip_files = || -> Vec<FileInfo> {
        zip_extract::unzip_and_extract_file(source_path)
            .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
    };
    // 读取或生成文件索引
    let files: Vec<FileInfo> = match file_type {
        1 => process_dir_index(path, exist_source_idx, extract_files),
        _ => process_file_index(path, unzip_files),
    };

    // 处理CPU索引
    if !exist_cpu_idx {
        process_cpu_index(&files, path);
    }

    // 处理堆栈索引
    if !exist_stack_idx {
        process_stack_index(&files, path);
    }

    // 处理内存索引 (此处可根据需要进行实现)
    if !exist_memory_idx {
        process_memory_index(&files, path);
    }
}


pub fn list_dump_file(path: &str) -> Result<Vec<DumpFile>, AnalysisError>{
    Ok(vec![])
}

// 处理文件索引，读取或生成并写入索引文件
fn process_dir_index(
    path: &str,
    exist_source_idx: bool,
    extract_files: impl FnOnce() -> Vec<FileInfo>,
) -> Vec<FileInfo> {
    if exist_source_idx {
        match SourceFile::read_index(path) {
            Ok(lines) => lines
                .into_iter()
                .map(|line| from_str(&line).expect(&format!("无法解析:{}", &line)))
                .collect(),
            Err(e) => {
                println!("{:?}", e);
                let file_info = extract_files();
                write_source_index(&file_info, path);
                file_info
            }
        }
    } else {
        let file_info = extract_files();
        write_source_index(&file_info, path);
        file_info
    }
}

// 处理其他文件类型 (非1)
fn process_file_index(path: &str, extract_files: impl FnOnce() -> Vec<FileInfo>) -> Vec<FileInfo> {
    let file_info = extract_files();
    write_source_index(&file_info, path);
    file_info
}

// 写入源文件索引
fn write_source_index(file_info: &Vec<FileInfo>, path: &str) {
    let lines: Vec<String> = file_info
        .iter()
        .map(|file| to_string(&file).expect(&format!("序列化错误：{:?}", file)))
        .collect();
    SourceFile::write_index(&lines, path).ok();
}

// 处理CPU索引
fn process_cpu_index(files: &Vec<FileInfo>, path: &str) {
    let cpu_file: Vec<FileInfo> = files
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
    CpuFile::write_index(&cpu_lines, path).ok();
}

// 处理堆栈索引
fn process_stack_index(files: &Vec<FileInfo>, path: &str) {
    let stack_file: Vec<FileInfo> = files
        .iter()
        .filter(|f| f.file_type == FileType::StackTrace)
        .cloned()
        .collect();
    let mut stack_lines = Vec::with_capacity(stack_file.len());

    for file in stack_file {
        let file = fs::File::open(file.path).unwrap();
        let reader = io::BufReader::new(file);
        let mut thread_lines: Vec<Vec<String>> = Vec::new();
        let mut current_group: Vec<String> = Vec::new();
        let mut start = false;
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if l.is_empty() {
                        start = false;
                        continue;
                    }
                    if l.contains("nid=") {
                        start = true;
                        if !current_group.is_empty() {
                            thread_lines.push(current_group);
                            current_group = Vec::new();
                        }
                    }
                    if start {
                        current_group.push(l);
                    }
                }
                Err(_) => {}
            }
        }
        if !current_group.is_empty() {
            thread_lines.push(current_group);
        }
        for group in thread_lines {
            match Thread::new(group) {
                Ok(thread) => {
                    let line = to_string(&thread).expect("解析thread信息错误");
                    stack_lines.push(line);
                }
                Err(e) => println!("解析失败: {}", e),
            }
        }
    }
    StackFile::write_index(&stack_lines, path).ok();
}

// 处理内存索引
fn process_memory_index(files: &Vec<FileInfo>, path: &str) {
    let gc_file: Vec<FileInfo> = files
        .iter()
        .filter(|f| f.file_type == FileType::Gc)
        .cloned()
        .collect();
    // 处理两种情况：
    let mut memory_infos = Vec::new();
    let mut sorted_files: Vec<&FileInfo> = files.iter().filter(|f| f.time.is_some()).collect();
    sorted_files.sort_by_key(|f| {
        NaiveDateTime::parse_from_str(f.time.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S").unwrap()
    });
    // 提取排序后的前两个元素
    let mut cycle = 0;
    let mut start_time = None;
    if let (Some(first), Some(second)) = (sorted_files.get(0), sorted_files.get(1)) {
        let first_time =
            NaiveDateTime::parse_from_str(first.time.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S")
                .unwrap();
        let second_time =
            NaiveDateTime::parse_from_str(second.time.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S")
                .unwrap();
        // 计算时间差值
        let time_difference: Duration = second_time - first_time;
        cycle = time_difference.num_milliseconds();
        start_time = Some(first_time);
    }
    for file in gc_file.iter() {
        let path = Path::new(&file.path);
        let parent = path.parent().unwrap();
        if parent.to_str().unwrap().contains("gc") {
            memory_infos = memory::batch_crate_memory_info(file.path.as_str(), start_time.unwrap(), cycle);
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
    MemoryFile::write_index(&lines, path).ok();
}
/**
 * 获取选中的路径类型，是文件夹还是压缩包
 */
fn get_file_type(path: &str) -> Result<u8, Error> {
    let meta_data = fs::metadata(path).unwrap_or_else(|e| {
        println!("Error reading metadata: {:?}", e);
        panic!("无法解析路径:{} 对应的文件", path);
    });
    if meta_data.is_dir() {
        return Ok(1);
    }

    let mut file = File::open(path).unwrap();
    let mut buffer = [0; 5];
    let bytes_read = file.read(&mut buffer)?;
    if bytes_read < buffer.len() {
        return Err(Error::new(io::ErrorKind::InvalidData, "文件长度为0"));
    }
    if !check_file_type(&buffer) {
        return Err(Error::new(io::ErrorKind::InvalidData, "非法的文件类型"));
    }
    Ok(0)
}

/**
 * 创建索引文件
 */
fn check_and_build_index(path: &Path, file_info: Vec<FileInfo>) {
    let target: &Path = path.parent().expect("获取压缩包的上级路径错误");
    let call_tree_idx_path = target.join("call_tree".replace("\\", ""));
    if !call_tree_idx_path.exists() {
        if let Ok(file) = File::create(call_tree_idx_path) {
            let mut writer = BufWriter::new(file);
            // 多行数据
            let lines = ["这是第一行", "这是第二行", "这是第三行"];
            // 写入多行数据
            for line in lines.iter() {
                if let Err(e) = writeln!(writer, "{}", line) {
                    eprintln!("写入数据时发生错误: {}", e);
                }
            }
        }
    }
}

/**
 * 检查压缩包类型
 */
fn check_file_type(buffer: &[u8; 5]) -> bool {
    match buffer {
        [0x50, 0x4B, 0x03, 0x04, ..] => {
            println!("This is a ZIP file");
            true
        }
        [0x1F, 0x8B, ..] => {
            println!("This is a GZIP file");
            true
        }
        [0x42, 0x5A, 0x68, ..] => {
            println!("This is a BZIP2 file");
            true
        }
        [0x52, 0x61, 0x72, 0x21, ..] => {
            println!("This is a RAR file");
            true
        }
        [0x75, 0x73, 0x74, 0x61, 0x72, ..] => {
            println!("This is a TAR file");
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use actix_web::dev::Path;

    use super::{analysis, get_file_type};

    #[test]
    fn test_zip_type() {
        let path = Path::new("D:\\dump\\b.txt");
        let _ = get_file_type(path.as_str());
    }

    #[test]
    fn test_unzip() {
        let path = Path::new("D:\\dump\\20240726XNJK[非涉密].zip");
        analysis(path.as_str());
    }

    #[test]
    fn test_walk_dir() {
        let path = Path::new("D:\\dump\\20240726");
        analysis(path.as_str());
    }
}
