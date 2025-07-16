use actix_web::{web, HttpResponse};
use common::error::AnalysisError;

use crate::{resp::ApiResponse, state::AppState};


/**
 * 异步任务执行进度查询
 */
pub async fn query_task_process(
  app_state: web::Data<AppState>,
  process_id: web::Path<String>,
) -> Result<HttpResponse, AnalysisError> {
  match app_state.executor.get_task_status(&process_id).await{
    Some(status) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(status)))),
    None => Ok(HttpResponse::Ok().json(ApiResponse::success(Some("none".to_string())))),
  }
  
}
