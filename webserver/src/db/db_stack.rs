use actix_web::HttpResponse;
use sqlx::{Pool, Sqlite};

use crate::{error::AnalysisError, User};

pub async fn create(pool: Pool<Sqlite>) -> Result<HttpResponse, AnalysisError> {
    Ok(HttpResponse::Ok().json("create"))
}

pub async fn batch_add() -> Result<HttpResponse, AnalysisError> {
    Ok(HttpResponse::Ok().json("batch_add"))
}

pub async fn add() -> Result<HttpResponse, AnalysisError> {
    Ok(HttpResponse::Ok().json("add"))
}

pub async fn query(pool: &Pool<Sqlite>) -> Result<HttpResponse, AnalysisError> {
    // let user_query = sqlx::query_as::<_, User>("SELECT * FROM USERS")
    //     .fetch_all(pool)
    //     .await?;
    Ok(HttpResponse::Ok().json("query"))
}
pub async fn update() -> Result<HttpResponse, AnalysisError> {
    Ok(HttpResponse::Ok().json("update"))
}
pub async fn delete() -> Result<HttpResponse, AnalysisError> {
    Ok(HttpResponse::Ok().json("delete"))
}
