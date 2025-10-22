/// API Gateway 响应处理模块
/// 提供响应格式化、缓存、压缩、错误处理等功能
use rocket::http::{ContentType, Header, Status};
use rocket::response::{Responder, Response as RocketResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::{Duration, Instant};

/// 响应配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseConfig {
    /// 默认内容类型
    pub default_content_type: String,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 压缩级别 (1-9)
    pub compression_level: u32,
    /// 缓存配置
    pub cache_config: CacheConfig,
    /// 错误处理配置
    pub error_config: ErrorConfig,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 默认缓存时间（秒）
    pub default_ttl: u64,
    /// 最大缓存大小（字节）
    pub max_size: usize,
    /// 缓存键前缀
    pub key_prefix: String,
}

/// 错误处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorConfig {
    /// 是否显示详细错误信息
    pub show_details: bool,
    /// 是否记录错误日志
    pub log_errors: bool,
    /// 错误响应格式
    pub format: String, // "json" 或 "html"
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            default_content_type: "application/json".to_string(),
            enable_compression: true,
            compression_level: 6,
            cache_config: CacheConfig::default(),
            error_config: ErrorConfig::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: 300,            // 5分钟
            max_size: 100 * 1024 * 1024, // 100MB
            key_prefix: "gateway:".to_string(),
        }
    }
}

impl Default for ErrorConfig {
    fn default() -> Self {
        Self {
            show_details: false,
            log_errors: true,
            format: "json".to_string(),
        }
    }
}

/// 统一响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 状态码
    pub status_code: u16,
    /// 消息
    pub message: String,
    /// 数据
    pub data: Option<T>,
    /// 时间戳
    pub timestamp: String,
    /// 请求ID
    pub request_id: Option<String>,
    /// 错误详情（仅在错误时显示）
    pub error_details: Option<ErrorDetails>,
}

/// 错误详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// 错误类型
    pub error_type: String,
    /// 错误代码
    pub error_code: String,
    /// 错误描述
    pub description: String,
    /// 堆栈跟踪（仅在开发环境）
    pub stack_trace: Option<String>,
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// 响应时间（毫秒）
    pub response_time_ms: f64,
    /// 服务器信息
    pub server: String,
    /// 版本信息
    pub version: String,
    /// 缓存信息
    pub cache_info: Option<CacheInfo>,
}

/// 缓存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// 是否命中缓存
    pub hit: bool,
    /// 缓存键
    pub key: String,
    /// 缓存时间（秒）
    pub ttl: u64,
    /// 创建时间
    pub created_at: String,
}

/// 响应处理器
pub struct ResponseHandler {
    config: ResponseConfig,
    cache: HashMap<String, CachedResponse>,
}

/// 缓存的响应
#[derive(Debug, Clone)]
struct CachedResponse {
    content: String,
    headers: HashMap<String, String>,
    created_at: Instant,
    ttl: Duration,
}

