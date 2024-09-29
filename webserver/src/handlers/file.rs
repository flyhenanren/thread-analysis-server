use actix_web::{web, HttpResponse};
use crate::{error::AnalysisError, service::file_service, state::AppState};

pub async fn load_file_handler(
    app_state: web::Data<AppState>,
    path: String
) -> Result<HttpResponse, AnalysisError>  {
    match file_service::exist_work_space(&app_state.pool, &path).await {
        Ok(exist) => {
            let mut state_path = app_state.path.lock().unwrap();
            *state_path = path.clone();
            match exist {
                true => Ok(HttpResponse::Ok().json("OK")),
                false => {
                    let _ = file_service::analysis(&app_state.pool, &path).await?;
                    Ok(HttpResponse::Ok().json("OK"))
                }
            }
        },
        Err(err) => Err(AnalysisError::ActixError(format!("工作空间查询异常：{}",err))),
    }
}
