use std::env;

use chrono::{Local, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};

use crate::{common::utils, models::{cpu::Cpu, file_info::FileInfo, memory::MemoryValue, thread::Thread}};

pub async fn establish_connection() -> Pool<Sqlite> {
    let database_url = env::var("DATABASE_URL").expect("无法获取数据库链接");
    SqlitePool::connect(&database_url)
        .await
        .expect("无法连接数据库")
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct FileWorkSpace {
    pub id: String,
    pub file_path: String,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct SourceFileInfo {
    pub id: String,
    pub workspace: String,
    pub file_path: String,
    pub file_type: i8,
    pub exe_time: Option<NaiveDateTime>,
}


#[derive(Serialize, Debug, Clone, FromRow)]
pub struct ThreadInfo {
    pub id: String,
    pub file_id: String,
    pub thread_id: Option<String>,
    pub thread_name: String,
    pub daemon: bool,
    pub prio: Option<u16>,
    pub os_prio: u32,
    pub tid: u64,
    pub nid: u64,
    pub address: Option<String>,
    pub thread_status: i8,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct ThreadStack {
    pub id: String,
    pub work_space: String,
    pub thread_id: String,
    pub class_name: String,
    pub method_name: Option<String>,
    pub stack_lin: Option<u32>,
    pub stack_status: String,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct CpuInfo {
    pub id: String,
    pub workspace: String,
    pub exe_time: NaiveTime,
    pub us: f64,
    pub sy: f64,
    pub ids: f64,
    pub tasks: u32,
    pub running: u32,
    pub sleeping: u32,
    pub mem_total: f64,
    pub mem_free: f64,
    pub mem_used: f64,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct MemoryInfo {
    pub id: String,
    pub workspace: String,
    pub file_id: String,
    pub s0: f64,
    pub s0c: f64,
    pub s0u: f64,
    pub s1c: f64,
    pub s1u: f64,
    pub ec: f64,
    pub eu: f64,
    pub oc: f64,
    pub ou: f64,
    pub mc: f64,
    pub mu: f64,
    pub ccsc: f64,
    pub ccsu: f64,
    pub ygc: f64,
    pub ygct: f64,
    pub fgc: f64,
    pub fgct: f64,
    pub cgc: f64,
    pub cgct: f64,
    pub gct: f64,
    pub exe_time: Option<NaiveDateTime>,
}

pub trait ModelTransfer<T, U> {
    fn new(file: &T, file_id: &str, work_space: &str) -> U;
}

impl ModelTransfer<Cpu, CpuInfo> for CpuInfo {
    fn new(file: &Cpu, _file_id: &str, work_space: &str) -> CpuInfo {
        CpuInfo {
            id: utils::rand_id(),
            workspace: work_space.into(),
            exe_time: file.exe_time,
            us: file.us,
            sy: file.sy,
            ids: file.ids,
            tasks: file.tasks,
            running: file.running,
            sleeping: file.sleeping,
            mem_total: file.mem_total,
            mem_free: file.mem_free,
            mem_used: file.mem_used,
        }
    }
}

impl ModelTransfer<FileInfo, SourceFileInfo> for SourceFileInfo{
    fn new(file: &FileInfo, file_id: &str, work_space: &str) -> Self {
        Self{
            id: file.id.clone(),
            workspace: work_space.into(),
            file_path: file.path.clone(),
            file_type: file.file_type.clone().try_into().unwrap(),
            exe_time: file.time.clone(),
        }
    }
}

impl ThreadStack{
    pub fn new(thread: &Thread, thread_id: &str, work_space: &str) -> Vec<ThreadStack> {
        thread.frames
        .clone()
        .into_iter()
        .map(|frame| {
            ThreadStack{
                id: utils::rand_id(),
                work_space: work_space.into(),
                thread_id: thread_id.into(),
                class_name: frame.class_name,
                method_name: frame.method_name,
                stack_lin: frame.line_number,
                stack_status: to_string(&frame.frame).unwrap(),
            }
        })
        .collect()
    }
}

impl FileWorkSpace{
    pub fn new(path: &str) -> Self{
        FileWorkSpace {
            id: utils::rand_id(),
            file_path: path.into(),
            create_time: Utc::now().naive_utc(),
            update_time: Utc::now().naive_utc()
        }
    }
}

impl ThreadInfo{
    pub fn new(thread: &Thread, file_id: &str) -> Self{
        ThreadInfo {
            id: utils::rand_id(),
            file_id: file_id.into(),
            thread_id: Some(utils::rand_id()),
            thread_name: thread.name.clone(),
            daemon: thread.daemon,
            prio: thread.prio.clone(),
            os_prio: thread.os_prio,
            tid: thread.tid,
            nid: thread.nid,
            address: thread.address.clone(),
            thread_status: thread.status.clone().into()
        }
    }
}


impl MemoryInfo {
    pub fn new(memory: &MemoryValue, work_space: &str) -> Self{
        Self {
            id: utils::rand_id(),
            workspace: work_space.into(),
            file_id: memory.file_id.clone(),
            s0: memory.value[0],
            s0c: memory.value[1],
            s0u: memory.value[2],
            s1c: memory.value[3],
            s1u: memory.value[4],
            ec: memory.value[5],
            eu: memory.value[6],
            oc: memory.value[7],
            ou: memory.value[8],
            mc: memory.value[9],
            mu: memory.value[10],
            ccsc: memory.value[11],
            ccsu: memory.value[12],
            ygc: memory.value[13],
            ygct: memory.value[14],
            fgc: memory.value[15],
            fgct: memory.value[16],
            cgc: memory.value[17],
            cgct: memory.value[18],
            gct: memory.value[19],
            exe_time: memory.time,
        }
    }
}