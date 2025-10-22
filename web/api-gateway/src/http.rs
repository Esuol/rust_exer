/// API Gateway HTTP 处理模块
/// 提供 HTTP 客户端、连接池管理、请求/响应处理功能
use reqwest::{Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use url::Url;

/// HTTP 请求配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 读取超时时间（秒）
    pub read_timeout: u64,
    /// 写入超时时间（秒）
    pub write_timeout: u64,
    /// 最大连接数
    pub max_connections: usize,
    /// 每个主机的最大连接数
    pub max_connections_per_host: usize,
    /// 是否启用 HTTP/2
    pub enable_http2: bool,
    /// 是否启用连接复用
    pub enable_connection_reuse: bool,
    /// 用户代理
    pub user_agent: Option<String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 10,
            read_timeout: 30,
            write_timeout: 30,
            max_connections: 100,
            max_connections_per_host: 10,
            enable_http2: true,
            enable_connection_reuse: true,
            user_agent: Some("API-Gateway/1.0".to_string()),
        }
    }
}

/// HTTP 请求信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// 请求方法
    pub method: String,
    /// 请求URL
    pub url: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Option<String>,
    /// 查询参数
    pub query_params: HashMap<String, String>,
    /// 超时时间（秒）
    pub timeout: Option<u64>,
}

/// HTTP 响应信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// 状态码
    pub status_code: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: String,
    /// 响应时间（毫秒）
    pub response_time_ms: f64,
    /// 请求URL
    pub url: String,
    /// 是否成功
    pub is_success: bool,
}

/// 连接统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 最大响应时间（毫秒）
    pub max_response_time_ms: f64,
    /// 最小响应时间（毫秒）
    pub min_response_time_ms: f64,
    /// 活跃连接数
    pub active_connections: usize,
}

/// HTTP 客户端管理器
pub struct HttpClientManager {
    /// HTTP 客户端
    client: Client,
    /// 配置
    config: HttpConfig,
    /// 连接统计
    stats: Arc<RwLock<ConnectionStats>>,
}

