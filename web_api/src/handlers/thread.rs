use actix_web::{web, HttpResponse};
use common::error::AnalysisError;
use domain::model::thread::{StatusQuery, ThreadsQuery};

use crate::{resp::ApiResponse, service::{file_service, thread_dump}, state::AppState};

pub async fn list_dump_handler(
    app_state: web::Data<AppState>,
    work_space_id: web::Path<String>,
) -> Result<HttpResponse, AnalysisError> {
    match file_service::list_dump_file(&app_state.context.pool, &work_space_id).await{
        Ok(files) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(files)))),
        Err(err) => Ok(HttpResponse::Ok().json(ApiResponse::error(201, &format!("{:?}",err))))
    }
}

pub async fn query_threads(
    app_state: web::Data<AppState>,
    query_info: web::Json<ThreadsQuery>,
) -> Result<HttpResponse, AnalysisError> {
    // 读取索引并处理
    thread_dump::get_thread_detail(&app_state.context.pool, &query_info).await
        .map(|threads_data| {
            // 返回成功的 HttpResponse
            HttpResponse::Ok().json(ApiResponse::success(Some(threads_data)))
        })
        .map_err(|err| AnalysisError::DBError(format!("对象转换错误:{}", err)))
}

pub async fn count_file_threads(
    app_state: web::Data<AppState>,
    file_id: web::Path<String>,
) -> Result<HttpResponse, AnalysisError> {
  
    thread_dump::count_status_by_file(&app_state.context.pool, &file_id).await
        .map(|stack_data| {
            // 返回成功的 HttpResponse
            HttpResponse::Ok().json(ApiResponse::success(Some(stack_data)))
        })
        .map_err(|err| AnalysisError::DBError(format!("对象转换错误:{}", err)))
}


pub async fn count_file_status(app_state: web::Data<AppState>,
    count_query: web::Json<StatusQuery>) -> Result<HttpResponse, AnalysisError> {
        match thread_dump::count_status_by_files(&app_state.context.pool, &count_query).await {
            Ok(status_counts) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(status_counts)))),
            Err(err) => Err(AnalysisError::DBError(format!("对象转换错误:{}", err))),
        }
}


pub async fn count_thread_status(app_state: web::Data<AppState>,
    count_query: web::Json<StatusQuery>) -> Result<HttpResponse, AnalysisError> {
        match thread_dump::count_status_by_thread(&app_state.context.pool, &count_query).await {
            Ok(status_counts) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(status_counts)))),
            Err(err) => Err(AnalysisError::DBError(format!("对象转换错误:{}", err))),
        }
}

pub async fn get_thread_content(app_state: web::Data<AppState>,
    thread_id: web::Path<String>,) -> Result<HttpResponse, AnalysisError> {
        match thread_dump::get_thread_content(&app_state.context.pool, &thread_id).await {
            Ok(status_counts) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(status_counts)))),
            Err(err) => {
                    Ok(HttpResponse::Ok().json(ApiResponse::error(201, format!("执行失败:{}", err).as_str())))
            }
                
        }
}

