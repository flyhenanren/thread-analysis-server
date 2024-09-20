use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Error, Read, Write};
use std::path::Path;

use crate::error::AnalysisError;
use crate::models::thread::ThreadsInfo;
use crate::{models::file_info::FileInfo};
use crate::common::file_utils::*;
use super::index::{CpuIndex, FileIndex, MemoryIndex, SourceIndex, ThreadsIndex};


/// 分析文件夹或者文件中的线程信息
pub fn analysis(path: &str) {
    let exist_source_idx = SourceIndex::exist_index(path);
    let exist_cpu_idx = CpuIndex::exist_index(path);
    let exist_memory_idx = MemoryIndex::exist_index(path);
    let exist_stack_idx = ThreadsIndex::exist_index(path);
    if exist_source_idx && exist_cpu_idx && exist_memory_idx && exist_stack_idx {
        return;
    }
    let file_type = get_file_type(path).unwrap_or_else(|e| {
        panic!("文件类型校验时发生错误：{}", e);
    });
    let source_path = Path::new(path);
    // 提取文件
    let extract_files = || -> Vec<FileInfo> {
        extract_file(source_path)
            .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
    };
    let unzip_files = || -> Vec<FileInfo> {
        unzip_and_extract_file(source_path)
            .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
    };
    // 读取或生成文件索引
    let files: Vec<FileInfo> = match file_type {
        1 => process_dir_index(path, exist_source_idx, extract_files),
        _ => process_file_index(path, unzip_files),
    };

    // 处理CPU索引
    if !exist_cpu_idx {
        CpuIndex::write_index(&files, path);
    }

    // 处理堆栈索引
    if !exist_stack_idx {
        ThreadsIndex::write_index(&files, path);
    }

    // 处理内存索引 (此处可根据需要进行实现)
    if !exist_memory_idx {
        MemoryIndex::write_index(&files, path);
    }
}

/// 获取所有线程文件信息
pub fn list_dump_file(path: &str) -> Result<Vec<ThreadsInfo>, AnalysisError> {
    ThreadsIndex::read_index(path)
        .map_err(|err| AnalysisError::NotFound(format!("读取不到线程信息:{}", err)))
}

// 处理文件索引，读取或生成并写入索引文件
fn process_dir_index(
    path: &str,
    exist_source_idx: bool,
    extract_files: impl FnOnce() -> Vec<FileInfo>,
) -> Vec<FileInfo> {
    if exist_source_idx {
        match SourceIndex::read_index(path) {
            Ok(files) =>  files,
            Err(e) => {
                println!("{:?}", e);
                let file_info = extract_files();
                SourceIndex::write_index(&file_info, path);
                file_info
            }
        }
    } else {
        let file_info = extract_files();
        SourceIndex::write_index(&file_info, path);
        file_info
    }
}

// 处理其他文件类型 (非1)
fn process_file_index(path: &str, extract_files: impl FnOnce() -> Vec<FileInfo>) -> Vec<FileInfo> {
    let file_info = extract_files();
    SourceIndex::write_index(&file_info, path);
    file_info
}


#[cfg(test)]
mod tests {
    use actix_web::dev::Path;

    use crate::common;

    use super::{analysis};

    #[test]
    fn test_zip_type() {
        let path = Path::new("D:\\dump\\b.txt");
        let _ = common::file_utils::get_file_type(path.as_str());
    }

    #[test]
    fn test_unzip() {
        let path = Path::new("D:\\dump\\20240726XNJK[非涉密].zip");
        analysis(path.as_str());
    }

    #[test]
    fn test_walk_dir_all() {
        let path = Path::new("D:\\dump\\20240726");
        analysis(path.as_str());
    }
    #[test]
    fn test_walk_dir() {
        let path = Path::new("D:\\dump\\20240809_1");
        analysis(path.as_str());
    }
}
