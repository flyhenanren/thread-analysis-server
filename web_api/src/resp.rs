use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T = ()> {
    pub code: u16,         // 状态码
    pub data: Option<T>,   // 返回的数据
    pub message: String,   // 错误消息或成功描述
}

impl<T> ApiResponse<T> {
    // 成功的响应
    pub fn success(data: Option<T>) -> Self {
        ApiResponse {
            code: 200,
            data,
            message: "success".to_string(),
        }
    }

}

impl ApiResponse<()> {
    /// 不带数据的成功响应
    pub fn ok() -> Self {
        ApiResponse {
            code: 200,
            data: None,
            message: "success".to_string(),
        }
    }
}

impl ApiResponse<()> {
    // 失败的响应
    pub fn error(code: u16, message: &str) -> Self {
        ApiResponse {
            code,
            data: None,
            message: message.to_string(),
        }
    }
}

   
