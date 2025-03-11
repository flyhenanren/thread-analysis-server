use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, SourceFileInfo};

use super::db::DBThread;


pub async fn batch_add(pool: &SqlitePool, file_infos: Vec<SourceFileInfo>) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for file_info in file_infos {
        sqlx::query(
            r#"INSERT INTO FILE_INFO (id, workspace, file_path, file_type, exe_time) VALUES (?,?,?,?,?) "#)
            .bind(file_info.id)
            .bind(file_info.workspace)
            .bind(file_info.file_path)
            .bind(file_info.file_type)
            .bind(file_info.exe_time)
            .execute(pool)
            .await?;    
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<SourceFileInfo>, DBError> {
    let work_space = sqlx::query_as::<_, SourceFileInfo>("SELECT * FROM FILE_INFO WHERE WORKSPACE = ?")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}

pub async fn delete_all(pool: &SqlitePool) -> Result<(), DBError> {
    sqlx::query("DELETE FROM FILE_INFO")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_file_by_thread(pool: &SqlitePool, id: &str) -> Result<DBThread, DBError> {
    let file_info = sqlx::query_as::<_, DBThread>(r#"SELECT T.ID, F.FILE_PATH, T.THREAD_NAME, T.THREAD_STATUS, T.START_LINE, T.END_LINE FROM FILE_INFO F 
                                LEFT JOIN THREAD_INFO T 
                                ON F.ID = T.FILE_ID 
                                WHERE T.ID = ?"#)
    .bind(id)
    .fetch_one(pool)
    .await?;
Ok(file_info)
}