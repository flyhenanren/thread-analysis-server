use actix_web::{web, App, HttpServer};
use db_access::db::*;
use fern::Dispatch;
use std::io;
use dotenv::dotenv;
use log::info;

#[path = "../handlers/mod.rs"]
mod handlers;
#[path = "../routers.rs"]
mod routers;
#[path = "../state.rs"]
mod state;
#[path ="../file/mod.rs"]
mod file;
#[path ="../models/mod.rs"]
mod models;
#[path ="../error.rs"]
mod error;
#[path = "../common/mod.rs"]
mod common;

#[path = "../service/mod.rs"]
mod service;

#[path = "../db_access/mod.rs"]
mod db_access;

use routers::*;
use state::AppState;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    setup_logger(true, "webserver.log").unwrap();
    info!("Application started");
    dotenv().ok();
    // let _url = env::var("DATABASE_URL").expect("找不到环境变量中的信息");
    let pool: sqlx::Pool<sqlx::Sqlite> = establish_connection().await;
    // 引入数据库

    // 初始化共享数据
    let shared_data = web::Data::new(AppState {
        pool
    });

    let app = move || {
        App::new()
            .app_data(shared_data.clone()) // 将数据绑定到内存中
            .configure(general_routers)
            .configure(file_routes)
    };
    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}


pub fn setup_logger(log_to_console: bool, log_file: &str) -> Result<(), fern::InitError> {
    let base_config = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {}-{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.file().unwrap_or("unknown file"),
                record.line().unwrap_or(0),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info); // 默认日志级别

    let file_config = base_config.chain(fern::log_file(log_file)?); // 输出到文件
    let logger = if log_to_console {
        file_config.chain(std::io::stdout()) // 输出到控制台
    } else {
        file_config
    };

    logger.apply()?; // 应用日志配置
    Ok(())
}