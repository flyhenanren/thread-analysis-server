use common::error::DBError;
use futures::future::join_all; // 在文件顶部添加这个导入

use std::sync::Arc;

use sqlx::{SqlitePool, Transaction};
use tokio::sync::mpsc::{self, Sender, Receiver};


pub async fn batch_add(pool: &SqlitePool, threads: &Vec<String>) -> Result<(), DBError> {
    let _ = adjust_config(pool).await;
    let (tx, rx): (Sender<ParamValues<String>>, Receiver<ParamValues<String>>) = mpsc::channel(100);
   
    let cpu_count = num_cpus::get();
    let arc_pool = Arc::new(pool.clone());
    let each_producer_count = (threads.len() / cpu_count) as usize;
    let reminder = (threads.len() % cpu_count) as usize;
    let mut handlers = Vec::with_capacity(cpu_count);
    // 消费者任务
    let consumer_task = tokio::spawn(async move {
        consumer(rx, execute_batch_add).await;
    });

    for i in 0..cpu_count {
        let start = i * each_producer_count;
        let mut end = start + each_producer_count;
        if i == cpu_count - 1 {
            end += reminder;
        }
        let vec = threads[start..end].to_vec();
        let thread_tx = tx.clone();
        let pool_clone = arc_pool.clone();
        handlers.push(tokio::spawn(async move  {
            producer(thread_tx, pool_clone, vec).await;
        }))
    }
  
    // 等待所有生产者任务完成
    for handler in handlers {
        handler.await.unwrap();
    }
    drop(tx);
    consumer_task.await.unwrap();
    Ok(())
}


enum ParamValues<T> {
    WithValue(Arc<SqlitePool>, Vec<T>),
}

async fn consumer<T, F>(mut rx: Receiver<ParamValues<T>>, execute_fn: F) 
where 
    T: Send + 'static,
    F: Fn(Arc<SqlitePool>, Vec<T>) -> tokio::task::JoinHandle<Result<(), DBError>> + Send + Sync + 'static,
{
    let mut tasks: Vec<tokio::task::JoinHandle<Result<(), DBError>>> = Vec::new();

    while let Some(param_value) = rx.recv().await {
        match param_value {
            ParamValues::WithValue(pool, vec) => {
               let handler = execute_fn(pool,vec);
               tasks.push(handler);
            }
        }
    }
    let _ = join_all(tasks).await;
}

async fn producer<T>(tx: Sender<ParamValues<T>>,pool: Arc<SqlitePool>, rows: Vec<T>)
where
    T: Send + 'static,
{
    let _ = tx.send(ParamValues::WithValue(pool, rows)).await;
}


fn execute_batch_add(pool: Arc<SqlitePool>, threads: Vec<String>) -> tokio::task::JoinHandle<Result<(), DBError>> {
    tokio::spawn(async move {
        let mut transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
        let insert_sql = String::from(r#"INSERT INTO THREAD_STACK (ID, WORKSPACE, THREAD_ID, CLASS_NAME, METHOD_NAME, METHOD_LINE, STACK_STATUS)
            VALUES (?,?,?,?,?,?,?)"#);
        for stack in threads {
            sqlx::query(&insert_sql)
            .bind(stack)
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    })
}

async fn adjust_config(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA synchronous = OFF;").execute(pool).await?;
    sqlx::query("PRAGMA journal_mode = OFF;").execute(pool).await?;
    sqlx::query("PRAGMA temp_store = MEMORY;").execute(pool).await?;
    Ok(())
}
