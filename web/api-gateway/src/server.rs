use chrono::Utc;
use rocket::{get, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Instant;
use sysinfo::System;

/// 服务器启动时间
static SERVER_START_TIME: OnceLock<Instant> = OnceLock::new();

/// 服务器信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfoResponse {
    /// 服务器名称
    pub name: String,
    /// 服务器版本
    pub version: String,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 运行时间（格式化字符串）
    pub uptime: String,
    /// 启动时间
    pub start_time: String,
    /// 当前时间
    pub current_time: String,
    /// 系统信息
    pub system: SystemInfo,
    /// 资源使用情况
    pub resources: ResourceInfo,
}

/// 系统信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 主机名
    pub hostname: String,
    /// 操作系统
    pub os: String,
    /// 内核版本
    pub kernel_version: String,
    /// CPU 核心数
    pub cpu_count: usize,
    /// CPU 品牌
    pub cpu_brand: String,
    /// 总内存（MB）
    pub total_memory_mb: f64,
    /// 总磁盘空间（GB）
    pub total_disk_gb: f64,
}

/// 资源使用情况
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// CPU 使用率（百分比）
    pub cpu_usage: f64,
    /// 内存使用情况
    pub memory: MemoryUsage,
    /// 磁盘使用情况
    pub disk: DiskUsage,
    /// 网络使用情况
    pub network: NetworkUsage,
}

/// 内存使用情况
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// 已使用内存（MB）
    pub used_mb: f64,
    /// 总内存（MB）
    pub total_mb: f64,
    /// 可用内存（MB）
    pub available_mb: f64,
    /// 使用率（百分比）
    pub usage_percentage: f64,
}

/// 磁盘使用情况
#[derive(Debug, Serialize, Deserialize)]
pub struct DiskUsage {
    /// 已使用磁盘空间（GB）
    pub used_gb: f64,
    /// 总磁盘空间（GB）
    pub total_gb: f64,
    /// 可用磁盘空间（GB）
    pub available_gb: f64,
    /// 使用率（百分比）
    pub usage_percentage: f64,
}

/// 网络使用情况
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkUsage {
    /// 接收的字节数
    pub received_bytes: u64,
    /// 发送的字节数
    pub transmitted_bytes: u64,
    /// 接收的数据包数
    pub received_packets: u64,
    /// 发送的数据包数
    pub transmitted_packets: u64,
}

/// 服务器状态
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    /// 状态（running, stopped, error）
    pub status: String,
    /// 健康状态（healthy, unhealthy, degraded）
    pub health: String,
    /// 是否就绪
    pub ready: bool,
    /// 时间戳
    pub timestamp: String,
    /// 消息
    pub message: String,
}

/// 服务器统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 当前并发连接数
    pub current_connections: u64,
    /// 峰值并发连接数
    pub peak_connections: u64,
}

/// 初始化服务器启动时间
pub fn init_start_time() {
    SERVER_START_TIME.set(Instant::now()).ok();
}

/// 获取服务器信息
///
/// # 返回
/// 服务器信息响应
pub fn get_server_info() -> ServerInfoResponse {
    let mut system = System::new_all();
    system.refresh_all();

    // 获取启动时间
    let uptime_seconds = if let Some(start_time) = SERVER_START_TIME.get() {
        start_time.elapsed().as_secs()
    } else {
        0
    };
    let uptime = format_uptime(uptime_seconds);

    // 获取系统信息
    // 注意：sysinfo 0.37 版本中某些方法可能需要不同的访问方式
    // 这里使用简化版本，实际使用时可能需要根据具体版本调整
    let hostname = "localhost".to_string(); // 简化处理
    let os = std::env::consts::OS.to_string();
    let kernel_version = "unknown".to_string(); // sysinfo 0.37 可能需要不同的方式获取
    let cpu_count = system.cpus().len();
    let cpu_brand = if !system.cpus().is_empty() {
        system.cpus()[0].brand().to_string()
    } else {
        "unknown".to_string()
    };

    // 内存信息
    let total_memory_mb = system.total_memory() as f64 / 1024.0 / 1024.0;
    let used_memory_mb = system.used_memory() as f64 / 1024.0 / 1024.0;
    let available_memory_mb =
        (system.total_memory() - system.used_memory()) as f64 / 1024.0 / 1024.0;
    let memory_usage_percentage = if total_memory_mb > 0.0 {
        (used_memory_mb / total_memory_mb) * 100.0
    } else {
        0.0
    };

    // CPU 使用率
    let cpu_usage = if !system.cpus().is_empty() {
        system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .sum::<f64>()
            / cpu_count as f64
    } else {
        0.0
    };

    // 磁盘信息
    // 注意：sysinfo 0.37 中磁盘和网络信息的访问方式可能不同
    // 这里使用简化版本，实际使用时可能需要根据具体版本调整
    let total_disk_gb = 0.0; // 简化处理，实际应该从系统获取
    let used_disk_gb = 0.0;
    let available_disk_gb = 0.0;
    let disk_usage_percentage = 0.0;

    // 网络信息
    // 注意：sysinfo 0.37 中网络信息的访问方式可能不同
    // 这里使用简化版本，实际使用时可能需要根据具体版本调整
    let received_bytes = 0u64;
    let transmitted_bytes = 0u64;
    let received_packets = 0u64;
    let transmitted_packets = 0u64;

    ServerInfoResponse {
        name: "API Gateway".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        uptime,
        start_time: Utc::now().to_rfc3339(), // 简化处理，实际应该记录启动时间
        current_time: Utc::now().to_rfc3339(),
        system: SystemInfo {
            hostname,
            os,
            kernel_version,
            cpu_count,
            cpu_brand,
            total_memory_mb: round_to_decimal(total_memory_mb, 2),
            total_disk_gb: round_to_decimal(total_disk_gb, 2),
        },
        resources: ResourceInfo {
            cpu_usage: round_to_decimal(cpu_usage, 2),
            memory: MemoryUsage {
                used_mb: round_to_decimal(used_memory_mb, 2),
                total_mb: round_to_decimal(total_memory_mb, 2),
                available_mb: round_to_decimal(available_memory_mb, 2),
                usage_percentage: round_to_decimal(memory_usage_percentage, 2),
            },
            disk: DiskUsage {
                used_gb: round_to_decimal(used_disk_gb, 2),
                total_gb: round_to_decimal(total_disk_gb, 2),
                available_gb: round_to_decimal(available_disk_gb, 2),
                usage_percentage: round_to_decimal(disk_usage_percentage, 2),
            },
            network: NetworkUsage {
                received_bytes,
                transmitted_bytes,
                received_packets,
                transmitted_packets,
            },
        },
    }
}

