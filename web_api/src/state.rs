
use sqlx::Pool;
use task::async_task::TaskExecutor;

use crate::config::SharedConfig;


/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub pool: Pool<sqlx::Sqlite>,
    pub executor: TaskExecutor,
    pub shared_config: SharedConfig,
}
