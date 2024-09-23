use crate::{error::AnalysisError, files::*, models::thread::StatusQuery, service::thread_dump, state::AppState};
use actix_web::{web, HttpResponse};

pub async fn list_dump_handler(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    file::list_dump_file(&path.clone()).map(|files| HttpResponse::Ok().json(files))
}

pub async fn query_stack(
    app_state: web::Data<AppState>,
    file_name: String,
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    // 读取索引并处理
    thread_dump::get_thread_detail(&path.clone(), &file_name)
        .map(|stack_data| {
            // 返回成功的 HttpResponse
            HttpResponse::Ok().json(stack_data)
        })
        .map_err(|err| AnalysisError::DBError(format!("对象转换错误:{}", err)))
}

pub async fn count_dumps_info(
    app_state: web::Data<AppState>,
    count_query: web::Json<StatusQuery>
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    thread_dump::count_dumps_info(&path.clone(), &count_query)
    .map(|thread_count|  HttpResponse::Ok().json(thread_count))
    .map_err(|err|  AnalysisError::DBError(format!("对象转换错误:{}", err)))
}

pub async fn count_threads_status(app_state: web::Data<AppState>,
    count_query: web::Json<StatusQuery>) -> Result<HttpResponse, AnalysisError> {
        let path = app_state.path.lock().unwrap();
        if path.clone().is_empty() {
            return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
        }
        thread_dump::count_threads_info(&path.clone(), &count_query)
        .map(|thread_count|  HttpResponse::Ok().json(thread_count))
        .map_err(|err|  AnalysisError::DBError(format!("对象转换错误:{}", err)))
}
