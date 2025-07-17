use actix_web::{web, HttpResponse};
use common::error::AnalysisError;
use db::db_access::db_memeory;

use crate::state::AppState;

#[allow(dead_code)]
pub async fn query_memeory(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
) -> Result<HttpResponse, AnalysisError> {
    let memrory = db_memeory::list(&app_state.pool, &path).await?;
    Ok(HttpResponse::Ok().json(memrory))
}