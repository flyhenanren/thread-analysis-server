use std::path;

use crate::{error::AnalysisError, models::{resp::ApiResponse, thread::StatusQuery}, service::{file_service, thread_dump}, state::AppState};
use actix_web::{web, HttpResponse};

pub async fn list_dump_handler(
    app_state: web::Data<AppState>,
    work_space_id: web::Path<String>,
) -> Result<HttpResponse, AnalysisError> {
    let files = file_service::list_dump_file(&app_state.pool, &work_space_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(Some(files))))
}

pub async fn query_stack(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
    file_name: String,
) -> Result<HttpResponse, AnalysisError> {
    // 读取索引并处理
    thread_dump::get_thread_detail(&path.clone(), &file_name)
        .map(|stack_data| {
            // 返回成功的 HttpResponse
            HttpResponse::Ok().json(ApiResponse::success(Some(stack_data)))
        })
        .map_err(|err| AnalysisError::DBError(format!("对象转换错误:{}", err)))
}

pub async fn count_dumps_info(
    app_state: web::Data<AppState>,
    path: web::Json<String>,
    count_query: web::Json<StatusQuery>
) -> Result<HttpResponse, AnalysisError> {
    thread_dump::count_dumps_info(&path.clone(), &count_query)
    .map(|thread_count|  HttpResponse::Ok().json(ApiResponse::success(Some(thread_count))))
    .map_err(|err|  AnalysisError::DBError(format!("对象转换错误:{}", err)))
}

pub async fn count_threads_status(app_state: web::Data<AppState>,
    path: web::Json<String>,
    count_query: web::Json<StatusQuery>) -> Result<HttpResponse, AnalysisError> {
        thread_dump::count_threads_info(&path.clone(), &count_query)
        .map(|thread_count|  HttpResponse::Ok().json(ApiResponse::success(Some(thread_count))))
        .map_err(|err|  AnalysisError::DBError(format!("对象转换错误:{}", err)))
}
