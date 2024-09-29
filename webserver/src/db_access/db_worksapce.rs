use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::{error::DBError, FileWorkSpace};

pub async fn add(pool: &SqlitePool, work_space: &FileWorkSpace) -> Result<FileWorkSpace, DBError> {
    let row = sqlx::query_as::<_, FileWorkSpace>(
        r#"INSERT INTO FILE_WORKSAPCE (ID, file_path) VALUES (?,?) "#)
        .bind(work_space.id.to_string())
        .bind(work_space.file_path.to_string())
        .fetch_one(pool)
        .await?;
    Ok(row)
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<FileWorkSpace>, DBError> {
    let work_space = sqlx::query_as::<_, FileWorkSpace>("SELECT * FROM FILE_WORKSPACE")
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}

pub async fn get(pool: &SqlitePool, id: i32) -> Result<FileWorkSpace, DBError> {
    let work_sapce =
        sqlx::query_as::<_, FileWorkSpace>("SELECT * FROM FILE_WORKSPACE WHERE ID = ?")
            .bind(id)
            .fetch_one(pool)
            .await?;
    Ok(work_sapce)
}

pub async fn get_by_path(pool: &SqlitePool, path: &str) -> Result<Option<FileWorkSpace>, DBError> {
    let work_sapce =
        sqlx::query_as::<_, FileWorkSpace>("SELECT * FROM FILE_WORKSPACE WHERE FILE_PATH = ?")
            .bind(path)
            .fetch_optional(pool)
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

pub async fn update_time(pool: &SqlitePool, id: i32, time:NaiveDateTime) -> Result<FileWorkSpace, DBError> {
    let result = sqlx::query_as::<_, FileWorkSpace>("UPDATE FILE_WORKSAPCE SET update_time = $1 WHERE ID = $2 
    RETURNING id, file_path, create_time, update_time")
        .bind(time)
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(result)
}
