use task::async_task::{AsyncTask, ExecuteContext};

use crate::service::file_service;


pub struct ParseFile;

#[async_trait::async_trait]
impl AsyncTask for ParseFile{
 async fn execute(&self, context: &ExecuteContext) -> Result<String, String> {
        context.update_progress(0.1,Some("开始读取压缩包".to_string())).await;
         // 等待任务完成
        let pool = context.pool.as_ref().ok_or("失败")?;
        let path = context.param.as_ref().ok_or("err")?;
        match file_service::analysis(&pool, &path, context).await{
            Ok(result) => Ok(result),
            Err(err) => Err(err.to_string()),
        }
    }
}
