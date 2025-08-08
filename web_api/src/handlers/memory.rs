use actix_web::{web, HttpResponse};
use common::error::AnalysisError;
use domain::db::db_memory;

use crate::state::AppState;

#[allow(dead_code)]
pub async fn query_memeory(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
) -> Result<HttpResponse, AnalysisError> {
    let memrory = db_memory::list(&app_state.context.pool, &path).await?;
    Ok(HttpResponse::Ok().json(memrory))
}