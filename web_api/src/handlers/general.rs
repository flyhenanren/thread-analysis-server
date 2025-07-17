use crate::state::AppState;
use actix_web::{web, HttpResponse};

pub async fn health_check_handler(_app_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json("OK")
}
