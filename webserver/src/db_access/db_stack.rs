use futures::future::join_all; // 在文件顶部添加这个导入

use std::sync::Arc;

use chrono::Utc;
use sqlx::{SqlitePool, Transaction};
use tokio::sync::mpsc::{self, Sender, Receiver};

use crate::{error::DBError, ThreadStack};

pub async fn batch_add(pool: &SqlitePool, threads: &Vec<ThreadStack>) -> Result<(), DBError> {
    let _ = adjust_config(pool).await;
    let (tx, rx): (Sender<ParamValues<ThreadStack>>, Receiver<ParamValues<ThreadStack>>) = mpsc::channel(100);
   
    let cpu_count = num_cpus::get();
    println!("cpu:{},rows:{}", cpu_count, threads.len());
    let arc_pool = Arc::new(pool.clone());
    let each_producer_count = (threads.len() / cpu_count) as usize;
    let reminder = (threads.len() % cpu_count) as usize;
    let mut handlers = Vec::with_capacity(cpu_count);
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
            producer(thread_tx, i.try_into().unwrap(), pool_clone, vec).await;
        }))
    }
    // 消费者任务
    let consumer_task = tokio::spawn(async move {
        consumer(rx, execute_batch_add).await;
    });
    // 等待所有生产者任务完成
    for handler in handlers {
        handler.await.unwrap();
    }
    drop(tx);
    consumer_task.await.unwrap();
    Ok(())
}


enum ParamValues<T> {
    WithValue(Arc<SqlitePool>, Vec<T>, i32),
}

async fn consumer<T, F>(mut rx: Receiver<ParamValues<T>>, execute_fn: F) 
where 
    T: Send + 'static,
    F: Fn(Arc<SqlitePool>, i32, Vec<T>) -> tokio::task::JoinHandle<Result<(), DBError>> + Send + Sync + 'static,
{
    let mut tasks: Vec<tokio::task::JoinHandle<Result<(), DBError>>> = Vec::new();

    while let Some(param_value) = rx.recv().await {
        match param_value {
            ParamValues::WithValue(pool, vec, cpu) => {
               println!("consumer:{}", Utc::now().timestamp_millis());
               let handler = execute_fn(pool, cpu, vec);
               tasks.push(handler);
            }
        }
    }
    let _ = join_all(tasks).await;
}

async fn producer<T>(tx: Sender<ParamValues<T>>,cpu: i32, pool: Arc<SqlitePool>, rows: Vec<T>)
where
    T: Send + 'static,
{
    println!("producer");
    let _ = tx.send(ParamValues::WithValue(pool, rows, cpu)).await;
}


fn execute_batch_add(pool: Arc<SqlitePool>, cpu: i32, threads: Vec<ThreadStack>) -> tokio::task::JoinHandle<Result<(), DBError>> {
    tokio::spawn(async move {
        let start = Utc::now().timestamp_millis();
        println!("start,cput:{},start:{}", cpu, start);
        const BATCH_SIZE:usize = 5000;
        for chunk in threads.chunks(BATCH_SIZE){
            let mut transaction: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;
            let insert_sql = String::from(r#"INSERT INTO THREAD_STACK (ID, WORKSPACE, THREAD_ID, CLASS_NAME, METHOD_NAME, STACK_LINE, STACK_STATUS)
                VALUES (?,?,?,?,?,?,?)"#);
            for stack in chunk {
                sqlx::query(&insert_sql)
                .bind(stack.id.clone())
                .bind(stack.work_space.clone())
                .bind(stack.thread_id.clone())
                .bind(stack.class_name.clone())
                .bind(stack.method_name.clone())
                .bind(stack.stack_lin)
                .bind(stack.stack_status.clone())
                .execute(&mut *transaction)
                .await?;
            }
            transaction.commit().await?;
        }
        println!("end_add:{}", Utc::now().timestamp_millis() - start);
        Ok(())
    })
}

async fn adjust_config(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA synchronous = OFF;").execute(pool).await?;
    sqlx::query("PRAGMA journal_mode = OFF;").execute(pool).await?;
    sqlx::query("PRAGMA temp_store = MEMORY;").execute(pool).await?;
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


