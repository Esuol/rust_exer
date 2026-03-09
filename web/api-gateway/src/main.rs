use chrono::Utc;
use rocket::{get, launch, post, routes, serde::json::Json, State};
use std::path::PathBuf;
use sysinfo::System;
// 添加一个静态变量记录启动时间
// 改为使用 OnceLock
use serde::Deserialize;
// AtomicUsize: 线程安全的计数器，用于轮询算法
// Ordering: 内存顺序保证
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

// 导入自定义模块
mod api;
mod calc;
mod code;
mod debug;
mod error;
mod http;
mod keep;
mod logger;
mod pie;
mod pop;
mod proxy;
mod request;
mod response;
mod select;
mod server;
mod service;
mod table;
mod utils;
mod warning;

static START_TIME: OnceLock<Instant> = OnceLock::new();

// 健康检查响应结构体
// 自动生成JSON序列化代码
#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    uptime: String,
    memory: MemoryInfo,
    cpu: CpuInfo,
}

// 内存信息结构体
#[derive(serde::Serialize)]
struct MemoryInfo {
    used_mb: f64,
    total_mb: f64,
    usage_percentage: f64,
}

// CPU信息结构体
#[derive(serde::Serialize)]
struct CpuInfo {
    usage_percentage: f64,
}

// 应用配置结构体
#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    logging: LoggingConfig,
    #[serde(default)]
    debug: debug::DebugConfig, // 添加调试配置
    #[serde(default)]
    routes: Vec<RouteConfig>, // 直接使用Vec<RouteConfig>，提供默认值
}

// 服务器配置结构体
#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
    #[allow(dead_code)]
    format: String,
}

#[derive(Debug, Deserialize)]
struct RouteConfig {
    path: String,
    method: String,
    upstreams: Vec<UpstreamServer>, // 支持多个上游服务器
    timeout: u64,
    load_balance: String, // 负载均衡算法："round_robin", "weighted", "least_conn"
}

#[derive(Debug, Deserialize)]
struct UpstreamServer {
    url: String,
    weight: u32, // 用于加权轮询
}

// 移除RoutesConfig，直接使用Vec<RouteConfig>

#[get("/")]
fn index() -> &'static str {
    "Welcome to API Gateway!"
}

