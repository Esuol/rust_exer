/// API Gateway 日志模块
/// 提供统一的日志初始化和配置功能
use log::LevelFilter;
use std::io::Write;

/// 日志配置结构体
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式：text 或 json
    pub format: String,
}

/// 初始化日志系统
///
/// # 参数
/// * `config` - 日志配置
///
/// # 示例
/// ```
/// let config = LogConfig {
///     level: "info".to_string(),
///     format: "text".to_string(),
/// };
/// init_logger(&config);
/// ```
pub fn init_logger(config: &LogConfig) {
    let level = parse_log_level(&config.level);

    let builder = env_logger::Builder::new();

    match config.format.as_str() {
        "json" => {
            // JSON 格式的日志
            builder
                .filter_level(level)
                .format(|buf, record| {
                    writeln!(
                        buf,
                        r#"{{"timestamp":"{}","level":"{}","target":"{}","message":"{}"}}"#,
                        chrono::Utc::now().to_rfc3339(),
                        record.level(),
                        record.target(),
                        record.args()
                    )
                })
                .init();
        }
        _ => {
            // 默认文本格式
            builder
                .filter_level(level)
                .format_timestamp_secs()
                .format(|buf, record| {
                    writeln!(
                        buf,
                        "[{} {} {}] {}",
                        buf.timestamp_seconds(),
                        record.level(),
                        record.target(),
                        record.args()
                    )
                })
                .init();
        }
    }

    log::info!(
        "日志系统初始化完成 - 级别: {}, 格式: {}",
        config.level,
        config.format
    );
}

/// 解析日志级别字符串
///
/// # 参数
/// * `level_str` - 日志级别字符串 (error, warn, info, debug, trace)
///
/// # 返回
/// 对应的 LevelFilter 枚举
fn parse_log_level(level_str: &str) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "off" => LevelFilter::Off,
        _ => {
            eprintln!("未知的日志级别 '{}', 使用默认级别 'info'", level_str);
            LevelFilter::Info
        }
    }
}

/// 记录请求日志的辅助宏
/// 用于统一格式化 HTTP 请求日志
#[macro_export]
macro_rules! log_request {
    ($method:expr, $path:expr, $status:expr, $duration:expr) => {
        log::info!(
            "REQUEST {} {} - {} ({:.2}ms)",
            $method,
            $path,
            $status,
            $duration
        );
    };
}

/// 记录错误日志的辅助宏
/// 用于统一格式化错误信息
#[macro_export]
macro_rules! log_error {
    ($context:expr, $error:expr) => {
        log::error!("[{}] 错误: {}", $context, $error);
    };
}

/// 记录上游服务调用的辅助宏
#[macro_export]
macro_rules! log_upstream {
    ($upstream:expr, $success:expr) => {
        if $success {
            log::debug!("上游服务调用成功: {}", $upstream);
        } else {
            log::warn!("上游服务调用失败: {}", $upstream);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("error"), LevelFilter::Error);
        assert_eq!(parse_log_level("warn"), LevelFilter::Warn);
        assert_eq!(parse_log_level("info"), LevelFilter::Info);
        assert_eq!(parse_log_level("debug"), LevelFilter::Debug);
        assert_eq!(parse_log_level("trace"), LevelFilter::Trace);
        assert_eq!(parse_log_level("off"), LevelFilter::Off);
        assert_eq!(parse_log_level("unknown"), LevelFilter::Info);
    }

    #[test]
    fn test_parse_log_level_case_insensitive() {
        assert_eq!(parse_log_level("ERROR"), LevelFilter::Error);
        assert_eq!(parse_log_level("WaRn"), LevelFilter::Warn);
        assert_eq!(parse_log_level("INFO"), LevelFilter::Info);
    }
}
