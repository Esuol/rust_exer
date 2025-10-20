/// API Gateway 请求处理模块
/// 提供请求解析、验证和转换功能
use rocket::http::Status;
use rocket::request::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 请求信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    /// 请求方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 查询参数
    pub query_params: HashMap<String, String>,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 客户端IP
    pub client_ip: Option<String>,
    /// 用户代理
    pub user_agent: Option<String>,
    /// 请求时间戳
    pub timestamp: String,
    /// 请求ID（用于追踪）
    pub request_id: String,
}

/// 请求验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误信息
    pub errors: Vec<String>,
    /// 警告信息
    pub warnings: Vec<String>,
}

/// 请求处理器
pub struct RequestHandler {
    /// 最大请求大小（字节）
    pub max_request_size: usize,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 允许的方法列表
    pub allowed_methods: Vec<String>,
    /// 允许的头部列表
    pub allowed_headers: Vec<String>,
}

impl RequestHandler {
    /// 创建新的请求处理器
    pub fn new() -> Self {
        Self {
            max_request_size: 10 * 1024 * 1024, // 10MB
            request_timeout: 30,                // 30秒
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "HEAD".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "User-Agent".to_string(),
                "Accept".to_string(),
                "Accept-Language".to_string(),
                "Accept-Encoding".to_string(),
            ],
        }
    }

    /// 解析请求信息
    pub fn parse_request_info(&self, request: &Request) -> RequestInfo {
        let method = request.method().to_string();
        let path = request.uri().path().to_string();

        // 解析查询参数
        let mut query_params = HashMap::new();
        if let Some(query) = request.uri().query() {
            for param in query.split('&') {
                let parts: Vec<&str> = param.split('=').map(|s| s.as_str()).collect();
                if parts.len() == 2 {
                    query_params.insert(
                        urlencoding::decode(parts[0])
                            .unwrap_or_default()
                            .to_string(),
                        urlencoding::decode(parts[1])
                            .unwrap_or_default()
                            .to_string(),
                    );
                }
            }
        }

        // 解析请求头
        let mut headers = HashMap::new();
        for header in request.headers().iter() {
            headers.insert(header.name().to_string(), header.value().to_string());
        }

        // 获取客户端IP
        let client_ip = request
            .headers()
            .get_one("X-Forwarded-For")
            .or_else(|| request.headers().get_one("X-Real-IP"))
            .map(|s| s.to_string());

        // 获取用户代理
        let user_agent = request
            .headers()
            .get_one("User-Agent")
            .map(|s| s.to_string());

        RequestInfo {
            method,
            path,
            query_params,
            headers,
            client_ip,
            user_agent,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: self.generate_request_id(),
        }
    }

    /// 验证请求
    pub fn validate_request(&self, request_info: &RequestInfo) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 验证请求方法
        if !self.allowed_methods.contains(&request_info.method) {
            errors.push(format!("不允许的请求方法: {}", request_info.method));
        }

        // 验证请求头
        for (header_name, _) in &request_info.headers {
            if !self
                .allowed_headers
                .iter()
                .any(|h| h.eq_ignore_ascii_case(header_name))
            {
                warnings.push(format!("未识别的请求头: {}", header_name));
            }
        }

        // 验证路径长度
        if request_info.path.len() > 2048 {
            errors.push("请求路径过长".to_string());
        }

        // 验证查询参数数量
        if request_info.query_params.len() > 100 {
            warnings.push("查询参数过多".to_string());
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// 生成请求ID
    fn generate_request_id(&self) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("req_{:016x}", id)
    }

    /// 检查请求是否超时
    pub fn is_request_timeout(&self, start_time: Instant) -> bool {
        start_time.elapsed() > Duration::from_secs(self.request_timeout)
    }

    /// 获取请求大小估算
    pub fn estimate_request_size(&self, request_info: &RequestInfo) -> usize {
        let mut size = 0;

        // 方法大小
        size += request_info.method.len();

        // 路径大小
        size += request_info.path.len();

        // 查询参数大小
        for (key, value) in &request_info.query_params {
            size += key.len() + value.len() + 2; // +2 for '=' and '&'
        }

        // 头部大小
        for (key, value) in &request_info.headers {
            size += key.len() + value.len() + 4; // +4 for ': ' and '\r\n'
        }

        size
    }

    /// 检查请求大小是否超限
    pub fn is_request_size_valid(&self, request_info: &RequestInfo) -> bool {
        self.estimate_request_size(request_info) <= self.max_request_size
    }

    /// 创建错误响应
    pub fn create_error_response(&self, status: Status, message: &str) -> String {
        serde_json::json!({
            "error": true,
            "status": status.code,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string()
    }

    /// 创建成功响应
    pub fn create_success_response(&self, data: &serde_json::Value) -> String {
        serde_json::json!({
            "success": true,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string()
    }
}

impl Default for RequestHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求追踪宏
/// 用于记录请求的详细信息
#[macro_export]
macro_rules! trace_request_info {
    ($request_info:expr) => {
        log::debug!(
            "[请求追踪] {} {} - ID: {} - IP: {:?}",
            $request_info.method,
            $request_info.path,
            $request_info.request_id,
            $request_info.client_ip
        );
    };
}

/// 请求验证宏
/// 用于快速验证请求
#[macro_export]
macro_rules! validate_request {
    ($handler:expr, $request_info:expr) => {{
        let validation = $handler.validate_request($request_info);
        if !validation.is_valid {
            log::warn!(
                "[请求验证失败] {} - 错误: {:?}",
                $request_info.request_id,
                validation.errors
            );
            return Err(rocket::http::Status::BadRequest);
        }
        if !validation.warnings.is_empty() {
            log::warn!(
                "[请求验证警告] {} - 警告: {:?}",
                $request_info.request_id,
                validation.warnings
            );
        }
        validation
    }};
}

/// 请求大小检查宏
#[macro_export]
macro_rules! check_request_size {
    ($handler:expr, $request_info:expr) => {
        if !$handler.is_request_size_valid($request_info) {
            log::warn!(
                "[请求大小超限] {} - 大小: {} 字节",
                $request_info.request_id,
                $handler.estimate_request_size($request_info)
            );
            return Err(rocket::http::Status::PayloadTooLarge);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Method;
    use rocket::local::blocking::Client;
    use rocket::Rocket;

    #[test]
    fn test_request_handler_creation() {
        let handler = RequestHandler::new();
        assert_eq!(handler.max_request_size, 10 * 1024 * 1024);
        assert_eq!(handler.request_timeout, 30);
        assert!(!handler.allowed_methods.is_empty());
    }

    #[test]
    fn test_request_id_generation() {
        let handler = RequestHandler::new();
        let id1 = handler.generate_request_id();
        let id2 = handler.generate_request_id();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("req_"));
        assert!(id2.starts_with("req_"));
    }

    #[test]
    fn test_request_size_estimation() {
        let handler = RequestHandler::new();
        let request_info = RequestInfo {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            client_ip: None,
            user_agent: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: "test".to_string(),
        };

        let size = handler.estimate_request_size(&request_info);
        assert!(size > 0);
    }

    #[test]
    fn test_request_validation() {
        let handler = RequestHandler::new();
        let mut request_info = RequestInfo {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            client_ip: None,
            user_agent: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: "test".to_string(),
        };

        let validation = handler.validate_request(&request_info);
        assert!(validation.is_valid);

        // 测试无效方法
        request_info.method = "INVALID".to_string();
        let validation = handler.validate_request(&request_info);
        assert!(!validation.is_valid);
    }
}
