use crate::handlers::{general::*, file::*, stack::*};
use actix_web::web;


pub fn general_routers(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check_handler));
}

pub fn file_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/file")
            .route("/open", web::post().to(load_file_handler))
            .route("/list", web::get().to(list_file_handler))
            // .route("/{teacher_id}", web::get().to(get_course_for_teacher))
            // .route("/{teacher_id}/{course_id}", web::get().to(get_course_detail))
            // .route("/{teacher_id}/{course_id}", web::delete().to(delete_course))
            // .route("/{teacher_id}/{course_id}", web::put().to(update_coruse_detail)),
    )
    .service(
        web::scope("/stack")
            .route("/query", web::post().to(query_stack))
            
    );
}
