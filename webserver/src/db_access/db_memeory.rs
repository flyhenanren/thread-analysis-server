use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, MemoryInfo};

pub async fn batch_add(pool: &SqlitePool, mem_infos: &Vec<MemoryInfo>) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for mem_info in mem_infos {
        sqlx::query(
            r#"INSERT INTO MEMORY_INFO (ID, WORKSAPCE, MEMORY_INFO, S0, S0C, S0U, S1C, S1U, EC, EU, OC, OU, MC, MU, CCSC, CCSU, YGC, YGCT, FGC, FGCT, CGC, CGCT, GCT, EXE_TIME)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#)
             .bind(mem_info.id.clone())
             .bind(mem_info.workspace.clone())
            .bind(mem_info.file_id.clone())
            .bind(mem_info.s0)
            .bind(mem_info.s0c)
            .bind(mem_info.s0u)
            .bind(mem_info.s1c)
            .bind(mem_info.s1u)
            .bind(mem_info.ec)
            .bind(mem_info.eu)
            .bind(mem_info.oc)
            .bind(mem_info.ou)
            .bind(mem_info.mc)
            .bind(mem_info.mu)
            .bind(mem_info.ccsc)
            .bind(mem_info.ccsu)
            .bind(mem_info.ygc)
            .bind(mem_info.ygct)
            .bind(mem_info.fgc)
            .bind(mem_info.fgct)
            .bind(mem_info.cgc)
            .bind(mem_info.cgct)
            .bind(mem_info.gct)
            .bind(mem_info.exe_time)
            .execute(pool)
            .await?;    
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space: &str) -> Result<Vec<MemoryInfo>, DBError> {
    let work_space = sqlx::query_as::<_, MemoryInfo>("SELECT * FROM MEMORY_INFO where WORKSPACE = ?")
        .bind(work_space)
        .fetch_all(pool)
        .await?;
    Ok(work_space)
}

