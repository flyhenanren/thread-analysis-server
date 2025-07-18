use actix_web::{web, HttpResponse};
use common::{error::AnalysisError, string_utils::rand_id};

use crate::{ executor::file_prase::ParseFileAsyncTask, resp::ApiResponse, service::file_service, state::AppState};

/// 将文件内容解析为工作空间，并存储到数据库中
/// # Arguments
/// * `app_state` - 应用状态，包含数据库连接池和其他共享资源
/// * `path` - 文件路径，可能是压缩文件或普通文件
/// # Returns
/// * `Result<HttpResponse, AnalysisError>` - 返回 HTTP 响应，包含操作结果
/// # Errors
/// * 如果文件解析失败，或者在查询工作空间时发生错误，将返回 `AnalysisError`
/// # Example
/// ```rust
/// let app_state = AppState::new(pool, channel);
/// let path = "path/to/file_or/directory/dump.zip";
/// let response = load_file_handler(app_state, path).await;
/// ```
/// # Note
/// 此函数会检查工作空间是否已存在，如果不存在，则提交一个异步任务来解析文件。
/// 如果工作空间已存在，则直接返回成功响应。
/// # Panics
/// 如果在解析文件或查询工作空间时发生错误，将触发 panic。
/// # Asynchronous
/// 此函数是异步的，使用 `async` 和 `await` 语法
pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    path: String
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::exist_work_space(&app_state.context.pool, &path).await {
        Ok(exist) => {
            match exist {
                true => Ok(HttpResponse::Ok().json(ApiResponse::ok())),
                false => {
                    let task_id = rand_id();
                    app_state.executor.submit_task(&task_id, ParseFileAsyncTask, &app_state.context, Some(path.to_string())).await;
                    Ok(HttpResponse::Ok().json(ApiResponse::success(Some(task_id.to_string()))))
                }
            }
        },
        Err(err) => Ok(HttpResponse::Ok().json(ApiResponse::error(500, &format!("查询工作空间异常：{}", err))))
    }
}

/// 列出所有工作空间
/// # Arguments 
/// * `app_state` - 应用状态，包含数据库连接池和其他共享资源
/// # Returns
/// * `Result<HttpResponse, AnalysisError>` - 返回 HTTP 响应，包含工作空间列表
/// # Errors
/// * 如果查询工作空间时发生错误，将返回 `AnalysisError`
/// # Example
/// ```rust
/// let app_state = AppState::new(pool, channel);
/// let response = list_work_space(app_state).await;
/// ```
/// # Note
/// 此函数会从数据库中查询所有工作空间，并返回它们的列表。
/// 如果没有工作空间，则返回一个空列表。
/// # Panics
/// 如果在查询工作空间时发生错误，将触发 panic。
/// # Asynchronous
/// 此函数是异步的，使用 `async` 和 `await` 语法
pub async fn list_work_space(
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::list_work_space(&app_state.context.pool).await {
        Ok(work_space) => Ok(HttpResponse::Ok().json(ApiResponse::success(Some(work_space)))),
        Err(err) => Ok(HttpResponse::Ok().json(ApiResponse::error(500, &format!("查询工作空间异常：{}", err))))
    }
}


/// 清理打开的工作空间
/// # Arguments
///     * `app_state` - 应用状态，包含数据库连接池和其他共享资源
/// # Returns
/// * `Result<HttpResponse, AnalysisError>` - 返回 HTTP 响应，表示清理结果
/// # Errors
/// * 如果清理工作空间时发生错误，将返回 `AnalysisError`
/// # Example
/// ```rust
/// let app_state = AppState::new(pool, channel);
/// let response = clean_open_file(app_state).await;
/// ```
/// # Note
/// 此函数会清理所有打开的工作空间，并释放相关资源。
/// 如果清理过程中发生错误，将返回错误信息。
/// # Panics
/// 如果在清理工作空间时发生错误，将触发 panic。
/// # Asynchronous
/// 此函数是异步的，使用 `async` 和 `await` 语法
pub async fn clean_open_file(
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::clean_work_space(&app_state.context.pool).await {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::ok())),
        Err(err) => Ok(HttpResponse::Ok().json(ApiResponse::error(500, &format!("清理工作空间异常：{}", err))))
    }
}