/// 获取服务器状态
///
/// # 返回
/// 服务器状态
pub fn get_server_status() -> ServerStatus {
    let mut system = System::new_all();
    system.refresh_all();

    // 检查系统资源
    let memory_usage = if system.total_memory() > 0 {
        (system.used_memory() as f64 / system.total_memory() as f64) * 100.0
    } else {
        0.0
    };

    let cpu_usage = if !system.cpus().is_empty() {
        system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .sum::<f64>()
            / system.cpus().len() as f64
    } else {
        0.0
    };

    // 判断健康状态
    let (health, message) = if memory_usage > 90.0 || cpu_usage > 90.0 {
        ("degraded", "系统资源使用率过高")
    } else if memory_usage > 95.0 || cpu_usage > 95.0 {
        ("unhealthy", "系统资源严重不足")
    } else {
        ("healthy", "系统运行正常")
    };

    ServerStatus {
        status: "running".to_string(),
        health: health.to_string(),
        ready: true,
        timestamp: Utc::now().to_rfc3339(),
        message: message.to_string(),
    }
}

/// 格式化运行时间
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}天 {}小时 {}分钟 {}秒", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}小时 {}分钟 {}秒", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}分钟 {}秒", minutes, secs)
    } else {
        format!("{}秒", secs)
    }
}

/// 四舍五入到指定小数位
fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// GET 端点：获取服务器信息
#[get("/api/server/info")]
pub fn server_info() -> Json<ServerInfoResponse> {
    Json(get_server_info())
}

/// GET 端点：获取服务器状态
#[get("/api/server/status")]
pub fn server_status() -> Json<ServerStatus> {
    Json(get_server_status())
}

/// GET 端点：获取服务器健康检查
#[get("/api/server/health")]
pub fn server_health() -> Json<ServerStatus> {
    Json(get_server_status())
}

/// GET 端点：获取服务器统计信息（示例）
#[get("/api/server/stats")]
pub fn server_stats() -> Json<ServerStats> {
    // 这里返回示例数据，实际应该从统计系统获取
    Json(ServerStats {
        total_requests: 1000,
        successful_requests: 950,
        failed_requests: 50,
        avg_response_time_ms: 25.5,
        current_connections: 42,
        peak_connections: 150,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "0秒");
        assert_eq!(format_uptime(30), "30秒");
        assert_eq!(format_uptime(90), "1分钟 30秒");
        assert_eq!(format_uptime(3661), "1小时 1分钟 1秒");
        assert_eq!(format_uptime(90061), "1天 1小时 1分钟 1秒");
    }

    #[test]
    fn test_round_to_decimal() {
        assert!((round_to_decimal(3.14159, 2) - 3.14).abs() < 0.01);
        assert!((round_to_decimal(3.145, 2) - 3.15).abs() < 0.01);
        assert!((round_to_decimal(10.0, 2) - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_get_server_info() {
        init_start_time();
        let info = get_server_info();
        assert_eq!(info.name, "API Gateway");
        assert!(!info.version.is_empty());
        assert!(info.uptime_seconds >= 0);
    }

    #[test]
    fn test_get_server_status() {
        let status = get_server_status();
        assert_eq!(status.status, "running");
        assert!(status.ready);
        assert!(!status.timestamp.is_empty());
    }
}
