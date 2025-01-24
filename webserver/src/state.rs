use std::sync::Mutex;

use sqlx::Pool;

/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub pool: Pool<sqlx::Sqlite>
}
