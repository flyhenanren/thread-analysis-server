use crate::{error::AnalysisError, files::*, state::AppState};
use actix_web::{web, HttpResponse};
use index::{FileIndex, StackIndex, ThreadsIndex};

pub async fn query_stack(
    app_state: web::Data<AppState>,
    file_name: String,
) -> Result<HttpResponse, AnalysisError> {
    let path = app_state.path.lock().unwrap();
    if path.clone().is_empty() {
        return Ok(HttpResponse::Ok().json("请先选择需要解析的文件或文件夹"));
    }
    // 读取索引并处理
     ThreadsIndex::read_index(&path)
     .map_err(|_| AnalysisError::DBError("索引错误".to_string()))  // 处理索引读取错误
     .and_then(|files| {
         // 找到对应文件，如果没有找到则返回错误
         files
             .into_iter()
             .find(|file| file.file_name == file_name)
             .ok_or_else(|| AnalysisError::NotFound("未找到匹配的文件".to_string()))
     })
     .and_then(|file| {
         // 根据文件的行号读取堆栈数据
         StackIndex::read_index_by_line(&path, file.start_line, file.end_line)
             .map_err(|err| AnalysisError::DBError(format!("读取索引失败:{}", err)))
     })
     .map(|stack_data| {
         // 返回成功的 HttpResponse
         HttpResponse::Ok().json(stack_data)
     })
     .map_err(|err| AnalysisError::DBError(format!("对象转换错误:{}", err)))
}