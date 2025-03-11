use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{error::AnalysisError, models::resp::ApiResponse, state::AppState, task::{async_task::ExecuteContext, file_analysis_task::ParseFile}};


pub async fn query_task_process(
  app_state: web::Data<AppState>,
  process_id: web::Path<String>,
) -> Result<HttpResponse, AnalysisError> {
  match app_state.executor.get_task_status(&process_id).await{
    Some(status) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(status)))),
    None => Ok(HttpResponse::Ok().json(ApiResponse::success(Some("none".to_string())))),
  }
  
}
