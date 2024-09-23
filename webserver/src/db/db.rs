use std::env;

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};

pub async fn establish_connection() -> Pool<Sqlite>{
  let database_url = env::var("DATABASE_URL").expect("无法获取数据库链接");
  SqlitePool::connect(&database_url)
  .await
  .expect("无法连接数据库")
}


#[derive(Serialize, Debug, Clone,FromRow)]
pub struct User{
  pub id: i32,
  pub openid: String,
  pub session_key: String,
  pub create_at: NaiveDateTime,
  pub update_at: NaiveDateTime
}