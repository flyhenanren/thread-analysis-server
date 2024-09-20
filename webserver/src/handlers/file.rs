use actix_web::{web, HttpResponse};
use crate::{files::*, state::AppState};

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


