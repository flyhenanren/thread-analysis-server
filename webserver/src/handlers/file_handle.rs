use actix_web::{web, HttpResponse};

use crate::{files::zip_analysis, state::AppState};

pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    params: web::Path<String>,
) -> HttpResponse {
    let path = params.into_inner();
    let file_info =zip_analysis::analysis(&path);
    HttpResponse::Ok().json("value")
}
