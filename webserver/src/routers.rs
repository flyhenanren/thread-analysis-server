use crate::handlers::{general::*, file_handle::*};
use actix_web::web;


pub fn general_routers(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check_handler));
}

pub fn file_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/thread")
            .route("/{file_path}", web::get().to(load_file_handler))
            // .route("/{teacher_id}", web::get().to(get_course_for_teacher))
            // .route("/{teacher_id}/{course_id}", web::get().to(get_course_detail))
            // .route("/{teacher_id}/{course_id}", web::delete().to(delete_course))
            // .route("/{teacher_id}/{course_id}", web::put().to(update_coruse_detail)),
    );
}
