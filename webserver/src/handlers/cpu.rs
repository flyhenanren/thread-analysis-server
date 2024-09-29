use crate::{db_access::db_cpu, error::AnalysisError, file::*, state::AppState};
use actix_web::{web, HttpResponse};

pub async fn query_cpu(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    let cpu = db_cpu::list(&app_state.pool, &path).await?;
    Ok(HttpResponse::Ok().json(cpu))
}