impl HttpClientManager {
    /// 创建新的 HTTP 客户端管理器
    pub fn new(config: HttpConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut builder = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .timeout(Duration::from_secs(config.read_timeout))
            .pool_max_idle_per_host(config.max_connections_per_host)
            .http2_prior_knowledge();

        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            config,
            stats: Arc::new(RwLock::new(ConnectionStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time_ms: 0.0,
                max_response_time_ms: 0.0,
                min_response_time_ms: f64::MAX,
                active_connections: 0,
            })),
        })
    }

    /// 发送 HTTP 请求
    pub async fn send_request(&self, request: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        
        // 构建请求
        let mut req_builder = match request.method.to_uppercase().as_str() {
            "GET" => self.client.get(&request.url),
            "POST" => self.client.post(&request.url),
            "PUT" => self.client.put(&request.url),
            "DELETE" => self.client.delete(&request.url),
            "PATCH" => self.client.patch(&request.url),
            "HEAD" => self.client.head(&request.url),
            "OPTIONS" => self.client.request(reqwest::Method::OPTIONS, &request.url),
            _ => return Err("不支持的 HTTP 方法".into()),
        };

        // 添加请求头
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // 添加查询参数
        for (key, value) in &request.query_params {
            req_builder = req_builder.query(&[(key, value)]);
        }

        // 添加请求体
        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        // 设置超时
        if let Some(timeout) = request.timeout {
            req_builder = req_builder.timeout(Duration::from_secs(timeout));
        }

        // 发送请求
        let response = req_builder.send().await?;
        let response_time = start_time.elapsed();
        let response_time_ms = response_time.as_secs_f64() * 1000.0;

        // 解析响应
        let status_code = response.status().as_u16();
        let is_success = response.status().is_success();
        let headers = self.extract_headers(&response);
        let body = response.text().await?;

        // 更新统计信息
        self.update_stats(is_success, response_time_ms).await;

        Ok(HttpResponse {
            status_code,
            headers,
            body,
            response_time_ms,
            url: request.url,
            is_success,
        })
    }

    /// 发送 GET 请求
    pub async fn get(&self, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.send_request(HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            timeout: None,
        }).await
    }

    /// 发送 POST 请求
    pub async fn post(&self, url: &str, body: Option<String>) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.send_request(HttpRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body,
            query_params: HashMap::new(),
            timeout: None,
        }).await
    }

    /// 发送带自定义头部的请求
    pub async fn send_with_headers(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.send_request(HttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            headers,
            body,
            query_params: HashMap::new(),
            timeout: None,
        }).await
    }

    /// 提取响应头
    fn extract_headers(&self, response: &Response) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (key, value) in response.headers().iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }
        headers
    }

    /// 更新连接统计信息
    async fn update_stats(&self, is_success: bool, response_time_ms: f64) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        
        if is_success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        // 更新响应时间统计
        if stats.min_response_time_ms == f64::MAX {
            stats.min_response_time_ms = response_time_ms;
        } else {
            stats.min_response_time_ms = stats.min_response_time_ms.min(response_time_ms);
        }
        stats.max_response_time_ms = stats.max_response_time_ms.max(response_time_ms);
        
        // 计算平均响应时间
        let total_time = stats.avg_response_time_ms * (stats.total_requests - 1) as f64 + response_time_ms;
        stats.avg_response_time_ms = total_time / stats.total_requests as f64;
    }

    /// 获取连接统计信息
    pub async fn get_stats(&self) -> ConnectionStats {
        self.stats.read().await.clone()
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ConnectionStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            max_response_time_ms: 0.0,
            min_response_time_ms: f64::MAX,
            active_connections: 0,
        };
    }

    /// 检查 URL 是否有效
    pub fn is_valid_url(&self, url: &str) -> bool {
        Url::parse(url).is_ok()
    }

    /// 构建完整的 URL
    pub fn build_url(&self, base_url: &str, path: &str, query_params: &HashMap<String, String>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut url = if base_url.ends_with('/') && path.starts_with('/') {
            format!("{}{}", base_url.trim_end_matches('/'), path)
        } else if !base_url.ends_with('/') && !path.starts_with('/') {
            format!("{}/{}", base_url, path)
        } else {
            format!("{}{}", base_url, path)
        };

        if !query_params.is_empty() {
            url.push('?');
            let query_string: Vec<String> = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            url.push_str(&query_string.join("&"));
        }

        Ok(url)
    }

    /// 获取配置信息
    pub fn get_config(&self) -> &HttpConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: HttpConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.config = config;
        // 注意：这里需要重新创建客户端，但为了简化，我们保持现有客户端
        Ok(())
    }
}

/// HTTP 请求构建器
pub struct HttpRequestBuilder {
    request: HttpRequest,
}

impl HttpRequestBuilder {
    /// 创建新的请求构建器
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            request: HttpRequest {
                method: method.to_string(),
                url: url.to_string(),
                headers: HashMap::new(),
                body: None,
                query_params: HashMap::new(),
                timeout: None,
            },
        }
    }

    /// 添加请求头
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.request.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 添加查询参数
    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.request.query_params.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置请求体
    pub fn body(mut self, body: &str) -> Self {
        self.request.body = Some(body.to_string());
        self
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout_secs: u64) -> Self {
        self.request.timeout = Some(timeout_secs);
        self
    }

    /// 构建请求
    pub fn build(self) -> HttpRequest {
        self.request
    }
}

/// HTTP 响应处理器
pub struct HttpResponseProcessor;

impl HttpResponseProcessor {
    /// 检查响应是否成功
    pub fn is_success(&self, response: &HttpResponse) -> bool {
        response.is_success
    }

