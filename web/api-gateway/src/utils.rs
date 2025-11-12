use chrono::{DateTime, Utc};
use url::Url;

/// 解析和规范化 URL
#[allow(dead_code)] // 预留功能，未来可能使用
pub fn normalize_url(url: &str) -> Result<String, url::ParseError> {
    let parsed = Url::parse(url)?;
    Ok(parsed.to_string())
}

/// 获取当前时间的 RFC3339 格式字符串
#[allow(dead_code)] // 预留功能，未来可能使用
pub fn current_timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}

/// 检查 URL 是否有效
#[allow(dead_code)] // 预留功能，未来可能使用
pub fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

/// 从 URL 中提取域名
#[allow(dead_code)] // 预留功能，未来可能使用
pub fn extract_domain(url: &str) -> Option<String> {
    match Url::parse(url) {
        Ok(parsed) => parsed.host_str().map(|h| h.to_string()),
        Err(_) => None,
    }
}
