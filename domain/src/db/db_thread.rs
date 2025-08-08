use serde_json::from_str;

use common::string_utils::rand_id;
use serde::Serialize;
use serde_json::to_string;
use sqlx::FromRow;
use common::error::DBError;
use sqlx::{SqlitePool};

use crate::{model::thread::{CallFrame, StatusQuery, Thread, ThreadStatus}};

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct DBThreadInfo {
    #[sqlx(rename = "ID")]
    pub id: String,
    #[sqlx(rename = "FILE_ID")]
    pub file_id: String,
    #[sqlx(rename = "THREAD_ID")]
    pub thread_id: Option<String>,
    #[sqlx(rename = "THREAD_NAME")]
    pub thread_name: String,
    #[sqlx(rename = "DAEMON")]
    pub daemon: bool,
    #[sqlx(rename = "PRIO")]
    pub prio: Option<u16>,
    #[sqlx(rename = "OS_PRIO")]
    pub os_prio: u32,
    #[sqlx(rename = "TID")]
    pub tid: String,
    #[sqlx(rename = "NID")]
    pub nid: String,
    #[sqlx(rename = "ADDRESS")]
    pub address: Option<String>,
    #[sqlx(rename = "THREAD_STATUS")]
    pub thread_status: i8,
    #[sqlx(rename = "START_LINE")]
    pub start_line: i64,
    #[sqlx(rename = "END_LINE")]
    pub end_line: i64,
    #[sqlx(rename = "TOP_METHOD")]
    pub top_method: String,
    #[sqlx(rename = "STACK_INFO")]
    pub stack_info: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DBThread {
    #[sqlx(rename = "ID")]
    pub id: String,
    #[sqlx(rename = "FILE_PATH")]
    pub file_path: String,
    #[sqlx(rename = "THREAD_NAME")]
    pub thread_name: String,
    #[sqlx(rename = "THREAD_STATUS")]
    pub thread_status: i8,
    #[sqlx(rename = "START_LINE")]
    pub start_line: i64,
    #[sqlx(rename = "END_LINE")]
    pub end_line: i64
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct StatusInfo {
    #[sqlx(rename = "FILE_PATH")]
    pub file_name: String,
    #[sqlx(rename = "THREAD_NAME")]
    pub thread_name: String,
    #[sqlx(rename = "THREAD_STATUS")]
    pub thread_status: i8,
}

impl DBThreadInfo{
  pub fn new(thread: &Thread, file_id: &str) -> Self{
      DBThreadInfo {
          id: rand_id(),
          file_id: file_id.into(),
          thread_id: thread.id.clone(),
          thread_name: thread.name.clone(),
          daemon: thread.daemon,
          prio: thread.prio.clone(),
          os_prio: thread.os_prio,
          tid: thread.tid.clone(),
          nid: thread.nid.clone(),
          address: thread.address.clone(),
          thread_status: thread.status.clone().into(),
          start_line: thread.start,
          end_line: thread.end,
          top_method: thread.frames.first().and_then(|frame| frame.signature.clone()).unwrap_or_default(),
          stack_info: thread.frames.iter().map(|frame| to_string(&frame.frame).unwrap()).collect::<Vec<String>>().join("\n"),
      }
  }
  pub fn to_thread(&self) -> Thread {
        let frames: Vec<CallFrame> = self.stack_info
            .lines()
            .filter_map(|line| from_str(line).ok())
            .collect();
        Thread {
            id: self.thread_id.clone(),
            name: self.thread_name.clone(),
            daemon: self.daemon,
            prio: self.prio,
            os_prio: self.os_prio,
            tid: self.tid.clone(),
            nid: self.nid.clone(),
            status: ThreadStatus::try_from(self.thread_status).unwrap_or(ThreadStatus::Unknown),
            address: self.address.clone(),
            frames,
            start: self.start_line,
            end: self.end_line,
        }
    }
}


pub async fn batch_add(
    pool: &SqlitePool,
    thread_infos: Vec<DBThreadInfo>
) -> Result<(), DBError> {
    const BATCH_SIZE: usize = 1000; // 每个事务处理的最大记录数
    for chunk in thread_infos.chunks(BATCH_SIZE) {
        // 开始一个新的事务
        let mut transaction = pool.begin().await?;

        // 构建批量插入的 SQL 语句
        let insert_query = String::from(
            r#"INSERT INTO THREAD_INFO 
            (ID, FILE_ID, THREAD_ID, THREAD_NAME, DAEMON, PRIO, OS_PRIO, TID, NID, ADDRESS,THREAD_STATUS, START_LINE, END_LINE, TOP_METHOD, STACK_INFO) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        );
        for thread_info in chunk.iter() {
            sqlx::query(&insert_query)
                .bind(thread_info.id.to_owned())
                .bind(thread_info.file_id.to_owned())
                .bind(thread_info.thread_id.clone().unwrap_or_default())
                .bind(thread_info.thread_name.to_owned())
                .bind(thread_info.daemon)
                .bind(thread_info.prio)
                .bind(thread_info.os_prio)
                .bind(thread_info.tid.to_owned())
                .bind(thread_info.nid.to_owned())
                .bind(thread_info.address.to_owned())
                .bind(thread_info.thread_status)
                .bind(thread_info.start_line)
                .bind(thread_info.end_line)
                .bind(thread_info.top_method.to_owned())
                .bind(thread_info.stack_info.clone())
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
    }
    Ok(())
}

pub async fn list_by_work_space(
    pool: &SqlitePool,
    work_space_id: &str,
) -> Result<Vec<DBThreadInfo>, DBError> {
    let work_space =
        sqlx::query_as::<_, DBThreadInfo>("SELECT T.* FROM THREAD_INFO T 
                                                    LEFT JOIN FILE_INFO I
                                                    ON T.FILE_ID = I.ID
                                                    WHERE I.WORKSPACE = ?")

            .bind(work_space_id)
            .fetch_all(pool)
            .await?;
    Ok(work_space)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<DBThreadInfo, DBError> {
    let work_sapce = sqlx::query_as::<_, DBThreadInfo>("SELECT * FROM THREAD_INFO WHERE ID = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(work_sapce)
}

pub async fn delete(pool: &SqlitePool, id: i32) -> Result<bool, DBError> {
    let result = sqlx::query("DELETE * FROM THREAD_INFO WHERE ID = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_all(pool: &SqlitePool) -> Result<(), DBError> {
    sqlx::query("DELETE FROM THREAD_INFO")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_threads_status(
    pool: &SqlitePool,
    _status: &StatusQuery,
) -> Result<Vec<StatusInfo>, DBError> {
    let result = sqlx::query_as::<_, StatusInfo>(
        r#"select f.FILE_PATH, t.* from THREAD_INFO t
                                left join file_info f
                                on t.FILE_ID == f.id
                                order by f.EXE_TIME asc"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(result)
}

pub async fn list_threads(
    pool: &SqlitePool,
    file_id: &str,
    status: &Option<ThreadStatus>,
    thread_ids: &Option<Vec<String>>,
) -> Result<Vec<DBThreadInfo>, DBError> {
    let mut sql = "SELECT * FROM THREAD_INFO WHERE file_id = ?".to_string();

    if let Some(ids) = thread_ids.as_ref().filter(|ids| !ids.is_empty()) {
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        sql.push_str(&format!(" AND ID IN ({})", placeholders));
    }

    if status.is_some() {
        sql.push_str(" AND THREAD_STATUS = ?");
    }

    let mut query_builder = sqlx::query_as::<_, DBThreadInfo>(&sql).bind(file_id);

    if let Some(ids) = thread_ids {
        for id in ids {
            query_builder = query_builder.bind(id);
        }
    }

    if let Some(status) = status {
        query_builder = query_builder.bind(i8::from(status.clone())); // 使用 From 转换
    }
    let result = query_builder.fetch_all(pool).await?;
    Ok(result)
}



