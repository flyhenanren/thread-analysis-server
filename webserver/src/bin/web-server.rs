use actix_web::{web, App, HttpServer};
use std::io;
use std::sync::Mutex;

#[path = "../handlers/mod.rs"]
mod handlers;
#[path = "../routers.rs"]
mod routers;
#[path = "../state.rs"]
mod state;
#[path ="../files/mod.rs"]
mod files;

use routers::*;
use state::AppState;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    // dotenv().ok();
    // let _url = env::var("DATABASE_URL").expect("找不到环境变量中的信息");

    // 引入数据库

    // 初始化共享数据
    let shared_data = web::Data::new(AppState {
        health_check_response: "I'm Ok.".to_string(),
        visit_count: Mutex::new(0),
    });

    let app = move || {
        App::new()
            .app_data(shared_data.clone()) // 将数据绑定到内存中
            .configure(general_routers)
        // .configure(course_routes)
    };

    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
