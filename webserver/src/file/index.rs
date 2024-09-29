use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::{fs::File, path::Path};

use crate::models::file_info::{FileInfo, FileType};
use crate::models::thread::{Thread, ThreadStatus};

pub trait FileIndex<T, U> {
    fn read_index(path: &str) -> std::io::Result<Vec<T>>;
    fn write_index(files: &Vec<U>, path: &str);
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

fn read_by_line(
    path: &str,
    file_name: &str,
    start: i64,
    end: i64,
) -> std::io::Result<Vec<String>> {
    let target_path = Path::new(path).join(file_name);
    if !target_path.exists() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "不存在索引文件"));
    }

    let file = File::open(&target_path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    // 改为批量读取并处理
    for line in reader.lines().skip((start - 1) as usize).take((end - start) as usize) {
        let line = line?;
        lines.push(line);
    }

    Ok(lines)
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



#[derive(Serialize, Deserialize)]
pub struct DumpInfo {
    pub start_time: NaiveDateTime,
    pub time_cycle: i64,
}

pub fn write_dump(dump_info: DumpInfo, path: &str) {
    let _ = write(&vec![to_string(&dump_info).expect("msg")], path, "dump_idx");
}

pub fn read_dump(path: &str) -> io::Result<DumpInfo> {
    read(path, "dump_idx")?
        .into_iter()
        .find_map(|line| {
            match from_str::<DumpInfo>(&line) {
                Ok(info) => Some(Ok(info)), // 返回解析成功的 DumpInfo
                Err(_) => None,             // 如果解析失败，则返回 None，继续查找下一个
            }
        })
        .unwrap_or_else(|| {
            // 如果没有成功解析的条目，返回错误
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "无法解析任何有效的 DumpInfo",
            ))
        })
}