impl ResponseHandler {
    /// 创建新的响应处理器
    pub fn new(config: ResponseConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// 创建成功响应
    pub fn success<T: Serialize>(
        &self,
        data: T,
        message: Option<String>,
        request_id: Option<String>,
    ) -> ApiResponse<T> {
        ApiResponse {
            success: true,
            status_code: 200,
            message: message.unwrap_or_else(|| "操作成功".to_string()),
            data: Some(data),
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id,
            error_details: None,
        }
    }

    /// 创建错误响应
    pub fn error(
        &self,
        status_code: u16,
        message: String,
        error_type: Option<String>,
        request_id: Option<String>,
    ) -> ApiResponse<serde_json::Value> {
        let error_details = if self.config.error_config.show_details {
            Some(ErrorDetails {
                error_type: error_type.unwrap_or_else(|| "UnknownError".to_string()),
                error_code: format!("ERR_{}", status_code),
                description: message.clone(),
                stack_trace: None, // 生产环境不显示堆栈跟踪
            })
        } else {
            None
        };

        ApiResponse {
            success: false,
            status_code,
            message,
            data: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id,
            error_details,
        }
    }

    /// 创建分页响应
    pub fn paginated<T: Serialize>(
        &self,
        data: Vec<T>,
        page: u32,
        page_size: u32,
        total: u64,
        request_id: Option<String>,
    ) -> ApiResponse<PaginatedData<T>> {
        let paginated_data = PaginatedData {
            items: data,
            pagination: PaginationInfo {
                page,
                page_size,
                total,
                total_pages: ((total as f64) / (page_size as f64)).ceil() as u32,
                has_next: page * page_size < total as u32,
                has_prev: page > 1,
            },
        };

        self.success(paginated_data, Some("数据获取成功".to_string()), request_id)
    }

    /// 格式化响应为 JSON
    pub fn format_json<T: Serialize>(
        &self,
        response: &ApiResponse<T>,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(response)
    }

    /// 格式化响应为 HTML
    pub fn format_html<T: Serialize>(&self, response: &ApiResponse<T>) -> String {
        if response.success {
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>API 响应</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>操作成功</h1>
    <p><strong>消息:</strong> {}</p>
    <p><strong>时间:</strong> {}</p>
    <pre>{}</pre>
</body>
</html>"#,
                response.message,
                response.timestamp,
                serde_json::to_string_pretty(&response.data).unwrap_or_default()
            )
        } else {
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>API 错误</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>操作失败</h1>
    <p><strong>状态码:</strong> {}</p>
    <p><strong>消息:</strong> {}</p>
    <p><strong>时间:</strong> {}</p>
</body>
</html>"#,
                response.status_code, response.message, response.timestamp
            )
        }
    }

    /// 压缩响应内容
    pub fn compress_content(
        &self,
        content: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enable_compression {
            return Ok(content.as_bytes().to_vec());
        }

        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder =
            GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level));
        encoder.write_all(content.as_bytes())?;
        Ok(encoder.finish()?)
    }

    /// 设置缓存
    pub fn set_cache(
        &mut self,
        key: String,
        content: String,
        headers: HashMap<String, String>,
        ttl: Option<u64>,
    ) {
        if !self.config.cache_config.enabled {
            return;
        }

        let ttl_duration = Duration::from_secs(ttl.unwrap_or(self.config.cache_config.default_ttl));
        self.cache.insert(
            key,
            CachedResponse {
                content,
                headers,
                created_at: Instant::now(),
                ttl: ttl_duration,
            },
        );
    }

    /// 获取缓存
    pub fn get_cache(&self, key: &str) -> Option<(String, HashMap<String, String>)> {
        if !self.config.cache_config.enabled {
            return None;
        }

        if let Some(cached) = self.cache.get(key) {
            if cached.created_at.elapsed() < cached.ttl {
                return Some((cached.content.clone(), cached.headers.clone()));
            }
        }
        None
    }

    /// 清理过期缓存
    pub fn cleanup_expired_cache(&mut self) {
        let now = Instant::now();
        self.cache
            .retain(|_, cached| now.duration_since(cached.created_at) < cached.ttl);
    }

    /// 创建 Rocket 响应
    pub fn create_rocket_response<T: Serialize>(
        &self,
        response: &ApiResponse<T>,
        status: Status,
    ) -> Result<RocketResponse<'static>, Box<dyn std::error::Error + Send + Sync>> {
        let content = match self.config.error_config.format.as_str() {
            "html" => self.format_html(response),
            _ => self.format_json(response)?,
        };

        let content_type = match self.config.error_config.format.as_str() {
            "html" => ContentType::HTML,
            _ => ContentType::JSON,
        };

        // 尝试压缩内容
        let compressed_content = self.compress_content(&content)?;
        let should_compress = compressed_content.len() < content.len();

        let final_content = if should_compress {
            compressed_content
        } else {
            content.as_bytes().to_vec()
        };

        let mut rocket_response = RocketResponse::build()
            .status(status)
            .header(content_type)
            .sized_body(final_content.len(), Cursor::new(final_content))
            .finalize();

        if should_compress {
            rocket_response.set_header(Header::new("Content-Encoding", "gzip"));
        }

        Ok(rocket_response)
    }

    /// 获取配置
    pub fn get_config(&self) -> &ResponseConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ResponseConfig) {
        self.config = config;
    }
}

/// 分页数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedData<T> {
    /// 数据项
    pub items: Vec<T>,
    /// 分页信息
    pub pagination: PaginationInfo,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// 当前页码
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 总记录数
    pub total: u64,
    /// 总页数
    pub total_pages: u32,
    /// 是否有下一页
    pub has_next: bool,
    /// 是否有上一页
    pub has_prev: bool,
}

/// 响应构建器
pub struct ResponseBuilder {
    response: ApiResponse<serde_json::Value>,
    metadata: Option<ResponseMetadata>,
}

impl ResponseBuilder {
    /// 创建新的响应构建器
    pub fn new() -> Self {
        Self {
            response: ApiResponse {
                success: true,
                status_code: 200,
                message: "操作成功".to_string(),
                data: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
                request_id: None,
                error_details: None,
            },
            metadata: None,
        }
    }

    /// 设置成功状态
    pub fn success(mut self) -> Self {
        self.response.success = true;
        self.response.status_code = 200;
        self
    }

    /// 设置错误状态
    pub fn error(mut self, status_code: u16, message: String) -> Self {
        self.response.success = false;
        self.response.status_code = status_code;
        self.response.message = message;
        self
    }

    /// 设置数据
    pub fn data<T: Serialize>(mut self, data: T) -> Self {
        self.response.data = Some(serde_json::to_value(data).unwrap_or_default());
        self
    }

    /// 设置消息
    pub fn message(mut self, message: String) -> Self {
        self.response.message = message;
        self
    }

    /// 设置请求ID
    pub fn request_id(mut self, request_id: String) -> Self {
        self.response.request_id = Some(request_id);
        self
    }

