use crate::{db_access::db_stack::*, handlers::{file::*, general::*, thread::*}};
use actix_web::web;


pub fn general_routers(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check_handler));
}

pub fn file_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/file")
            .route("/open", web::post().to(load_file_handler))
            .route("/list", web::get().to(list_work_space))
            .route("/clean", web::get().to(clean_open_file))
            // .route("/{teacher_id}", web::get().to(get_course_for_teacher))
            // .route("/{teacher_id}/{course_id}", web::get().to(get_course_detail))
            // .route("/{teacher_id}/{course_id}", web::delete().to(delete_course))
            // .route("/{teacher_id}/{course_id}", web::put().to(update_coruse_detail)),
    )
    .service(
        web::scope("/dump")
            .route("/list/{work_space_id}", web::get().to(list_dump_handler))
            .route("/query", web::post().to(query_stack))
            .route("/count_file_status", web::post().to(count_file_status))
            .route("/count_thread_status", web::post().to(count_thread_status))
            .route("/list_thread_pool/{file_id}", web::get().to(count_file_threads))
    );
}