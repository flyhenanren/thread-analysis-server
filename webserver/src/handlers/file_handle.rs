use actix_web::{web, HttpResponse};

use crate::{files::zip_analysis, models::file_info::FileInfo, state::AppState};

pub async fn load_file_handler(
    path: String
) -> HttpResponse {
    println!("path:{}", path);
    let file_info =zip_analysis::analysis(&path).unwrap();
    HttpResponse::Ok().json(file_info)
}
