
use actix_web::{web, HttpResponse};
use common::error::AnalysisError;
use db::db_access::db_cpu;

use crate::{service::cpu_service, state::AppState};

#[allow(dead_code)]
pub async fn query_cpu(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
) -> Result<HttpResponse, AnalysisError> {
    let cpu = db_cpu::list(&app_state.pool, &path).await?;
    Ok(HttpResponse::Ok().json(cpu))
}
#[allow(dead_code)]
pub async fn cpu_used_count(
    app_state: web::Data<AppState>,
    workspace_id: web::Path<String>
) -> Result<HttpResponse, AnalysisError> {
    match cpu_service::count_cpu_status(&app_state.pool, &workspace_id).await {
        Ok(result) =>  Ok(HttpResponse::Ok().json(result)),
        Err(err) => Err(AnalysisError::DBError(format!("对象转换错误:{}", err))),
    }
   
}