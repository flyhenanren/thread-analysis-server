use actix_web::{web, HttpResponse};
use crate::{error::AnalysisError, files::*, state::AppState};

pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    path: String
) -> HttpResponse {
    println!("path:{}", path);
    file::analysis(&path);
    let mut state_path = app_state.path.lock().unwrap();
    *state_path = path;
    HttpResponse::Ok().json("OK")
}

pub async fn list_file_handler(
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    file::list_dump_file(&path.clone())
    .map(|files| HttpResponse::Ok().json(files))
}
