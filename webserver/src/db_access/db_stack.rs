use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, ThreadStack};

pub async fn batch_add(pool: &SqlitePool, threads: &Vec<ThreadStack>) -> Result<(), DBError> {
    let transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
    for thread in threads {
        sqlx::query(
            r#"INSERT INTO THREAD_STACK (ID, WORKSPACE, THREAD_ID, CLASS_NAME, METHOD_NAME, STACK_LINE, STACK_STATUS)
            VALUES (?,?,?,?,?,?,?)"#)
        .bind(thread.id.clone())
        .bind(thread.work_space.clone())
        .bind(thread.thread_id.clone())
        .bind(thread.class_name.clone())
        .bind(thread.method_name.clone())
        .bind(thread.stack_lin)
        .bind(thread.stack_status.clone())
        .execute(pool)
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, work_space:&str) -> Result<Vec<ThreadStack>, DBError> {
    let stack = sqlx::query_as::<_, ThreadStack>("SELECT * FROM THREAD_STACK WHERE WORKSPACE = ?")
        .bind(work_space.clone())
        .fetch_all(pool)
        .await?;
    Ok(stack)
}

pub async fn get(pool: &SqlitePool, id: i32) -> Result<ThreadStack, DBError> {
    let work_sapce =
        sqlx::query_as::<_, ThreadStack>("SELECT * FROM THREAD_STACK WHERE ID = ?")
            .bind(id)
            .fetch_one(pool)
            .await?;
    Ok(work_sapce)
}
