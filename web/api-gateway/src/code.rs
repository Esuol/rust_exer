/// API Gateway 状态码和错误码管理模块
/// 提供统一的错误码、状态码定义和管理功能
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 错误码分类
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 客户端错误 (4xx)
    ClientError,
    /// 服务器错误 (5xx)
    ServerError,
    /// 网关错误
    GatewayError,
    /// 网络错误
    NetworkError,
    /// 配置错误
    ConfigError,
    /// 认证错误
    AuthError,
    /// 授权错误
    AuthorizationError,
    /// 限流错误
    RateLimitError,
    /// 熔断器错误
    CircuitBreakerError,
}

/// 错误码定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCode {
    /// 错误码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 错误描述
    pub description: String,
    /// 错误分类
    pub category: ErrorCategory,
    /// HTTP状态码
    pub http_status: u16,
    /// 是否可重试
    pub retryable: bool,
    /// 建议的解决方案
    pub suggestion: Option<String>,
}

/// 状态码管理器
pub struct CodeManager {
    /// 错误码映射
    error_codes: HashMap<String, ErrorCode>,
    /// 状态码映射
    status_codes: HashMap<u16, StatusCodeInfo>,
}

/// 状态码信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCodeInfo {
    /// 状态码
    pub code: u16,
    /// 状态名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 是否成功状态
    pub is_success: bool,
    /// 是否客户端错误
    pub is_client_error: bool,
    /// 是否服务器错误
    pub is_server_error: bool,
}

impl CodeManager {
    /// 创建新的状态码管理器
    pub fn new() -> Self {
        let mut manager = Self {
            error_codes: HashMap::new(),
            status_codes: HashMap::new(),
        };

        // 初始化默认错误码
        manager.init_default_error_codes();
        // 初始化默认状态码
        manager.init_default_status_codes();

        manager
    }

