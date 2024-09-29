use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, SourceFileInfo};


pub async fn batch_add(pool: &SqlitePool, file_infos: Vec<SourceFileInfo>) -> Result<Vec<SourceFileInfo>, DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    let mut result: Vec<SourceFileInfo> = Vec::with_capacity(file_infos.len());
    for file_info in file_infos {
        let row = sqlx::query_as::<_, SourceFileInfo>(
            r#"INSERT INTO FILE_INFO (id, workspace, file_path, file_tyep, exe_time) VALUES (?,?,?,?) "#)
            .bind(file_info.id)
            .bind(file_info.workspace)
            .bind(file_info.file_path)
            .bind(file_info.file_type)
            .bind(file_info.exe_time)
            .fetch_one(pool)
            .await?;    
        result.push(row);
    }
    transaction.commit().await?;
    Ok(result)
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<SourceFileInfo>, DBError> {
    let work_space = sqlx::query_as::<_, SourceFileInfo>("SELECT * FROM FILE_INFO WHERE WORKSPACE = ?")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}