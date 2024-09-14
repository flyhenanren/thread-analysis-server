use std::fs::File;
use std::{fs, io};
use std::io::{copy, Error, Read};
use std::path::{Path, PathBuf};

use crate::models::file_info::FileInfo;

/// 从文件夹或者压缩包中提取信息
pub fn unzip_and_extract_file(source: &Path) -> io::Result<Vec<FileInfo>> {
    let target: &Path = source.parent().expect("获取压缩包的上级路径错误");
    let zip_file = fs::File::open(&source)?;
    let mut zip = zip::ZipArchive::new(zip_file)?;

    if !target.exists() {
        fs::create_dir_all(target)?;
    }
    let mut file_mapping:Vec<FileInfo> = vec![];

    for i in 0..zip.len() {
        match zip.by_index(i) {
            Ok(mut file) => {
                let file_name = file.name();
                if file.is_dir() {
                    let target_dir = target.join(file_name.replace("\\", ""));
                    if let Err(e) = fs::create_dir_all(&target_dir) {
                        eprintln!("Failed to create directory {:?}: {}", target_dir, e);
                    }
                } else {
                    let file_path = target.join(file_name);
                    match if file_path.exists() {
                        fs::File::open(&file_path)
                    } else {
                        fs::File::create(&file_path)
                    } {
                        Ok(mut target_file) => {
                            if let Err(e) = copy(&mut file, &mut target_file) {
                                eprintln!("Failed to copy file to {:?}: {}", file_path, e);
                            }
                            file_mapping.push(FileInfo::new(&file_path));
                        }
                        Err(e) => eprintln!("Failed to open or create file {:?}: {}", file_path, e),
                    }
                }
            }
            Err(e) => eprintln!("Failed to process file at index {}: {:?}", i, e)
        }
    }
    Ok(file_mapping)
}

pub fn extract_file(target: &Path)  -> io::Result<Vec<FileInfo>> {
    let mut file_mapping:Vec<FileInfo> = vec![];
    if target.is_dir() {
        for entry in fs::read_dir(target)?{
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 如果是文件夹，递归处理
                let mut children = extract_file(&path)?;
                if children.len() > 0 {
                    file_mapping.append(&mut children);
                }
            } else if path.is_file() {
                // 如果是文件，读取内容
                file_mapping.push(FileInfo::new(&entry.path()))
            }
        }
    }
    Ok(file_mapping)
}

    
/**
 * 获取选中的路径类型，是文件夹还是压缩包
 */
pub fn get_file_type(path: &str) -> Result<u8, Error> {
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
 * 检查压缩包类型
 */
pub fn check_file_type(buffer: &[u8; 5]) -> bool {
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