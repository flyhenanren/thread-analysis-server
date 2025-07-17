use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::{fs::File, path::Path};


pub trait FileIndex<T, U> {
    fn read_index(path: &str) -> std::io::Result<Vec<T>>;
    fn write_index(files: &Vec<U>, path: &str);
    fn exist_index(path: &str) -> bool;
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
#[allow(dead_code)]
fn exist(path: &str, name: &str) -> bool {
    let target_dir = Path::new(path).join(name);
    target_dir.exists()
}
#[allow(dead_code)]
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


pub fn read_lines_from_file(file_path: &str, start_line: usize, end_line: usize) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            if i + 1 >= start_line && i + 1 <= end_line {
                line.ok()
            } else {
                None
            }
        })
        .collect();

    Ok(lines)
}