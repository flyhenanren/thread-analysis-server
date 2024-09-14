use actix_web::web;
use chrono::NaiveDateTime;
use regex::Regex;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::models::config::EnvVars;

#[derive(Serialize,Deserialize, Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub path: String,
    pub file_type: FileType,
    pub time: Option<String>,
}

lazy_static::lazy_static! {
    static ref REGEX_TIME:Regex = Regex::new(r"(\d{8}_\d{6})").unwrap();
}


impl From<web::Json<FileInfo>> for FileInfo {
    fn from(info: web::Json<FileInfo>) -> Self {
        info.into_inner()
    }
}

impl FileInfo {
    pub fn new(path: &PathBuf) -> Self {
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
        // 查找匹配项并提取时间信息
        REGEX_TIME.captures(file_name)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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


impl From<web::Json<FileType>> for FileType{
    fn from(file_type: web::Json<FileType>) -> Self {
        file_type.into_inner()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThreadsInfo{
    pub file_name: String,
    pub time: NaiveDateTime,
    pub run_threads: i32,
    pub block_threads: i32,
    pub threads: i32
}

impl ThreadsInfo {
    pub fn new(file_name: &str, time: &NaiveDateTime, threads: i32) -> Self {
        ThreadsInfo {
            file_name: file_name.into(),
            time: time.clone(),
            run_threads: 0,
            block_threads: 0,
            threads,
        }
    }

    pub fn increment_run(&mut self){
        self.run_threads+=1;
    }

    pub fn increment_block(&mut self){
        self.block_threads +=1;
    }
}

impl From<web::Json<ThreadsInfo>> for ThreadsInfo {
    fn from(dump_file: web::Json<ThreadsInfo>) -> Self {
        ThreadsInfo {
            file_name: dump_file.file_name.clone(),
            time: dump_file.time.clone(),
            run_threads: dump_file.run_threads,
            block_threads: dump_file.block_threads,
            threads: dump_file.threads
        }
    }
}