#[get("/debug")]
fn debug_info(debug_manager: &State<debug::DebugManager>) -> Json<debug::DebugInfo> {
    Json(debug_manager.get_debug_info())
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
async fn proxy_request(
    path: PathBuf,
    config: &State<AppConfig>,
    debug_manager: &State<debug::DebugManager>,
) -> Result<String, error::ApiGatewayError> {
    let request_path = format!("/{}", path.display());
    let method = "GET"; // 暂时只支持GET，后面扩展
    let start_time = Instant::now();

    // 使用标准日志宏
    log::debug!(
        "Looking for route: {} with method: {}",
        request_path,
        method
    );
    log::debug!("Available routes: {}", config.routes.len());

    if let Some(route) = find_route(&config.routes, &request_path, method) {
        log::debug!("Found matching route: {}", route.path);
        if let Some(upstream) = select_upstream(route) {
            // 使用调试追踪宏
            trace_upstream!(&upstream.url, method, &request_path);

            let client = reqwest::Client::new();
            let response_result = client
                .get(&upstream.url) // 使用负载均衡选择的上游
                .timeout(Duration::from_secs(route.timeout))
                .send()
                .await;

            let response_time = start_time.elapsed();
            let response_time_ms = response_time.as_secs_f64() * 1000.0;

            match response_result {
                Ok(response) => {
                    match response.text().await {
                        Ok(text) => {
                            // 记录成功的请求统计
                            debug_manager.record_request(
                                true,
                                response_time_ms,
                                Some(&upstream.url),
                            );
                            log_request!(method, &request_path, "200", response_time_ms);
                            Ok(text)
                        }
                        Err(_) => {
                            // 记录失败的请求统计
                            debug_manager.record_request(
                                false,
                                response_time_ms,
                                Some(&upstream.url),
                            );
                            log_error!("响应解析", "无法解析响应文本");
                            Err(error::ApiGatewayError::internal_error("无法解析响应文本"))
                        }
                    }
                }
                Err(_) => {
                    // 记录失败的请求统计
                    debug_manager.record_request(false, response_time_ms, Some(&upstream.url));
                    log_error!("上游调用", "网络请求失败");
                    Err(error::ApiGatewayError::bad_gateway("网络请求失败"))
                }
            }
        } else {
            Err(error::ApiGatewayError::upstream_unavailable(
                "No upstream available",
            ))
        }
    } else {
        Err(error::ApiGatewayError::route_not_found(&request_path))
    }
}

#[get("/test-error/<error_type>")]
fn test_error(error_type: &str) -> Result<String, error::ApiGatewayError> {
    match error_type {
        "not-found" => Err(error::ApiGatewayError::route_not_found("/test/path")),
        "upstream-unavailable" => Err(error::ApiGatewayError::upstream_unavailable(
            "http://example.com",
        )),
        "timeout" => Err(error::ApiGatewayError::request_timeout(
            "http://example.com",
        )),
        "internal" => Err(error::ApiGatewayError::internal_error(
            "Test internal error",
        )),
        "bad-gateway" => Err(error::ApiGatewayError::bad_gateway("Test bad gateway")),
        _ => Ok("Unknown error type".to_string()),
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
    let log_config = logger::LogConfig {
        level: config.level.clone(),
        format: config.format.clone(),
    };

    logger::init_logger(&log_config);
}

// 查找路由
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

// 这是入口函数，根据算法选择不同的负载均衡策略
fn select_upstream(route: &RouteConfig) -> Option<&UpstreamServer> {
    if route.upstreams.is_empty() {
        return None;
    }

    match route.load_balance.as_str() {
        "round_robin" => select_round_robin(route),
        "weighted" => select_weighted(route),
        "least_conn" => select_least_conn(route),
        _ => select_round_robin(route), // 默认使用轮询
    }
}

// 实现轮询算法
fn select_round_robin(route: &RouteConfig) -> Option<&UpstreamServer> {
    // 使用静态原子计数器来跟踪轮询位置
    // 这需要在文件顶部添加 static 变量
    static ROUND_ROBIN_INDEX: AtomicUsize = AtomicUsize::new(0);

    let current_index = ROUND_ROBIN_INDEX.fetch_add(1, Ordering::SeqCst);
    let index = current_index % route.upstreams.len();

    route.upstreams.get(index)
}

// 实现加权轮询算法
fn select_weighted(route: &RouteConfig) -> Option<&UpstreamServer> {
    // 简化实现：根据权重随机选择
    // 你可以实现更复杂的算法
    let total_weight: u32 = route.upstreams.iter().map(|s| s.weight).sum();

    if total_weight == 0 {
        return select_round_robin(route);
    }

    let mut random_weight = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        % total_weight as u128) as u32;

    for server in &route.upstreams {
        if random_weight < server.weight {
            return Some(server);
        }
        random_weight -= server.weight;
    }

    // 兜底返回第一个
    route.upstreams.first()
}

// 实现最少连接算法（简化版）
fn select_least_conn(route: &RouteConfig) -> Option<&UpstreamServer> {
    // 简化实现：随机选择
    // 实际应该统计每个服务器的活跃连接数
    select_round_robin(route)
}

// 实现入口函数
#[launch]
fn rocket() -> _ {
    // 初始化启动时间
    START_TIME.set(Instant::now()).unwrap();
    server::init_start_time();
    // 加载配置
    let config = load_config().expect("Failed to load config");
    // 初始化日志系统
    init_logging(&config.logging);

    // 初始化调试管理器
    let debug_manager = debug::DebugManager::new(config.debug.clone());

    // 初始化服务管理器
    let service_manager = service::ServiceManager::new();

    // 初始化表格管理器
    let table_manager = table::TableManager::new();

    log::info!(
        "API Gateway starting on {}:{}",
        config.server.host,
        config.server.port
    );
    log::info!("Available routes: {}", config.routes.len());
    log::info!(
        "Starting API Gateway on {}:{}",
        config.server.host,
        config.server.port
    );

    // 构建Rocket应用
    rocket::build()
        .configure(rocket::Config {
            address: config.server.host.parse().unwrap(),
            port: config.server.port,
            workers: config.server.workers,
            ..Default::default()
        })
        .manage(config) // 添加状态管理
        .manage(debug_manager) // 添加调试管理器状态
        .manage(service_manager) // 添加服务管理器状态
        .manage(table_manager) // 添加表格管理器状态
        .mount(
            "/",
            routes![
                index,
                health,
                debug_info,
                proxy_request,
                test_error,
                api::hello,
                api::greet,
                api::echo,
                pie::calculate_pie_get,
                pie::calculate_pie_post,
                pie::pie_example,
                pop::calculate_pop_get,
                pop::calculate_pop_post,
                pop::pop_example,
                keep::filter_keep_get,
                keep::filter_keep_post,
                keep::keep_example,
                server::server_info,
                server::server_status,
                server::server_health,
                server::server_stats,
                service::list_services,
                service::get_service,
                service::register_service,
                service::update_service,
                service::delete_service,
                service::check_service_health,
                service::service_example,
                table::list_tables,
                table::get_table,
                table::get_table_data,
                table::get_table_stats,
                table::create_table,
                table::update_table,
                table::delete_table,
                table::add_row,
                table::update_row,
                table::delete_row,
                table::table_example,
                select::select_query,
                select::select_query_get,
                select::select_example
            ],
        )
}
