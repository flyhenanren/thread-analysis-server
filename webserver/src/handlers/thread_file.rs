use actix_web::{web, HttpResponse};

use crate::{files::file_handler, state::AppState};

pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    params: web::Path<String>,
) -> HttpResponse {
    let path = params.into_inner();
    file_handler::unzip_file(&path);
    HttpResponse::Ok().json("value")
}
