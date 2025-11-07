use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};

/// API 响应结构体
#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    status: String,
    message: String,
}

/// 处理简单的 API 请求
#[get("/api/hello")]
pub fn hello() -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "success".to_string(),
        message: "Hello from API Gateway!".to_string(),
    })
}

/// 处理带有参数的 API 请求
#[get("/api/greet/<name>")]
pub fn greet(name: &str) -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "success".to_string(),
        message: format!("Hello, {}! Welcome to API Gateway.", name),
    })
}

/// 处理 POST 请求的 API
#[post("/api/echo", data = "<data>")]
pub fn echo(data: Json<serde_json::Value>) -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "success".to_string(),
        message: format!("Received data: {}", data.0),
    })
}
