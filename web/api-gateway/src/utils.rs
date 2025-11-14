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

// ==================== 数组处理函数 ====================

/// 去除数组中的重复元素
#[allow(dead_code)]
pub fn deduplicate<T: Clone + Eq + std::hash::Hash>(vec: &[T]) -> Vec<T> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    vec.iter()
        .filter(|item| seen.insert(*item))
        .cloned()
        .collect()
}

/// 将数组分块，每块大小为 chunk_size
#[allow(dead_code)]
pub fn chunk<T: Clone>(vec: &[T], chunk_size: usize) -> Vec<Vec<T>> {
    vec.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect()
}

/// 过滤数组中的空值（None）
#[allow(dead_code)]
pub fn filter_none<T>(vec: Vec<Option<T>>) -> Vec<T> {
    vec.into_iter().flatten().collect()
}

/// 查找数组中第一个满足条件的元素
#[allow(dead_code)]
pub fn find_first<T, F>(vec: &[T], predicate: F) -> Option<&T>
where
    F: Fn(&T) -> bool,
{
    vec.iter().find(|item| predicate(item))
}

/// 数组切片（安全版本，自动处理边界）
#[allow(dead_code)]
pub fn safe_slice<T: Clone>(vec: &[T], start: usize, end: usize) -> Vec<T> {
    let start = start.min(vec.len());
    let end = end.min(vec.len());
    if start >= end {
        return Vec::new();
    }
    vec[start..end].to_vec()
}

/// 数组合并（去重）
#[allow(dead_code)]
pub fn merge_unique<T: Clone + Eq + std::hash::Hash>(vec1: &[T], vec2: &[T]) -> Vec<T> {
    use std::collections::HashSet;
    let mut set: HashSet<&T> = HashSet::new();
    let mut result = Vec::new();

    for item in vec1.iter().chain(vec2.iter()) {
        if set.insert(item) {
            result.push(item.clone());
        }
    }
    result
}

// ==================== 数字处理函数 ====================

/// 格式化数字，添加千分位分隔符
#[allow(dead_code)]
pub fn format_number(num: i64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();

    for (i, ch) in num_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    result.chars().rev().collect()
}

/// 格式化浮点数，添加千分位分隔符并保留小数位
#[allow(dead_code)]
pub fn format_float(num: f64, decimals: usize) -> String {
    let formatted = format!("{:.*}", decimals, num);
    let parts: Vec<&str> = formatted.split('.').collect();

    if parts.len() == 2 {
        let int_part = format_number(parts[0].parse().unwrap_or(0));
        format!("{}.{}", int_part, parts[1])
    } else {
        format_number(num as i64)
    }
}

/// 四舍五入到指定小数位
#[allow(dead_code)]
pub fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// 计算百分比
#[allow(dead_code)]
pub fn calculate_percentage(part: f64, total: f64) -> f64 {
    if total == 0.0 {
        return 0.0;
    }
    (part / total) * 100.0
}

/// 检查数字是否在指定范围内
#[allow(dead_code)]
pub fn in_range(num: f64, min: f64, max: f64) -> bool {
    num >= min && num <= max
}

/// 将数字限制在指定范围内
#[allow(dead_code)]
pub fn clamp(num: f64, min: f64, max: f64) -> f64 {
    if num < min {
        min
    } else if num > max {
        max
    } else {
        num
    }
}

/// 数字转换：字符串转数字（安全版本）
#[allow(dead_code)]
pub fn parse_number<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.trim().parse().ok()
}

// ==================== 字符串处理函数 ====================

/// 截断字符串到指定长度，如果超出则添加省略号
#[allow(dead_code)]
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return ".".repeat(max_len);
    }
    format!("{}...", &s[..max_len - 3])
}

/// 首字母大写
#[allow(dead_code)]
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// 去除字符串首尾空白
#[allow(dead_code)]
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// 去除字符串中的所有空白字符
#[allow(dead_code)]
pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 分割字符串并返回向量
#[allow(dead_code)]
pub fn split_string(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(|s| s.to_string()).collect()
}

/// 替换字符串中的所有匹配项
#[allow(dead_code)]
pub fn replace_all(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// 字符串反转
#[allow(dead_code)]
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

/// 检查字符串是否为空或只包含空白
#[allow(dead_code)]
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// 将字符串转换为驼峰命名
#[allow(dead_code)]
pub fn to_camel_case(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let first = words[0].to_lowercase();
    let rest: String = words[1..].iter().map(|word| capitalize(word)).collect();
    format!("{}{}", first, rest)
}

/// 将字符串转换为蛇形命名（snake_case）
#[allow(dead_code)]
pub fn to_snake_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_uppercase() {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_string()
            }
        })
        .collect::<String>()
        .to_lowercase()
        .trim_start_matches('_')
        .to_string()
}

/// 提取字符串中的所有数字（连续的数字字符会被解析为一个数字）
#[allow(dead_code)]
pub fn extract_numbers(s: &str) -> Vec<i64> {
    let mut result = Vec::new();
    let mut current_number = String::new();

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current_number.push(ch);
        } else if !current_number.is_empty() {
            if let Ok(num) = current_number.parse::<i64>() {
                result.push(num);
            }
            current_number.clear();
        }
    }

    // 处理字符串末尾的数字
    if !current_number.is_empty() {
        if let Ok(num) = current_number.parse::<i64>() {
            result.push(num);
        }
    }

    result
}

/// 检查字符串是否以指定前缀开头（不区分大小写）
#[allow(dead_code)]
pub fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// 检查字符串是否以指定后缀结尾（不区分大小写）
#[allow(dead_code)]
pub fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
    s.len() >= suffix.len() && s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}
