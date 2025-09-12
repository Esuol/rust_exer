use chrono::Utc;
use rocket::{get, launch, post, routes, serde::json::Json, State};
use std::path::PathBuf;
use sysinfo::{Cpu, System};
// 添加一个静态变量记录启动时间
// 改为使用 OnceLock
use log::{debug, LevelFilter};
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static START_TIME: OnceLock<Instant> = OnceLock::new();

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

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    logging: LoggingConfig,
    routes: RoutesConfig, // 添加路由配置
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
    format: String,
}

#[derive(Debug, Deserialize)]
struct RouteConfig {
    path: String,
    method: String,
    upstream: String,
    timeout: u64,
}

#[derive(Debug, Deserialize)]
struct RoutesConfig {
    routes: Vec<RouteConfig>,
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

    // 计算运行时间
    let uptime_seconds = START_TIME.get().unwrap().elapsed().as_secs();
    let uptime = format!(
        "{:02}:{:02}:{:02}",
        uptime_seconds / 3600,
        (uptime_seconds % 3600) / 60,
        uptime_seconds % 60
    );

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
async fn proxy(path: PathBuf, config: &State<AppConfig>) -> Result<String, rocket::http::Status> {
    let request_path = format!("/{}", path.display());
    let method = "GET"; // 暂时只支持GET，后面扩展

    // 查找匹配的路由
    if let Some(route) = find_route(&config.routes.routes, &request_path, method) {
        let client = reqwest::Client::new();

        match client
            .get(&route.upstream)
            .timeout(Duration::from_secs(route.timeout))
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(text) => Ok(text),
                Err(_) => Err(rocket::http::Status::InternalServerError),
            },
            Err(_) => Err(rocket::http::Status::BadGateway),
        }
    } else {
        Err(rocket::http::Status::NotFound)
    }
}

// 加载配置
fn load_config() -> Result<AppConfig, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config/default.toml"))
        .build()?;

    settings.try_deserialize::<AppConfig>()
}

// 初始化日志
fn init_logging(config: &LoggingConfig) {
    let level = match config.level.as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp_secs()
        .init();
}

// 查找路由
fn find_route<'a>(routes: &'a [RouteConfig], path: &str, method: &str) -> Option<&'a RouteConfig> {
    routes.iter().find(|route| {
        // 路径匹配（支持通配符）
        if route.path.ends_with("/*") {
            let prefix = route.path.trim_end_matches("/*");
            if !path.starts_with(prefix) {
                return false;
            }
        } else if route.path != path {
            return false;
        }

        // 方法匹配
        if route.method == "*" {
            return true;
        }

        route.method.split('|').any(|m| m.trim() == method)
    })
}

#[launch]
fn rocket() -> _ {
    // 初始化启动时间
    START_TIME.set(Instant::now()).unwrap();
    // 加载配置
    let config = load_config().expect("Failed to load config");
    // 初始化日志系统
    init_logging(&config.logging);
    log::info!(
        "API Gateway starting on {}:{}",
        config.server.host,
        config.server.port
    );

    rocket::build()
        .configure(rocket::Config {
            address: config.server.host.parse().unwrap(),
            port: config.server.port,
            workers: config.server.workers,
            ..Default::default()
        })
        .manage(config) // 添加状态管理
        .mount("/", routes![index, health, proxy])
}
