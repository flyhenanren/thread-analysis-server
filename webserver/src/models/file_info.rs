use actix_web::web;
use regex::Regex;
use std::path::PathBuf;

use crate::models::config::EnvVars;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: FileType,
    pub time: Option<String>,
}

impl From<web::Json<FileInfo>> for FileInfo {
    fn from(info: web::Json<FileInfo>) -> Self {
        info.into_inner()
    }
}

impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .expect("Invalid Path");
        let env_vars = EnvVars::load();
        let file_type = FileType::new(file_name, &env_vars);
        let time = if file_name.contains(&env_vars.thread_dump) {
            Self::extract_time_info(file_name)
        } else {
            None
        };
        FileInfo {
            path: path.to_str().expect("Invalid Path").to_string(),
            file_type,
            time,
        }
    }

    fn extract_time_info(file_name: &str) -> Option<String> {
        // 创建正则表达式来匹配日期和时间信息
        let re = Regex::new(r"(\d{8}_\d{6})").unwrap();
        // 查找匹配项并提取时间信息
        re.captures(file_name)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    CpuThread,
    CpuTop,
    StackTrace,
    Gc,
    GcUtil,
    None,
}

impl FileType {
    pub fn new(file_name: &str, env: &EnvVars) -> Self {
        match file_name {
            name if name.contains(&env.cpu_file) => FileType::CpuThread,
            name if name.contains(&env.cpu_top) => FileType::CpuTop,
            name if name.contains(&env.thread_dump) => FileType::StackTrace,
            name if name.contains(&env.gc_util) => FileType::GcUtil,
            name if name.contains(&env.gc) => FileType::Gc,
            _ => FileType::None,
        }
    }
}
