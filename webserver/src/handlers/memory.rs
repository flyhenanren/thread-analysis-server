use crate::{db_access::db_memeory, error::AnalysisError, file::parse::*, state::AppState};
use actix_web::{web, HttpResponse};

pub async fn query_memeory(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
) -> Result<HttpResponse, AnalysisError> {
    let memrory = db_memeory::list(&app_state.pool, &path).await?;
    Ok(HttpResponse::Ok().json(memrory))
}