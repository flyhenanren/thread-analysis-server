use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::{error::DBError, FileWorkSpace};

pub async fn add(pool: &SqlitePool, work_space: &FileWorkSpace) -> Result<(), DBError> {
    sqlx::query(
        r#"INSERT INTO FILE_WORKSPACE (ID, file_path) VALUES (?,?) "#)
        .bind(work_space.id.to_string())
        .bind(work_space.file_path.to_string())
        .execute(pool)
        .await?;
    Ok(())
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
    let work_space =
        sqlx::query_as::<_, FileWorkSpace>("SELECT * FROM FILE_WORKSPACE WHERE FILE_PATH = ?")
            .bind(path)
            .fetch_optional(pool)
            .await?;
    Ok(work_space)
}

pub async fn delete(pool: &SqlitePool, id: i32) -> Result<(), DBError> {
    sqlx::query("DELETE * FROM FILE_WORKSPACE WHERE ID = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_time(pool: &SqlitePool, id: i32, time:NaiveDateTime) -> Result<(), DBError> {
    sqlx::query("UPDATE FILE_WORKSPACE SET update_time = $1 WHERE ID = $2")
        .bind(time)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
