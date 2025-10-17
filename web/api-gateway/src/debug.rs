use serde::{Deserialize, Serialize};
/// API Gateway 调试模块
/// 提供调试工具、性能监控和诊断功能
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// 是否启用调试模式
    pub enabled: bool,
    /// 是否启用性能监控
    pub performance_monitoring: bool,
    /// 是否启用请求追踪
    pub request_tracing: bool,
    /// 调试端口（如果启用）
    pub debug_port: Option<u16>,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            performance_monitoring: false,
            request_tracing: false,
            debug_port: None,
        }
    }
}

/// 请求统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStats {
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
}

/// 上游服务器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamStats {
    /// 服务器URL
    pub url: String,
    /// 请求次数
    pub request_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 平均响应时间
    pub avg_response_time_ms: f64,
    /// 最后使用时间
    pub last_used: Option<String>,
}

/// 调试信息结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    /// 系统信息
    pub system_info: SystemInfo,
    /// 请求统计
    pub request_stats: RequestStats,
    /// 上游服务器统计
    pub upstream_stats: Vec<UpstreamStats>,
    /// 配置信息
    pub config: DebugConfig,
    /// 内存使用情况
    pub memory_usage: MemoryUsage,
}

/// 系统信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 启动时间
    pub start_time: String,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 当前时间
    pub current_time: String,
}

/// 内存使用情况
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// 已使用内存（MB）
    pub used_mb: f64,
    /// 总内存（MB）
    pub total_mb: f64,
    /// 使用百分比
    pub usage_percentage: f64,
}

/// 调试管理器
pub struct DebugManager {
    config: DebugConfig,
    request_stats: Arc<Mutex<RequestStats>>,
    upstream_stats: Arc<Mutex<HashMap<String, UpstreamStats>>>,
    start_time: Instant,
}

impl DebugManager {
    /// 创建新的调试管理器
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            request_stats: Arc::new(Mutex::new(RequestStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time_ms: 0.0,
                max_response_time_ms: 0.0,
                min_response_time_ms: f64::MAX,
            })),
            upstream_stats: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// 记录请求统计
    pub fn record_request(&self, success: bool, response_time_ms: f64, upstream_url: Option<&str>) {
        if !self.config.enabled {
            return;
        }

        // 更新总体统计
        if let Ok(mut stats) = self.request_stats.lock() {
            stats.total_requests += 1;
            if success {
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
            let total_time =
                stats.avg_response_time_ms * (stats.total_requests - 1) as f64 + response_time_ms;
            stats.avg_response_time_ms = total_time / stats.total_requests as f64;
        }

        // 更新上游服务器统计
        if let Some(url) = upstream_url {
            if let Ok(mut upstream_stats) = self.upstream_stats.lock() {
                let entry =
                    upstream_stats
                        .entry(url.to_string())
                        .or_insert_with(|| UpstreamStats {
                            url: url.to_string(),
                            request_count: 0,
                            success_count: 0,
                            failure_count: 0,
                            avg_response_time_ms: 0.0,
                            last_used: None,
                        });

                entry.request_count += 1;
                if success {
                    entry.success_count += 1;
                } else {
                    entry.failure_count += 1;
                }

                // 更新平均响应时间
                let total_time = entry.avg_response_time_ms * (entry.request_count - 1) as f64
                    + response_time_ms;
                entry.avg_response_time_ms = total_time / entry.request_count as f64;
                entry.last_used = Some(chrono::Utc::now().to_rfc3339());
            }
        }
    }

    /// 获取调试信息
    pub fn get_debug_info(&self) -> DebugInfo {
        let request_stats = self.request_stats.lock().unwrap().clone();
        let upstream_stats: Vec<UpstreamStats> = self
            .upstream_stats
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect();

        let uptime = self.start_time.elapsed();
        let system_info = SystemInfo {
            start_time: "系统启动时间".to_string(), // 这里可以记录实际的启动时间
            uptime_seconds: uptime.as_secs(),
            current_time: chrono::Utc::now().to_rfc3339(),
        };

        // 获取内存使用情况（简化版）
        let memory_usage = MemoryUsage {
            used_mb: 0.0, // 这里可以集成 sysinfo 获取实际内存使用
            total_mb: 0.0,
            usage_percentage: 0.0,
        };

        DebugInfo {
            system_info,
            request_stats,
            upstream_stats,
            config: self.config.clone(),
            memory_usage,
        }
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.request_stats.lock() {
            *stats = RequestStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time_ms: 0.0,
                max_response_time_ms: 0.0,
                min_response_time_ms: f64::MAX,
            };
        }

        if let Ok(mut upstream_stats) = self.upstream_stats.lock() {
            upstream_stats.clear();
        }
    }

    /// 检查是否启用调试
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 检查是否启用性能监控
    pub fn is_performance_monitoring_enabled(&self) -> bool {
        self.config.performance_monitoring
    }
}

/// 性能监控宏
/// 用于测量代码执行时间
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed();
        log::debug!(
            "[性能监控] {} 执行时间: {:.2}ms",
            $name,
            duration.as_secs_f64() * 1000.0
        );
        result
    }};
}

/// 调试日志宏
/// 只在调试模式下输出
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        log::debug!($($arg)*);
    };
}

/// 请求追踪宏
/// 记录请求的详细信息
#[macro_export]
macro_rules! trace_request {
    ($method:expr, $path:expr, $headers:expr) => {
        log::debug!("[请求追踪] {} {} - Headers: {:?}", $method, $path, $headers);
    };
}

/// 上游调用追踪宏
#[macro_export]
macro_rules! trace_upstream {
    ($upstream:expr, $method:expr, $path:expr) => {
        log::debug!("[上游追踪] 调用 {} - {} {}", $upstream, $method, $path);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_manager_creation() {
        let config = DebugConfig {
            enabled: true,
            performance_monitoring: true,
            request_tracing: true,
            debug_port: Some(8080),
        };
        let manager = DebugManager::new(config);
        assert!(manager.is_enabled());
        assert!(manager.is_performance_monitoring_enabled());
    }

    #[test]
    fn test_request_recording() {
        let config = DebugConfig {
            enabled: true,
            performance_monitoring: true,
            request_tracing: true,
            debug_port: None,
        };
        let manager = DebugManager::new(config);

        manager.record_request(true, 100.0, Some("http://test.com"));
        manager.record_request(false, 200.0, Some("http://test.com"));

        let debug_info = manager.get_debug_info();
        assert_eq!(debug_info.request_stats.total_requests, 2);
        assert_eq!(debug_info.request_stats.successful_requests, 1);
        assert_eq!(debug_info.request_stats.failed_requests, 1);
    }

    #[test]
    fn test_stats_reset() {
        let config = DebugConfig {
            enabled: true,
            performance_monitoring: true,
            request_tracing: true,
            debug_port: None,
        };
        let manager = DebugManager::new(config);

        manager.record_request(true, 100.0, Some("http://test.com"));
        manager.reset_stats();

        let debug_info = manager.get_debug_info();
        assert_eq!(debug_info.request_stats.total_requests, 0);
    }
}
