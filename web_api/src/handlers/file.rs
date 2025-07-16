use actix_web::{web, HttpResponse};
use common::{error::AnalysisError, string_utils::rand_id};

use crate::{ executor::file_analysis_task::ParseFile, resp::ApiResponse, service::file_service, state::AppState};

/**
 * 加载文件
 * 加载的文件
 */
pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    path: String
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::exist_work_space(&app_state.pool, &path).await {
        Ok(exist) => {
            match exist {
                true => Ok(HttpResponse::Ok().json(ApiResponse::ok())),
                false => {
                    let task_id = rand_id();
                    app_state.executor.submit_task(&task_id, ParseFile, Some(app_state.pool.clone()), Some(path.to_string())).await;
                    Ok(HttpResponse::Ok().json(ApiResponse::success(Some(task_id.to_string()))))
                }
            }
        },
        Err(err) => Err(AnalysisError::ActixError(format!("工作空间查询异常：{}",err))),
    }
}


pub async fn list_work_space(
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::list_work_space(&app_state.pool).await {
        Ok(work_space) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(work_space)))),
        Err(err) => Err(AnalysisError::ActixError(format!("工作空间查询异常：{}",err))),
    }
}

pub async fn clean_open_file(
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::clean_work_space(&app_state.pool).await {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::ok())),
        Err(err) => Err(AnalysisError::ActixError(format!("清理工作空间异常：{}", err))),
    }
}