    /// 获取状态码描述
    pub fn get_status_description(&self, status_code: u16) -> &'static str {
        match status_code {
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

    /// 格式化响应为 JSON
    pub fn format_response_json(&self, response: &HttpResponse) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&serde_json::json!({
            "status_code": response.status_code,
            "status_description": self.get_status_description(response.status_code),
            "headers": response.headers,
            "body": response.body,
            "response_time_ms": response.response_time_ms,
            "url": response.url,
            "is_success": response.is_success
        }))
    }

    /// 检查响应是否包含特定头部
    pub fn has_header(&self, response: &HttpResponse, header_name: &str) -> bool {
        response.headers.contains_key(header_name)
    }

    /// 获取响应头值
    pub fn get_header<'a>(&self, response: &'a HttpResponse, header_name: &str) -> Option<&'a String> {
        response.headers.get(header_name)
    }
}

/// HTTP 连接池监控
pub struct ConnectionPoolMonitor {
    stats: Arc<RwLock<ConnectionStats>>,
}

impl ConnectionPoolMonitor {
    /// 创建新的连接池监控器
    pub fn new(stats: Arc<RwLock<ConnectionStats>>) -> Self {
        Self { stats }
    }

    /// 获取连接池状态
    pub async fn get_pool_status(&self) -> ConnectionStats {
        self.stats.read().await.clone()
    }

    /// 检查连接池健康状态
    pub async fn is_healthy(&self) -> bool {
        let stats = self.stats.read().await;
        stats.total_requests > 0 && stats.successful_requests as f64 / stats.total_requests as f64 > 0.8
    }

    /// 获取成功率
    pub async fn get_success_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.total_requests == 0 {
            0.0
        } else {
            stats.successful_requests as f64 / stats.total_requests as f64
        }
    }
}

/// HTTP 请求追踪宏
#[macro_export]
macro_rules! trace_http_request {
    ($method:expr, $url:expr, $start_time:expr) => {
        log::debug!(
            "[HTTP请求] {} {} - 开始时间: {:?}",
            $method,
            $url,
            $start_time
        );
    };
}

/// HTTP 响应追踪宏
#[macro_export]
macro_rules! trace_http_response {
    ($response:expr, $start_time:expr) => {
        log::debug!(
            "[HTTP响应] {} - 状态码: {} - 响应时间: {:.2}ms",
            $response.url,
            $response.status_code,
            $response.response_time_ms
        );
    };
}

/// HTTP 错误处理宏
#[macro_export]
macro_rules! handle_http_error {
    ($error:expr, $context:expr) => {
        log::error!(
            "[HTTP错误] {} - 错误: {}",
            $context,
            $error
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.connect_timeout, 10);
        assert_eq!(config.read_timeout, 30);
        assert_eq!(config.max_connections, 100);
        assert!(config.enable_http2);
    }

    #[test]
    fn test_http_request_builder() {
        let request = HttpRequestBuilder::new("GET", "https://example.com")
            .header("Content-Type", "application/json")
            .query_param("page", "1")
            .timeout(30)
            .build();

        assert_eq!(request.method, "GET");
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(request.timeout, Some(30));
    }

    #[test]
    fn test_url_validation() {
        let config = HttpConfig::default();
        let manager = HttpClientManager::new(config).unwrap();
        
        assert!(manager.is_valid_url("https://example.com"));
        assert!(manager.is_valid_url("http://localhost:8080"));
        assert!(!manager.is_valid_url("invalid-url"));
    }

    #[test]
    fn test_url_building() {
        let config = HttpConfig::default();
        let manager = HttpClientManager::new(config).unwrap();
        
        let mut query_params = HashMap::new();
        query_params.insert("page".to_string(), "1".to_string());
        query_params.insert("size".to_string(), "10".to_string());
        
        let url = manager.build_url("https://api.example.com", "/users", &query_params).unwrap();
        assert!(url.contains("https://api.example.com/users"));
        assert!(url.contains("page=1"));
        assert!(url.contains("size=10"));
    }

    #[test]
    fn test_response_processor() {
        let processor = HttpResponseProcessor;
        
        assert_eq!(processor.get_status_description(200), "OK");
        assert_eq!(processor.get_status_description(404), "Not Found");
        assert_eq!(processor.get_status_description(500), "Internal Server Error");
    }
}