    /// 初始化默认错误码
    fn init_default_error_codes(&mut self) {
        let default_codes = vec![
            // 客户端错误 (4xx)
            ErrorCode {
                code: "BAD_REQUEST".to_string(),
                message: "请求格式错误".to_string(),
                description: "请求参数格式不正确或缺少必需参数".to_string(),
                category: ErrorCategory::ClientError,
                http_status: 400,
                retryable: false,
                suggestion: Some("请检查请求参数格式".to_string()),
            },
            ErrorCode {
                code: "UNAUTHORIZED".to_string(),
                message: "未授权访问".to_string(),
                description: "缺少有效的认证信息".to_string(),
                category: ErrorCategory::AuthError,
                http_status: 401,
                retryable: false,
                suggestion: Some("请提供有效的认证令牌".to_string()),
            },
            ErrorCode {
                code: "FORBIDDEN".to_string(),
                message: "禁止访问".to_string(),
                description: "没有权限访问该资源".to_string(),
                category: ErrorCategory::AuthorizationError,
                http_status: 403,
                retryable: false,
                suggestion: Some("请联系管理员获取访问权限".to_string()),
            },
            ErrorCode {
                code: "NOT_FOUND".to_string(),
                message: "资源未找到".to_string(),
                description: "请求的资源不存在".to_string(),
                category: ErrorCategory::ClientError,
                http_status: 404,
                retryable: false,
                suggestion: Some("请检查请求的URL路径".to_string()),
            },
            ErrorCode {
                code: "METHOD_NOT_ALLOWED".to_string(),
                message: "方法不允许".to_string(),
                description: "请求的HTTP方法不被允许".to_string(),
                category: ErrorCategory::ClientError,
                http_status: 405,
                retryable: false,
                suggestion: Some("请使用正确的HTTP方法".to_string()),
            },
            ErrorCode {
                code: "REQUEST_TIMEOUT".to_string(),
                message: "请求超时".to_string(),
                description: "请求处理超时".to_string(),
                category: ErrorCategory::ClientError,
                http_status: 408,
                retryable: true,
                suggestion: Some("请稍后重试或减少请求数据量".to_string()),
            },
            ErrorCode {
                code: "TOO_MANY_REQUESTS".to_string(),
                message: "请求过于频繁".to_string(),
                description: "请求频率超过限制".to_string(),
                category: ErrorCategory::RateLimitError,
                http_status: 429,
                retryable: true,
                suggestion: Some("请降低请求频率后重试".to_string()),
            },
            // 服务器错误 (5xx)
            ErrorCode {
                code: "INTERNAL_SERVER_ERROR".to_string(),
                message: "内部服务器错误".to_string(),
                description: "服务器内部发生未知错误".to_string(),
                category: ErrorCategory::ServerError,
                http_status: 500,
                retryable: true,
                suggestion: Some("请稍后重试，如问题持续请联系技术支持".to_string()),
            },
            ErrorCode {
                code: "BAD_GATEWAY".to_string(),
                message: "网关错误".to_string(),
                description: "上游服务器返回无效响应".to_string(),
                category: ErrorCategory::GatewayError,
                http_status: 502,
                retryable: true,
                suggestion: Some("请稍后重试".to_string()),
            },
            ErrorCode {
                code: "SERVICE_UNAVAILABLE".to_string(),
                message: "服务不可用".to_string(),
                description: "服务暂时不可用".to_string(),
                category: ErrorCategory::ServerError,
                http_status: 503,
                retryable: true,
                suggestion: Some("请稍后重试".to_string()),
            },
            ErrorCode {
                code: "GATEWAY_TIMEOUT".to_string(),
                message: "网关超时".to_string(),
                description: "上游服务器响应超时".to_string(),
                category: ErrorCategory::GatewayError,
                http_status: 504,
                retryable: true,
                suggestion: Some("请稍后重试".to_string()),
            },
            // 网关特定错误
            ErrorCode {
                code: "UPSTREAM_ERROR".to_string(),
                message: "上游服务错误".to_string(),
                description: "调用上游服务时发生错误".to_string(),
                category: ErrorCategory::GatewayError,
                http_status: 502,
                retryable: true,
                suggestion: Some("请稍后重试".to_string()),
            },
            ErrorCode {
                code: "CIRCUIT_BREAKER_OPEN".to_string(),
                message: "熔断器打开".to_string(),
                description: "服务熔断器已打开，暂时不可用".to_string(),
                category: ErrorCategory::CircuitBreakerError,
                http_status: 503,
                retryable: true,
                suggestion: Some("请稍后重试".to_string()),
            },
            ErrorCode {
                code: "LOAD_BALANCER_ERROR".to_string(),
                message: "负载均衡错误".to_string(),
                description: "没有可用的上游服务器".to_string(),
                category: ErrorCategory::GatewayError,
                http_status: 503,
                retryable: true,
                suggestion: Some("请稍后重试".to_string()),
            },
            ErrorCode {
                code: "CONFIG_ERROR".to_string(),
                message: "配置错误".to_string(),
                description: "网关配置错误".to_string(),
                category: ErrorCategory::ConfigError,
                http_status: 500,
                retryable: false,
                suggestion: Some("请联系管理员检查配置".to_string()),
            },
            ErrorCode {
                code: "NETWORK_ERROR".to_string(),
                message: "网络错误".to_string(),
                description: "网络连接错误".to_string(),
                category: ErrorCategory::NetworkError,
                http_status: 502,
                retryable: true,
                suggestion: Some("请检查网络连接后重试".to_string()),
            },
        ];

        for error_code in default_codes {
            self.error_codes.insert(error_code.code.clone(), error_code);
        }
    }

