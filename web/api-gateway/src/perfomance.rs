use crate::debug;
use rocket::{get, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 性能汇总响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// 生成时间（RFC3339）
    pub generated_at: String,
    /// 请求统计（复用 DebugManager）
    pub request_stats: debug::RequestStats,
    /// 上游统计（复用 DebugManager）
    pub upstream_stats: Vec<debug::UpstreamStats>,
}

/// 基准测试请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkRequest {
    /// 循环次数
    pub iterations: u64,
}

/// 基准测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResponse {
    /// 循环次数
    pub iterations: u64,
    /// 耗时（毫秒）
    pub elapsed_ms: f64,
    /// 每次迭代平均耗时（纳秒）
    pub ns_per_iter: f64,
}

/// GET：获取性能汇总（来自 DebugManager 的统计）
#[get("/api/performance/summary")]
pub fn performance_summary(debug_manager: &State<debug::DebugManager>) -> Json<PerformanceSummary> {
    let info = debug_manager.get_debug_info();
    Json(PerformanceSummary {
        generated_at: chrono::Utc::now().to_rfc3339(),
        request_stats: info.request_stats,
        upstream_stats: info.upstream_stats,
    })
}

/// POST：重置性能统计（DebugManager）
#[post("/api/performance/reset")]
pub fn performance_reset(debug_manager: &State<debug::DebugManager>) -> Json<String> {
    debug_manager.reset_stats();
    Json("ok".to_string())
}

/// POST：跑一个简单 CPU 基准（用于粗略评估运行环境性能）
#[post("/api/performance/benchmark", data = "<request>")]
pub fn performance_benchmark(
    request: Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, Json<String>> {
    let iterations = request.iterations;
    if iterations == 0 {
        return Err(Json("iterations 必须大于 0".to_string()));
    }

    let start = Instant::now();

    // 防止编译器过度优化：做一点可重复的计算并累计
    let mut acc: u64 = 0;
    for i in 0..iterations {
        acc = acc.wrapping_add(i.wrapping_mul(31) ^ 0x9E3779B97F4A7C15u64);
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let ns_per_iter = (elapsed.as_secs_f64() * 1_000_000_000.0) / iterations as f64;

    // 再次使用 acc，确保不会被优化掉
    if acc == u64::MAX {
        return Err(Json("benchmark overflow sentinel hit".to_string()));
    }

    Ok(Json(BenchmarkResponse {
        iterations,
        elapsed_ms,
        ns_per_iter,
    }))
}

/// GET：示例基准请求
#[get("/api/performance/example")]
pub fn performance_example() -> Json<BenchmarkRequest> {
    Json(BenchmarkRequest {
        iterations: 5_000_000,
    })
}
