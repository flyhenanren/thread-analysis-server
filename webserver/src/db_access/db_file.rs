use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, SourceFileInfo};


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