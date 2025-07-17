use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;

pub async fn establish_connection() -> Pool<Sqlite> {
    let database_url = env::var("DATABASE_URL").expect("无法获取数据库链接");
    SqlitePool::connect(&database_url)
        .await
        .expect("无法连接数据库")
}