use actix_web::{web, App, HttpServer};

use domain::{config::{AppConfig, SharedConfig}, context::Context, db::db::establish_connection};
use task::async_task::TaskExecutor;
use std::{io, net::ToSocketAddrs};
use log::*;

use crate::{logger::setup_logger, router::{file_routes, general_routers}, state::{AppState}};

#[path = "../state.rs"]
mod state;
#[path = "../router/routers.rs"]
mod router;
#[path = "../handlers/mod.rs"]
mod handlers;
#[path = "../resp.rs"]
mod resp;
#[path = "../service/mod.rs"]
mod service;
#[path = "../task/mod.rs"]
mod executor;

#[path = "../model/mod.rs"]
mod model;
#[path = "../logger.rs"]
mod logger;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    // 读取配置
    let default_cfg = AppConfig::from_env();
    let log_cfg = default_cfg.log.clone();
    // 读取实时配置
    let shared_config = SharedConfig::new(default_cfg);
    setup_logger(log_cfg).unwrap();
    let cfg = shared_config.get();
    // 引入数据库
    let pool: sqlx::Pool<sqlx::Sqlite> = establish_connection().await;
    // 异步任务执行器
    let executor: TaskExecutor = TaskExecutor::new();
    let context = Context {
        pool,
        shared_config,
    };
    // 初始化共享数据
    let shared_data = web::Data::new(AppState {
        context,
        executor,
    });
    let app = move || {
        App::new()
            .app_data(shared_data.clone()) // 将数据绑定到内存中
            .configure(general_routers)
            .configure(file_routes)
    };
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let socket_addr = addr
                                    .to_socket_addrs()?
                                    .next()
                                    .expect("Invalid address");
    info!("Application started");
    HttpServer::new(app).bind(socket_addr)?.run().await
}


