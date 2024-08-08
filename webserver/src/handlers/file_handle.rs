use actix_web::{HttpResponse};

use crate::{files::file_analysis};

pub async fn load_file_handler(
    path: String
) -> HttpResponse {
    println!("path:{}", path);
    file_analysis::analysis(&path);
    HttpResponse::Ok().json("OK")
}
