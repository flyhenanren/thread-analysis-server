use actix_web::web::{self, Form};
use chrono::NaiveDateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{common::utils, error::ThreadError, models::config::EnvVars};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub id: String,
    pub work_space: String,
    pub path: String,
    pub file_type: FileType,
    pub time: Option<NaiveDateTime>,
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
    pub fn new(path: &PathBuf, work_space: &str) -> Self {
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
            id: utils::rand_id(),
            work_space: work_space.into(),
            path: path.to_str().expect("Invalid Path").to_string(),
            file_type,
            time,
        }
    }

    fn extract_time_info(file_name: &str) -> Option<NaiveDateTime> {
        // 查找匹配项并提取时间信息
        match REGEX_TIME
            .captures(file_name)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string()) {
                Some(time) => Some(utils::parse_thread_time(&time).unwrap()),
                None => None,
            }
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

impl From<web::Json<FileType>> for FileType {
    fn from(file_type: web::Json<FileType>) -> Self {
        file_type.into_inner()
    }
}


impl TryFrom<i8> for FileType{
    type Error = ThreadError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(FileType::CpuThread),
            1 => Ok(FileType::CpuTop),
            1 => Ok(FileType::StackTrace),
            1 => Ok(FileType::Gc),
            1 => Ok(FileType::GcUtil),
            _ => Ok(FileType::None)
        }
    }
}


impl From<FileType> for i8 {
    fn from(value: FileType) -> Self {
        value as i8
    }
}