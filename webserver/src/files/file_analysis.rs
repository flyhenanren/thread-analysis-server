use std::{
    fs::{self, File},
    io::{self, Read},
};
use std::path::Path;
use crate::{files::zip_extract, models::file_info::FileInfo};

pub fn analysis(path: &str) {
    let file_type = get_file_type(path).map_err(|e| {
        panic!("文件类型校验时发生错误：{}", e);
    }).unwrap();
    let source_path = Path::new(path);
    if file_type == 1 {
        let file_info = zip_extract::unzip_and_extract_file(&source_path).map_err(|e| {
            panic!("解析文件时发生错误：{}", e);
        }).unwrap();    
        build_index(path, file_info);
    }
}

/**
 * 获取选中的路径类型，是文件夹还是压缩包
 */
fn get_file_type(path: &str) -> Result<u8, io::Error> {
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
        return Err(io::Error::new(io::ErrorKind::InvalidData, "文件长度为0"));
    }
    if !check_file_type(&buffer) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "非法的文件类型"));
    }
    Ok(0)
}

/**
 * 创建索引文件
 */
fn build_index(path: &str, file_info: Vec<FileInfo>) {
    
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
    fn test_unzip(){
        let path = Path::new("D:\\dump\\20240726XNJK[非涉密].zip");
        analysis(path.as_str());
    }
}