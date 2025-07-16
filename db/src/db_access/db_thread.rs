use chrono::Utc;
use common::error::DBError;
use domain::{db::{db::StatusInfo, db_thread::DBThreadInfo}, model::thread::{StatusQuery, ThreadStatus}};
use sqlx::{SqlitePool};



pub async fn batch_add(
    pool: &SqlitePool,
    thread_infos: Vec<DBThreadInfo>,
    work_space: &str,
) -> Result<(), DBError> {
    let start = Utc::now().timestamp_millis();
    const BATCH_SIZE: usize = 1000; // 每个事务处理的最大记录数
    for chunk in thread_infos.chunks(BATCH_SIZE) {
        let start_pre = Utc::now().timestamp_millis();
        // 开始一个新的事务
        let mut transaction = pool.begin().await?;

        // 构建批量插入的 SQL 语句
        let insert_query = String::from(
            r#"INSERT INTO THREAD_INFO 
            (ID, WORKSPACE, FILE_ID, THREAD_ID, THREAD_NAME, DAEMON, PRIO, OS_PRIO, TID, NID, ADDRESS,THREAD_STATUS, START_LINE, END_LINE, METHOD_NAME) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        );
        for thread_info in chunk.iter() {
            sqlx::query(&insert_query)
                .bind(thread_info.id.to_owned())
                .bind(work_space.to_owned())
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
                .bind(thread_info.method_name.to_owned())
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
        sqlx::query_as::<_, DBThreadInfo>("SELECT * FROM THREAD_INFO WHERE WORKSPACE = ?")
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
    status: &StatusQuery,
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



