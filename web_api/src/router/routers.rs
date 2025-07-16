
use actix_web::web;

use crate::handlers::{async_task::query_task_process, cpu::cpu_used_count, file::{clean_open_file, list_work_space, load_file_handler}, general::health_check_handler, thread::{count_file_status, count_file_threads, count_thread_status, get_thread_content, list_dump_handler, query_threads}};


pub fn general_routers(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check_handler));
}

pub fn file_routes(cfg: &mut web::ServiceConfig) {
    cfg
    .service(web::scope("/task")
                        .route("/query_process/{task_id}", web::get().to(query_task_process)))
    .service(
        web::scope("/file")
            .route("/open", web::post().to(load_file_handler))
            .route("/list", web::get().to(list_work_space))
            .route("/clean", web::get().to(clean_open_file))
    )
    .service(
        web::scope("/dump")
            .route("/list/{work_space_id}", web::get().to(list_dump_handler))
            .route("/count_file_status", web::post().to(count_file_status))
            .route("/count_thread_status", web::post().to(count_thread_status))
            .route("/list_thread_pool/{file_id}", web::get().to(count_file_threads))
    )
    .service(
        web::scope("/thread")
            .route("/query", web::post().to(query_threads))
            .route("/content/{thread_id}", web::get().to(get_thread_content))
    )
    .service(
        web::scope("/cpu")
            .route("/count_info/{workspace_id}", web::get().to(cpu_used_count))
            
    )
    ;
}