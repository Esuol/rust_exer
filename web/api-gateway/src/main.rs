use chrono::Utc;
use rocket::{get, launch, post, routes, serde::json::Json};
use std::path::PathBuf;
use sysinfo::{Cpu, System};

// 健康检查响应结构体
#[derive(serde::Serialize)] //  自动生成JSON序列化代码
struct HealthResponse {
    status: String,
    timestamp: String,
    uptime: String,
    memory: MemoryInfo,
    cpu: CpuInfo,
}

#[derive(serde::Serialize)]
struct MemoryInfo {
    used_mb: f64,
    total_mb: f64,
    usage_percentage: f64,
}

#[derive(serde::Serialize)]
struct CpuInfo {
    usage_percentage: f64,
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to API Gateway!"
}

#[get("/health")]
fn health() -> Json<HealthResponse> {
    // 获取当前时间
    let timestamp = Utc::now().to_rfc3339();

    //  初始化系统信息收集器
    let mut system = System::new_all();
    system.refresh_all();

    // 收集内存信息
    let total_memory_mb = system.total_memory() as f64 / 1024.0 / 1024.0;
    let used_memory_mb = system.used_memory() as f64 / 1024.0 / 1024.0;
    let memory_usage_percentage = (used_memory_mb / total_memory_mb) * 100.0;

    let memory_info: MemoryInfo = MemoryInfo {
        used_mb: (used_memory_mb * 100.0).round() / 100.0, // 保留2位小数
        total_mb: (total_memory_mb * 100.0).round() / 100.0,
        usage_percentage: (memory_usage_percentage * 100.0).round() / 100.0,
    };

    // 收集CPU信息
    let cpu_usage = system
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage() as f64)
        .sum::<f64>()
        / system.cpus().len() as f64;

    let cpu_info = CpuInfo {
        usage_percentage: (cpu_usage * 100.0).round() / 100.0,
    };

    // 计算运行时间（这里简化为当前时间，后面可以改进）
    let uptime = "00:00:00".to_string(); // 暂时用固定值，后面再改进

    // 构建响应
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp,
        uptime,
        memory: memory_info,
        cpu: cpu_info,
    };

    Json(response)
}

#[post("/proxy/<path..>")]
async fn proxy(path: PathBuf) -> Result<String, rocket::http::Status> {
    let target_url = format!("http://httpbin.org/{}", path.display());
    println!("target_url: {}", target_url);
    let client = reqwest::Client::new();

    match client.get(&target_url).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => Ok(text),
            Err(_) => Err(rocket::http::Status::InternalServerError),
        },
        Err(_) => Err(rocket::http::Status::BadGateway),
    }
}

#[launch] // entry point 启动宏
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, health, proxy]) // 注册路由
}
