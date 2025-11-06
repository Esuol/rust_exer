use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use std::io::Cursor;

/// 自定义错误类型
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiGatewayError {
    /// 路由未找到
    RouteNotFound(String),
    /// 上游服务不可用
    UpstreamUnavailable(String),
    /// 请求超时
    RequestTimeout(String),
    /// 内部服务器错误
    InternalServerError(String),
    /// 网关错误
    BadGateway(String),
}

impl fmt::Display for ApiGatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiGatewayError::RouteNotFound(msg) => write!(f, "Route not found: {}", msg),
            ApiGatewayError::UpstreamUnavailable(msg) => write!(f, "Upstream unavailable: {}", msg),
            ApiGatewayError::RequestTimeout(msg) => write!(f, "Request timeout: {}", msg),
            ApiGatewayError::InternalServerError(msg) => {
                write!(f, "Internal server error: {}", msg)
            }
            ApiGatewayError::BadGateway(msg) => write!(f, "Bad gateway: {}", msg),
        }
    }
}

impl StdError for ApiGatewayError {}

impl ApiGatewayError {
    /// 转换为 HTTP 状态码
    pub fn to_status(&self) -> Status {
        match self {
            ApiGatewayError::RouteNotFound(_) => Status::NotFound,
            ApiGatewayError::UpstreamUnavailable(_) => Status::ServiceUnavailable,
            ApiGatewayError::RequestTimeout(_) => Status::GatewayTimeout,
            ApiGatewayError::InternalServerError(_) => Status::InternalServerError,
            ApiGatewayError::BadGateway(_) => Status::BadGateway,
        }
    }

    /// 创建一个新的 RouteNotFound 错误
    pub fn route_not_found(path: &str) -> Self {
        ApiGatewayError::RouteNotFound(format!("Path {} not found", path))
    }

    /// 创建一个新的 UpstreamUnavailable 错误
    pub fn upstream_unavailable(url: &str) -> Self {
        ApiGatewayError::UpstreamUnavailable(format!("Upstream {} unavailable", url))
    }

    /// 创建一个新的 RequestTimeout 错误
    #[allow(dead_code)] // 预留功能，未来可能使用
    pub fn request_timeout(url: &str) -> Self {
        ApiGatewayError::RequestTimeout(format!("Request to {} timed out", url))
    }

    /// 创建一个新的 InternalServerError 错误
    pub fn internal_error(msg: &str) -> Self {
        ApiGatewayError::InternalServerError(msg.to_string())
    }

    /// 创建一个新的 BadGateway 错误
    pub fn bad_gateway(msg: &str) -> Self {
        ApiGatewayError::BadGateway(msg.to_string())
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiGatewayError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'o> {
        let status = self.to_status();
        let body = serde_json::to_string(&self).unwrap_or_else(|_| format!("{}", self));
        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

/// 错误处理结果类型
#[allow(dead_code)] // 预留类型别名，未来可能使用
pub type Result<T> = std::result::Result<T, ApiGatewayError>;
