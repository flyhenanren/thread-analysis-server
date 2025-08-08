use domain::{db::{self, db_thread::{self, DBThreadInfo}}, model::{stack::CallTree, thread::Thread}};
use indexer::cache::global::{CacheKey, GlobalCache};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use task::async_task::{AsyncTask, ExecuteContext};


pub struct BuildCacheAsyncTask;

#[async_trait::async_trait]
impl AsyncTask for BuildCacheAsyncTask{
    async fn execute(&self, context: &ExecuteContext) -> Result<String, String> {
        let pool = context.pool.as_ref().ok_or("数据库连接池缺失")?;
        let workspace = context.param.as_ref().ok_or("err")?;
        context.update_progress(0.1,Some("开始构建缓存".to_string())).await;
        match db_thread::list_by_work_space(pool, &workspace).await{
            Ok(threads) => threads_build(workspace, threads),
            Err(err) => log::error!("查询线程信息时发生错误:{:?}", err),
        }
        context.update_progress(1.0, Some("缓存构建完成".to_string())).await;
        Ok("".to_string())
    }


    
}

fn threads_build(workspace: &str, db_threads: Vec<DBThreadInfo>){
    let mut threads = Vec::new();
    for db_thread in db_threads {
        threads.push(db_thread.to_thread());
    }
    let call_tree = CallTree::new(threads);
    GlobalCache::put(CacheKey::call_tree(workspace), call_tree);
    ()
}