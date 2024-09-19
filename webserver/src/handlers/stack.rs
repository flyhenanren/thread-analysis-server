use actix_web::{web, HttpResponse};
use index::{FileIndex, ThreadsIndex};
use crate::{error::AnalysisError, files::*, state::AppState};

pub async fn query_stack(
  app_state: web::Data<AppState>,
  file_name: web::Json<String>,
) -> Result<HttpResponse, AnalysisError> {
  let path = app_state.path.lock().unwrap();
  if path.clone().is_empty() {
      return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
  }
  ThreadsIndex::read_index(&path.clone())
  .map(|files| {
    files
      .iter()
      .find(|file| file.file_name == *file_name)
      .map_or_else(
        || HttpResponse::Ok().json("文件未找到"),
        |file| HttpResponse::Ok().json(file)
      )
  })
  .map_err(|_| AnalysisError::DBError("数据查询错误".to_string()))

}