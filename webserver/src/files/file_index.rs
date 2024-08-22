use crate::models::file_info::FileInfo;
use serde_json::{from_str, to_string};
use std::io::{BufRead, BufReader, ErrorKind};
use std::io::{BufWriter, Write};
use std::{fs::File, path::Path};

pub trait FileIndex {
    fn read_index(path: &str) -> std::io::Result<Vec<FileInfo>>;
    fn write_index(file_info: &Vec<FileInfo>, path: &str) -> std::io::Result<()>;
    fn exist_index(path: &str) -> bool;

    fn read(path: &str, file_name: &str) -> std::io::Result<Vec<FileInfo>>{
        let target_path = Path::new(path).join(file_name);
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
    fn exist(path: &str, name: &str) -> bool{
        let target_dir = Path::new(path).join(name);
        target_dir.exists()
    }

    fn write(file_info: &Vec<FileInfo>, path: &str, file_name: &str)-> std::io::Result<()>{
        if !Self::exist(path, &file_name) {
            let file = File::create(Path::new(path).join(file_name))?;
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
}

pub struct SourceFile{}
pub struct CpuFile{}

impl FileIndex for SourceFile {
    fn read_index(path: &str) -> std::io::Result<Vec<FileInfo>> {
        Self::read(path, "f_idx")
    }

    fn write_index(file_info: &Vec<FileInfo>, path: &str) -> std::io::Result<()>{
       Self::write(file_info, path, "f_idx")
    }

    fn exist_index(path: &str) -> bool {
        Self::exist(path, "f_idx")        
    }
}
