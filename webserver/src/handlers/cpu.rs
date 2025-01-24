use std::path;

use crate::{db_access::db_cpu, error::AnalysisError, file::*, state::AppState};
use actix_web::{web, HttpResponse};

pub async fn query_cpu(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
) -> Result<HttpResponse, AnalysisError> {
    let cpu = db_cpu::list(&app_state.pool, &path).await?;
    Ok(HttpResponse::Ok().json(cpu))
}