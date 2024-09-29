use actix_web::{web, App, HttpServer};
use db_access::db::*;
use std::io;
use std::sync::Mutex;
use dotenv::dotenv;

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
    dotenv().ok();
    // let _url = env::var("DATABASE_URL").expect("找不到环境变量中的信息");
    let pool: sqlx::Pool<sqlx::Sqlite> = establish_connection().await;
    // 引入数据库

    // 初始化共享数据
    let shared_data = web::Data::new(AppState {
        path: Mutex::new("D:\\dump\\20240726".to_string()),
        pool
    });

    let app = move || {
        App::new()
            .app_data(shared_data.clone()) // 将数据绑定到内存中
            .configure(general_routers)
            .configure(file_routes)
    };
    println!("server startup on 3000");
    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
