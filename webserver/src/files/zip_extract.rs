use std::fs;
use std::fs::File;
use std::io::{copy, Read, Seek, Write};
use std::path::Path;
use std::str;
use walkdir::{DirEntry, WalkDir};


pub fn extract(test: &Path, mut target: &Path) {
    let zipfile = std::fs::File::open(&test).unwrap();
    let mut zip = zip::ZipArchive::new(zipfile).unwrap();

    if !target.exists() {
        fs::create_dir_all(target).map_err(|e| {
            println!("{}", e);
        });
    }
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        println!("Filename: {} {:?}", file.name(), file.sanitized_name());
        if file.is_dir() {
            println!("file utf8 path {:?}", file.name_raw()); //文件名编码,在windows下用winrar压缩的文件夹，中文文夹件会码(发现文件名是用操作系统本地编码编码的，我的电脑就是GBK),本例子中的压缩的文件再解压不会出现乱码
            let target = target.join(Path::new(&file.name().replace("\\", "")));
            fs::create_dir_all(target).unwrap();
        } else {
            let file_path = target.join(Path::new(file.name()));
            let mut target_file = if !file_path.exists() {
                println!("file path {}", file_path.to_str().unwrap());
                fs::File::create(file_path).unwrap()
            } else {
                fs::File::open(file_path).unwrap()
            };
            copy(&mut file, &mut target_file);
            // target_file.write_all(file.read_bytes().into());
        }
    }
}
