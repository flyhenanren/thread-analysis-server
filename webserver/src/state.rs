
use sqlx::Pool;

use crate::task::async_task::TaskExecutor;

/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub pool: Pool<sqlx::Sqlite>,
    pub executor: TaskExecutor
}
