use std::{
    os::windows::thread,
    sync::mpsc::{self, Receiver, Sender},
    thread as stdThread,
};

use sqlx::{Executor, SqlitePool};

static MIN_BATCH_SIZE: i64 = 10;

enum ParamValues {
    WithArea(Vec<(String, i64)>),
}

fn consumer(rx: Receiver<ParamValues>) {
    let mut count = 0;
    println!("consumer");
    for paramValue in rx {
        count += 1;
        match paramValue {
            ParamValues::WithArea(vec) => println!("consumer count:{}, value:{}", count, vec[0].0),
        }
    }
}

fn producer(tx: Sender<ParamValues>, count: i64) {
    if count < MIN_BATCH_SIZE {
        panic!("count size can't min 50");
    }
    println!("producer_count:{}", count);
    for i in 0..(count / MIN_BATCH_SIZE) {
        println!("producer_send,current:{}", i);
        tx.send(ParamValues::WithArea(vec![(i.to_string(), i)]));
    }
}

fn run() {
    let (tx, rx): (Sender<ParamValues>, Receiver<ParamValues>) = mpsc::channel();
    let consumer_handler = stdThread::spawn(|| consumer(rx));
    //  let cpu_count = num_cpus::get();
    let cpu_count = 5;
    let total_rows = 161;
    let each_producer_count = (total_rows / cpu_count) as i64;
    let mut handlers = Vec::with_capacity(cpu_count);
    for _ in 0..cpu_count {
        let thread_tx = tx.clone();
        handlers.push(stdThread::spawn(move || {
            producer(thread_tx, each_producer_count.clone())
        }))
    }
    for t in handlers {
        t.join().unwrap();
    }
    drop(tx);
    consumer_handler.join().unwrap();
}

#[cfg(test)]
pub mod tests {
    use super::run;

    #[test]
    pub fn run_test() {
        // 假设有一个集合
        let data = (1..107).collect::<Vec<_>>(); // 集合长度为 104

        // 获取 CPU 核心数，假设为 4
        let cpu_count = 4;
        let mut count = 1;
        let each = (data.len() / cpu_count) as usize;
        let reminder = (data.len() % cpu_count) as usize;
        for ck in data[0..data.len() - reminder].chunks(each) {
            println!("ck:{}",count);
            count +=1;
            ck.iter().for_each(|c| println!("c: {}", c));
        }
    }
}
