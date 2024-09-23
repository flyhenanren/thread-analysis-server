use std::sync::Mutex;

use sqlx::Pool;

/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub path: Mutex<String>, 
    pub pool: Pool<sqlx::Sqlite>
}
