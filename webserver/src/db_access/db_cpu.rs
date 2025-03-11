use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, CpuInfo};

pub async fn batch_add(pool: &SqlitePool, cpu_infos: Vec<CpuInfo>, work_space: &str) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for info in cpu_infos {
        sqlx::query(
            r#"INSERT INTO CPU_INFO (id,workspace, exe_time, us, sy, ids, tasks, running, sleeping, mem_total, mem_free, mem_used)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?) "#)
             .bind(info.id)
             .bind(work_space)
            .bind(info.exe_time)
            .bind(info.us)
            .bind(info.sy)
            .bind(info.ids)
            .bind(info.tasks)
            .bind(info.running)
            .bind(info.sleeping)
            .bind(info.mem_total)
            .bind(info.mem_free)
            .bind(info.mem_used)
            .execute(pool)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<CpuInfo>, DBError> {
    let work_space = sqlx::query_as::<_, CpuInfo>("SELECT * FROM CPU_INFO WHERE WORKSPACE = ? ")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}


pub async fn delete_all(pool: &SqlitePool) -> Result<(), DBError> {
    sqlx::query("DELETE FROM CPU_INFO")
        .execute(pool)
        .await?;
    Ok(())
}