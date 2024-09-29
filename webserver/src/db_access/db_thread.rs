use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, ThreadInfo};

pub async fn batch_add(pool: &SqlitePool, thread_infos: Vec<ThreadInfo>, work_space: &str) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;

    for thread_info in thread_infos {
        sqlx::query_as::<_, ThreadInfo>(
            r#"INSERT INTO THREAD_INFO (ID, WORKSPACE, OWN_FILE, THREAD_ID, THREAD_NAME, DAEMON, THREAD_STATUS)
             VALUES (?,?,?,?,?,?,?,?,?)"#)
             .bind(thread_info.id)
             .bind(work_space)
            .bind(thread_info.file_id)
            .bind(thread_info.thread_id)
            .bind(thread_info.thread_name)
            .bind(thread_info.daemon)
            .bind(thread_info.thread_status)
            .fetch_one(pool)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<ThreadInfo>, DBError> {
    let work_space = sqlx::query_as::<_, ThreadInfo>("SELECT * FROM FILE_WORKSPACE")
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}

pub async fn get(pool: &SqlitePool, id: i32) -> Result<ThreadInfo, DBError> {
    let work_sapce =
        sqlx::query_as::<_, ThreadInfo>("SELECT * FROM FILE_WORKSPACE WHERE ID = ?")
            .bind(id)
            .fetch_one(pool)
            .await?;
    Ok(work_sapce)
}

pub async fn delete(pool: &SqlitePool, id: i32) -> Result<bool, DBError> {
    let result = sqlx::query("DELETE * FROM FILE_WORKSAPCE WHERE ID = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