    /// 初始化默认状态码
    fn init_default_status_codes(&mut self) {
        let status_codes = vec![
            // 2xx 成功
            StatusCodeInfo {
                code: 200,
                name: "OK".to_string(),
                description: "请求成功".to_string(),
                is_success: true,
                is_client_error: false,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 201,
                name: "Created".to_string(),
                description: "资源创建成功".to_string(),
                is_success: true,
                is_client_error: false,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 204,
                name: "No Content".to_string(),
                description: "请求成功但无内容返回".to_string(),
                is_success: true,
                is_client_error: false,
                is_server_error: false,
            },
            // 4xx 客户端错误
            StatusCodeInfo {
                code: 400,
                name: "Bad Request".to_string(),
                description: "请求格式错误".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 401,
                name: "Unauthorized".to_string(),
                description: "未授权访问".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 403,
                name: "Forbidden".to_string(),
                description: "禁止访问".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 404,
                name: "Not Found".to_string(),
                description: "资源未找到".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 405,
                name: "Method Not Allowed".to_string(),
                description: "方法不允许".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 408,
                name: "Request Timeout".to_string(),
                description: "请求超时".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            StatusCodeInfo {
                code: 429,
                name: "Too Many Requests".to_string(),
                description: "请求过于频繁".to_string(),
                is_success: false,
                is_client_error: true,
                is_server_error: false,
            },
            // 5xx 服务器错误
            StatusCodeInfo {
                code: 500,
                name: "Internal Server Error".to_string(),
                description: "内部服务器错误".to_string(),
                is_success: false,
                is_client_error: false,
                is_server_error: true,
            },
            StatusCodeInfo {
                code: 502,
                name: "Bad Gateway".to_string(),
                description: "网关错误".to_string(),
                is_success: false,
                is_client_error: false,
                is_server_error: true,
            },
            StatusCodeInfo {
                code: 503,
                name: "Service Unavailable".to_string(),
                description: "服务不可用".to_string(),
                is_success: false,
                is_client_error: false,
                is_server_error: true,
            },
            StatusCodeInfo {
                code: 504,
                name: "Gateway Timeout".to_string(),
                description: "网关超时".to_string(),
                is_success: false,
                is_client_error: false,
                is_server_error: true,
            },
        ];

        for status_code in status_codes {
            self.status_codes.insert(status_code.code, status_code);
        }
    }

    /// 获取错误码
    pub fn get_error_code(&self, code: &str) -> Option<&ErrorCode> {
        self.error_codes.get(code)
    }

    /// 添加错误码
    pub fn add_error_code(&mut self, error_code: ErrorCode) {
        self.error_codes.insert(error_code.code.clone(), error_code);
    }

    /// 获取状态码信息
    pub fn get_status_code_info(&self, code: u16) -> Option<&StatusCodeInfo> {
        self.status_codes.get(&code)
    }

    /// 添加状态码信息
    pub fn add_status_code_info(&mut self, status_code_info: StatusCodeInfo) {
        self.status_codes
            .insert(status_code_info.code, status_code_info);
    }

    /// 根据HTTP状态码获取错误码
    pub fn get_error_code_by_http_status(&self, http_status: u16) -> Option<&ErrorCode> {
        self.error_codes
            .values()
            .find(|error_code| error_code.http_status == http_status)
    }

    /// 根据分类获取错误码列表
    pub fn get_error_codes_by_category(&self, category: &ErrorCategory) -> Vec<&ErrorCode> {
        self.error_codes
            .values()
            .filter(|error_code| &error_code.category == category)
            .collect()
    }

    /// 获取所有错误码
    pub fn get_all_error_codes(&self) -> Vec<&ErrorCode> {
        self.error_codes.values().collect()
    }

    /// 获取所有状态码
    pub fn get_all_status_codes(&self) -> Vec<&StatusCodeInfo> {
        self.status_codes.values().collect()
    }

    /// 检查状态码是否为成功状态
    pub fn is_success_status(&self, code: u16) -> bool {
        self.status_codes
            .get(&code)
            .map(|info| info.is_success)
            .unwrap_or(false)
    }

    /// 检查状态码是否为客户端错误
    pub fn is_client_error_status(&self, code: u16) -> bool {
        self.status_codes
            .get(&code)
            .map(|info| info.is_client_error)
            .unwrap_or(false)
    }

    /// 检查状态码是否为服务器错误
    pub fn is_server_error_status(&self, code: u16) -> bool {
        self.status_codes
            .get(&code)
            .map(|info| info.is_server_error)
            .unwrap_or(false)
    }

    /// 获取可重试的错误码
    pub fn get_retryable_error_codes(&self) -> Vec<&ErrorCode> {
        self.error_codes
            .values()
            .filter(|error_code| error_code.retryable)
            .collect()
    }
}

impl Default for CodeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误码构建器
pub struct ErrorCodeBuilder {
    code: String,
    message: String,
    description: String,
    category: ErrorCategory,
    http_status: u16,
    retryable: bool,
    suggestion: Option<String>,
}

impl ErrorCodeBuilder {
    /// 创建新的错误码构建器
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            description: String::new(),
            category: ErrorCategory::ServerError,
            http_status: 500,
            retryable: false,
            suggestion: None,
        }
    }

    /// 设置描述
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// 设置分类
    pub fn category(mut self, category: ErrorCategory) -> Self {
        self.category = category;
        self
    }

    /// 设置HTTP状态码
    pub fn http_status(mut self, status: u16) -> Self {
        self.http_status = status;
        self
    }

    /// 设置是否可重试
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// 设置建议
    pub fn suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    /// 构建错误码
    pub fn build(self) -> ErrorCode {
        ErrorCode {
            code: self.code,
            message: self.message,
            description: self.description,
            category: self.category,
            http_status: self.http_status,
            retryable: self.retryable,
            suggestion: self.suggestion,
        }
    }
}

/// 状态码工具函数
pub struct StatusCodeUtils;

