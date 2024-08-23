use serde_json::{from_str, to_string};
use std::io::{BufRead, BufReader, ErrorKind};
use std::io::{BufWriter, Write};
use std::{fs::File, path::Path};

pub trait FileIndex {
    fn read_index(path: &str) -> std::io::Result<Vec<String>>;
    fn write_index(files: &Vec<String>, path: &str) -> std::io::Result<()>;
    fn exist_index(path: &str) -> bool;

    fn read(path: &str, file_name: &str) -> std::io::Result<Vec<String>>{
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
    fn exist(path: &str, name: &str) -> bool{
        let target_dir = Path::new(path).join(name);
        target_dir.exists()
    }

    fn write(lines: &Vec<String>, path: &str, file_name: &str)-> std::io::Result<()>{
        if !Self::exist(path, &file_name) {
            let file = File::create(Path::new(path).join(file_name))?;
            let mut writer = BufWriter::new(file);
            for line in lines {
                // 写入 JSON 字符串到文件，每行一个 JSON 对象
                writeln!(writer, "{}", line)?;
            }
        }
        Ok(())
    }
}

pub struct SourceFile;
pub struct StackFile;
pub struct CpuFile;
pub struct MemoryFile;

impl FileIndex for SourceFile {
    fn read_index(path: &str) -> std::io::Result<Vec<String>> {
        Self::read(path, "f_idx")
    }

    fn write_index(file_info: &Vec<String>, path: &str) -> std::io::Result<()>{
       Self::write(file_info, path, "f_idx")
    }

    fn exist_index(path: &str) -> bool {
        Self::exist(path, "f_idx")        
    }
}

impl FileIndex for CpuFile {
    fn read_index(path: &str) -> std::io::Result<Vec<String>> {
        Self::read(path, "cpu_idx")
    }

    fn write_index(file_info: &Vec<String>, path: &str) -> std::io::Result<()>{
       Self::write(file_info, path, "cpu_idx")
    }

    fn exist_index(path: &str) -> bool {
        Self::exist(path, "cpu_idx")        
    }
}

impl FileIndex for MemoryFile {
    fn read_index(path: &str) -> std::io::Result<Vec<String>> {
        Self::read(path, "mem_idx")
    }

    fn write_index(file_info: &Vec<String>, path: &str) -> std::io::Result<()>{
       Self::write(file_info, path, "mem_idx")
    }

    fn exist_index(path: &str) -> bool {
        Self::exist(path, "mem_idx")        
    }
}

impl FileIndex for StackFile {
    fn read_index(path: &str) -> std::io::Result<Vec<String>> {
        Self::read(path, "stack_idx")
    }

    fn write_index(file_info: &Vec<String>, path: &str) -> std::io::Result<()>{
       Self::write(file_info, path, "stack_idx")
    }

    fn exist_index(path: &str) -> bool {
        Self::exist(path, "stack_idx")        
    }
}