    /// 设置元数据
    pub fn metadata(mut self, metadata: ResponseMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// 构建响应
    pub fn build(self) -> ApiResponse<serde_json::Value> {
        self.response
    }
}

/// 响应缓存管理器
pub struct ResponseCacheManager {
    cache: HashMap<String, CachedResponse>,
    max_size: usize,
}

impl ResponseCacheManager {
    /// 创建新的缓存管理器
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// 设置缓存
    pub fn set(
        &mut self,
        key: String,
        content: String,
        headers: HashMap<String, String>,
        ttl: Duration,
    ) {
        // 检查缓存大小限制
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }

        self.cache.insert(
            key,
            CachedResponse {
                content,
                headers,
                created_at: Instant::now(),
                ttl,
            },
        );
    }

    /// 获取缓存
    pub fn get(&self, key: &str) -> Option<(String, HashMap<String, String>)> {
        if let Some(cached) = self.cache.get(key) {
            if cached.created_at.elapsed() < cached.ttl {
                return Some((cached.content.clone(), cached.headers.clone()));
            }
        }
        None
    }

    /// 删除缓存
    pub fn remove(&mut self, key: &str) {
        self.cache.remove(key);
    }

    /// 清理过期缓存
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.cache
            .retain(|_, cached| now.duration_since(cached.created_at) < cached.ttl);
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        let total_size: usize = self.cache.values().map(|c| c.content.len()).sum();
        let expired_count = self
            .cache
            .values()
            .filter(|c| c.created_at.elapsed() >= c.ttl)
            .count();

        CacheStats {
            total_entries: self.cache.len(),
            total_size,
            expired_entries: expired_count,
        }
    }

    /// 驱逐最旧的缓存项
    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self
            .cache
            .iter()
            .min_by_key(|(_, cached)| cached.created_at)
        {
            self.cache.remove(oldest_key);
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总条目数
    pub total_entries: usize,
    /// 总大小（字节）
    pub total_size: usize,
    /// 过期条目数
    pub expired_entries: usize,
}

/// 响应追踪宏
#[macro_export]
macro_rules! trace_response {
    ($response:expr, $start_time:expr) => {
        log::debug!(
            "[响应追踪] 状态码: {} - 响应时间: {:.2}ms - 成功: {}",
            $response.status_code,
            $start_time.elapsed().as_secs_f64() * 1000.0,
            $response.success
        );
    };
}

/// 响应缓存宏
#[macro_export]
macro_rules! cache_response {
    ($cache_manager:expr, $key:expr, $content:expr, $headers:expr, $ttl:expr) => {
        $cache_manager.set($key, $content, $headers, $ttl);
    };
}

/// 响应错误宏
#[macro_export]
macro_rules! response_error {
    ($handler:expr, $status_code:expr, $message:expr, $request_id:expr) => {
        $handler.error($status_code, $message, None, $request_id)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_response_config_default() {
        let config = ResponseConfig::default();
        assert_eq!(config.default_content_type, "application/json");
        assert!(config.enable_compression);
        assert_eq!(config.compression_level, 6);
        assert!(config.cache_config.enabled);
    }

    #[test]
    fn test_response_handler_success() {
        let config = ResponseConfig::default();
        let handler = ResponseHandler::new(config);

        let data = serde_json::json!({"name": "test"});
        let response = handler.success(
            data,
            Some("测试成功".to_string()),
            Some("req_123".to_string()),
        );

        assert!(response.success);
        assert_eq!(response.status_code, 200);
        assert_eq!(response.message, "测试成功");
        assert!(response.request_id.is_some());
    }

    #[test]
    fn test_response_handler_error() {
        let config = ResponseConfig::default();
        let handler = ResponseHandler::new(config);

        let response = handler.error(
            404,
            "未找到".to_string(),
            Some("NotFound".to_string()),
            Some("req_123".to_string()),
        );

        assert!(!response.success);
        assert_eq!(response.status_code, 404);
        assert_eq!(response.message, "未找到");
        assert!(response.error_details.is_some());
    }

    #[test]
    fn test_response_builder() {
        let response = ResponseBuilder::new()
            .success()
            .data(serde_json::json!({"id": 1}))
            .message("创建成功".to_string())
            .request_id("req_123".to_string())
            .build();

        assert!(response.success);
        assert_eq!(response.status_code, 200);
        assert_eq!(response.message, "创建成功");
        assert!(response.request_id.is_some());
    }

    #[test]
    fn test_cache_manager() {
        let mut cache_manager = ResponseCacheManager::new(100);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        cache_manager.set(
            "test_key".to_string(),
            "test content".to_string(),
            headers.clone(),
            Duration::from_secs(60),
        );

        let cached = cache_manager.get("test_key");
        assert!(cached.is_some());

        let (content, cached_headers) = cached.unwrap();
        assert_eq!(content, "test content");
        assert_eq!(cached_headers, headers);
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo {
            page: 2,
            page_size: 10,
            total: 25,
            total_pages: 3,
            has_next: true,
            has_prev: true,
        };

        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 10);
        assert_eq!(pagination.total, 25);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }
}
