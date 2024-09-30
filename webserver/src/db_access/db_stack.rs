use std::{ sync::mpsc::{self, Receiver, Sender}, thread as stdThread};
use sqlx::{SqlitePool, Transaction};

use crate::{error::DBError, ThreadStack};

pub async fn batch_add(pool: &SqlitePool, threads: &Vec<ThreadStack>) -> Result<(), DBError> {
    let (tx, rx): (Sender<ThreadStack>, Receiver<ThreadStack>) = mpsc::channel();
    let consumer_handler = stdThread::spawn(|| consumer(rx));
    let cpu_count = num_cpus::get();
    let total_rows = threads.len();
    let each_producer_count = (total_rows / cpu_count) as i64;
    let remainder = (total_rows % cpu_count) as i64;
    let mut handlers = Vec::with_capacity(cpu_count);
    for i in 0..cpu_count{
       let thread_tx = tx.clone();
    //    let run_threads = Vec::with_capacity(each_producer_count as usize);
       if i == cpu_count - 1 {
        
       }else {

       }
       handlers.push(stdThread::spawn(move || {
           producer(thread_tx, each_producer_count.clone())
       }))
    }
    for t in  handlers{
       t.join().unwrap();
    }
    drop(tx);
    consumer_handler.join().unwrap();
    Ok(())
}


fn consumer(rx: Receiver<ThreadStack>) {
    let mut count = 0;
    println!("consumer");
    for paramValue in rx {
      count += 1;
    //   match paramValue {
        //   ParamValues::WithArea(vec) => println!("consumer count:{}, value:{}", count, vec[0].0),
    //   }
    }
  }
  static MIN_BATCH_SIZE:i64 = 10;
  fn producer(tx: Sender<ThreadStack>, count: i64) {
    if count < MIN_BATCH_SIZE {
        panic!("count size can't min 50");
    }
    println!("producer_count:{}", count);
    for i in 0..(count/MIN_BATCH_SIZE) {
      println!("producer_send,current:{}", i);
    //   tx.send(ParamValues::WithArea(vec![(i.to_string(), i)]));
    }
  }


pub async fn batch_add_copy(pool: &SqlitePool, threads: &Vec<ThreadStack>) -> Result<(), DBError> {
    const BATCH_SIZE:usize = 5000;
    let _ = adjust_config(pool).await;
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


async fn adjust_config(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA synchronous = OFF;").execute(pool).await?;
    sqlx::query("PRAGMA journal_mode = OFF;").execute(pool).await?;
    sqlx::query("PRAGMA temp_store = MEMORY;").execute(pool).await?;
    Ok(())
}