impl StatusCodeUtils {
    /// 获取状态码描述
    pub fn get_description(code: u16) -> &'static str {
        match code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "Unknown Status",
        }
    }

    /// 检查是否为成功状态码
    pub fn is_success(code: u16) -> bool {
        code >= 200 && code < 300
    }

    /// 检查是否为客户端错误状态码
    pub fn is_client_error(code: u16) -> bool {
        code >= 400 && code < 500
    }

    /// 检查是否为服务器错误状态码
    pub fn is_server_error(code: u16) -> bool {
        code >= 500 && code < 600
    }

    /// 检查是否应该重试
    pub fn should_retry(code: u16) -> bool {
        match code {
            408 | 429 | 500 | 502 | 503 | 504 => true,
            _ => false,
        }
    }

    /// 获取Rocket状态码
    pub fn to_rocket_status(code: u16) -> Status {
        match code {
            200 => Status::Ok,
            201 => Status::Created,
            204 => Status::NoContent,
            400 => Status::BadRequest,
            401 => Status::Unauthorized,
            403 => Status::Forbidden,
            404 => Status::NotFound,
            405 => Status::MethodNotAllowed,
            408 => Status::RequestTimeout,
            429 => Status::TooManyRequests,
            500 => Status::InternalServerError,
            502 => Status::BadGateway,
            503 => Status::ServiceUnavailable,
            504 => Status::GatewayTimeout,
            _ => Status::InternalServerError,
        }
    }
}

/// 错误码追踪宏
#[macro_export]
macro_rules! trace_error_code {
    ($code:expr, $context:expr) => {
        log::error!("[错误码] {} - 上下文: {}", $code, $context);
    };
}

/// 状态码追踪宏
#[macro_export]
macro_rules! trace_status_code {
    ($code:expr, $context:expr) => {
        log::debug!("[状态码] {} - 上下文: {}", $code, $context);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_manager_creation() {
        let manager = CodeManager::new();
        assert!(!manager.error_codes.is_empty());
        assert!(!manager.status_codes.is_empty());
    }

    #[test]
    fn test_get_error_code() {
        let manager = CodeManager::new();
        let error_code = manager.get_error_code("BAD_REQUEST");
        assert!(error_code.is_some());
        assert_eq!(error_code.unwrap().code, "BAD_REQUEST");
        assert_eq!(error_code.unwrap().http_status, 400);
    }

    #[test]
    fn test_get_status_code_info() {
        let manager = CodeManager::new();
        let status_info = manager.get_status_code_info(200);
        assert!(status_info.is_some());
        assert_eq!(status_info.unwrap().code, 200);
        assert!(status_info.unwrap().is_success);
    }

    #[test]
    fn test_error_code_builder() {
        let error_code = ErrorCodeBuilder::new("TEST_ERROR", "测试错误")
            .description("这是一个测试错误")
            .category(ErrorCategory::ClientError)
            .http_status(400)
            .retryable(false)
            .suggestion("请检查输入")
            .build();

        assert_eq!(error_code.code, "TEST_ERROR");
        assert_eq!(error_code.message, "测试错误");
        assert_eq!(error_code.description, "这是一个测试错误");
        assert_eq!(error_code.category, ErrorCategory::ClientError);
        assert_eq!(error_code.http_status, 400);
        assert!(!error_code.retryable);
        assert_eq!(error_code.suggestion, Some("请检查输入".to_string()));
    }

    #[test]
    fn test_status_code_utils() {
        assert_eq!(StatusCodeUtils::get_description(200), "OK");
        assert_eq!(StatusCodeUtils::get_description(404), "Not Found");
        assert_eq!(
            StatusCodeUtils::get_description(500),
            "Internal Server Error"
        );

        assert!(StatusCodeUtils::is_success(200));
        assert!(!StatusCodeUtils::is_success(400));
        assert!(!StatusCodeUtils::is_success(500));

        assert!(StatusCodeUtils::is_client_error(400));
        assert!(!StatusCodeUtils::is_client_error(200));
        assert!(!StatusCodeUtils::is_client_error(500));

        assert!(StatusCodeUtils::is_server_error(500));
        assert!(!StatusCodeUtils::is_server_error(200));
        assert!(!StatusCodeUtils::is_server_error(400));

        assert!(StatusCodeUtils::should_retry(408));
        assert!(StatusCodeUtils::should_retry(500));
        assert!(!StatusCodeUtils::should_retry(400));
    }

    #[test]
    fn test_get_error_codes_by_category() {
        let manager = CodeManager::new();
        let client_errors = manager.get_error_codes_by_category(&ErrorCategory::ClientError);
        assert!(!client_errors.is_empty());

        let server_errors = manager.get_error_codes_by_category(&ErrorCategory::ServerError);
        assert!(!server_errors.is_empty());
    }

    #[test]
    fn test_retryable_error_codes() {
        let manager = CodeManager::new();
        let retryable_codes = manager.get_retryable_error_codes();
        assert!(!retryable_codes.is_empty());

        for code in retryable_codes {
            assert!(code.retryable);
        }
    }
}
