use crate::models::file_info::FileInfo;
use actix_web::http::Error;
use serde_json::{from_str, to_string};
use zip::result;
use std::io::{BufRead, BufReader, ErrorKind};
use std::io::{BufWriter, Write};
use std::{fs::File, path::Path};


pub fn exist_file_index(path: &str) -> bool {
    let target_dir = Path::new(path).join("f_idx");
    target_dir.exists()
}
pub fn exist_cpu_index(path: &str) -> bool {
    let target_dir = Path::new(path).join("cpu_index");
    target_dir.exists()
}

pub fn build_cpu_index(cpu_file: Vec<&FileInfo>) {}

pub fn write_file_index(file_info: &Vec<FileInfo>, path: &str) -> std::io::Result<()> {
    if !exist_file_index(path) {
        let file = File::create(Path::new(path).join("f_idx"))?;
        let mut writer = BufWriter::new(file);
        for item in file_info {
            // 将每个对象序列化为 JSON 字符串
            let json_string = to_string(&item).expect("Failed to serialize");
            // 写入 JSON 字符串到文件，每行一个 JSON 对象
            writeln!(writer, "{}", json_string)?;
        }
    }
    Ok(())
}

pub fn read_file_index(target: &str) -> std::io::Result<Vec<FileInfo>> {
    let target_path = Path::new(target).join("f_idx");
    if target_path.exists() {
        let file = File::open(&target_path)?;
        let reader = BufReader::new(file);
        let mut file_info = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let item: FileInfo = from_str(&line).expect("Failed to deserialize");
            file_info.push(item);
        }
        return Ok(file_info);
    }
    Err(std::io::Error::new(ErrorKind::NotFound, "不存在索引文件"))
}
