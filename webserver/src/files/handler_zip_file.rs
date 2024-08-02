use std::{
    fs::{self, metadata, File},
    io::{self, Read},
};
use crate::files::zip_extract;
// 解压文件
pub fn unzip_file(path: &str) {
    let valid = valid_file_type(path);
    let traget = *path+"/unzip";
    zip_extract::extract(path, target)
}

fn valid_file_type(path: &str) -> Result<bool, io::Error> {
    let meta_data = fs::metadata(path).unwrap_or_else(|e| {
        println!("Error reading metadata: {:?}", e);
        panic!("无法解析路径:{} 对应的文件", path);
    });

    if !meta_data.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "不是文件或压缩包",
        ));
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
    Ok(true)
}

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

    use super::valid_file_type;

    #[test]
    fn test_zip_type() {
        let path = Path::new("D:\\dump\\b.txt");
        valid_file_type(path.as_str());
    }
}
