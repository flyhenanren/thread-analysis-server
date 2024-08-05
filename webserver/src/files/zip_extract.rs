use std::{fs, io};
use std::io::{copy};
use std::path::{Path, PathBuf};

pub fn extract_file(source: &Path) -> io::Result<Vec<PathBuf>> {
    let target: &Path = source.parent().expect("获取压缩包的上级路径错误");
    let zip_file = fs::File::open(&source)?;
    let mut zip = zip::ZipArchive::new(zip_file)?;

    if !target.exists() {
        fs::create_dir_all(target)?;
    }
    let mut root_paths = vec![];
    for i in 0..zip.len() {
        match zip.by_index(i) {
            Ok(mut file) => {
                let file_name = file.name();
                let file_mangled_name = file.mangled_name();
                let file_parent = file_mangled_name.parent();

                println!("Filename: {} {:?}", file_name, file_mangled_name);
                
                if file.is_dir() {
                    if let Some(parent) = file_parent {
                        if parent.as_os_str().is_empty() {
                            root_paths.push(file_mangled_name);
                        }
                    }
                    println!("File UTF8 path: {:?}", file.name_raw());
                    let target_dir = target.join(file_name.replace("\\", ""));
                    if let Err(e) = fs::create_dir_all(&target_dir) {
                        eprintln!("Failed to create directory {:?}: {}", target_dir, e);
                    }
                } else {
                    let file_path = target.join(file_name);
                    match if file_path.exists() {
                        fs::File::open(&file_path)
                    } else {
                        println!("Creating file: {}", file_path.to_str().unwrap());
                        fs::File::create(&file_path)
                    } {
                        Ok(mut target_file) => {
                            if let Err(e) = copy(&mut file, &mut target_file) {
                                eprintln!("Failed to copy file to {:?}: {}", file_path, e);
                            }
                        }
                        Err(e) => eprintln!("Failed to open or create file {:?}: {}", file_path, e),
                    }
                }
            }
            Err(e) => eprintln!("Failed to process file at index {}: {:?}", i, e)
        }
    }
    Ok(root_paths)
}
