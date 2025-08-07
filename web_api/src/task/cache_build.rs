use db::db_access::db_thread;
use domain::model::{stack::CallTree, thread::Thread};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use task::async_task::{AsyncTask, ExecuteContext};


pub struct BuildCacheAsyncTask;

#[async_trait::async_trait]
impl AsyncTask for BuildCacheAsyncTask{
    async fn execute(&self, context: &ExecuteContext) -> Result<String, String> {
        let pool = context.pool.as_ref().ok_or("数据库连接池缺失")?;
        let workspace = context.param.as_ref().ok_or("err")?;
        context.update_progress(0.1,Some("开始构建缓存".to_string())).await;
        let threads = db_thread::list_by_work_space(pool, &workspace)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|ele| ele.to_thread())
            .collect();
        let tree = CallTree::new(threads);
        Ok("".to_string())
    }
}

