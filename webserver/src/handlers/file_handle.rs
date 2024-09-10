use actix_web::{web, HttpResponse};

use crate::{files::file_analysis, state::AppState};

pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    path: String
) -> HttpResponse {
    println!("path:{}", path);
    file_analysis::analysis(&path);
    let mut state_path = app_state.path.lock().unwrap();
    *state_path = path;
    HttpResponse::Ok().json("OK")
}

pub async fn list_file_handler(
    app_state: web::Data<AppState>
) -> HttpResponse {
    let path = app_state.path.lock().unwrap();
    HttpResponse::Ok().json(path.clone())
}
