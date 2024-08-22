use std::fs::{self, File};
use std::io::{self, BufWriter, Error, Read, Write};
use std::path::Path;

use crate::models::file_info::FileType;
use crate::{files::file_index, files::zip_extract, models::file_info::FileInfo};

use super::file_index::FileIndex;

pub fn analysis(path: &str) {
    let file_type = get_file_type(path)
        .unwrap_or_else(|e| {
            panic!("文件类型校验时发生错误：{}", e);
        });
    let source_path = Path::new(path);
     // 封装处理文件提取和索引的逻辑
     let extract_files = |path: &Path| -> Vec<FileInfo> {
        zip_extract::extract_file(path)
            .unwrap_or_else(|e| panic!("读取文件时发生错误：{}", e))
    };
    let file_info = match file_type {
        1 => {
            if file_index::SourceFile::exist_index(path) {
                match file_index::SourceFile::read_index(path) {
                    Ok(file) => file,
                    Err(e) => {
                        println!("{:?}", e);
                        let file_info = extract_files(source_path);
                        file_index::SourceFile::write_index(&file_info, path).ok();
                        file_info
                    }
                }
            } else {
                let file_info = extract_files(source_path);
                file_index::SourceFile::write_index(&file_info, path).ok();
                file_info
            }
        },
        _ => {
            let extract_files = zip_extract::unzip_and_extract_file(source_path)
                .unwrap_or_else(|e| panic!("解析文件时发生错误：{}", e));
            file_index::SourceFile::write_index(&extract_files, path).ok();
            extract_files
        }
    };
    let stack_file: Vec<&FileInfo> = file_info
        .iter()
        .filter(|f| f.file_type == FileType::StackTrace)
        .collect();
    let gc_file: Vec<&FileInfo> = file_info
        .iter()
        .filter(|f| f.file_type == FileType::Gc)
        .collect();

}


fn analysis_cpu(file_info: &Vec<FileInfo>) {

    let cpu_file: Vec<&FileInfo> = file_info
    .iter()
    .filter(|f| f.file_type == FileType::CpuTop)
    .collect();

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
