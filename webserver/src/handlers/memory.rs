use crate::{db_access::db_memeory, error::AnalysisError, file::parse::*, state::AppState};
use actix_web::{web, HttpResponse};

pub async fn query_memeory(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    let memrory = db_memeory::list(&app_state.pool, &path).await?;
    Ok(HttpResponse::Ok().json(memrory))
}