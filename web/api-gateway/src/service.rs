use chrono::Utc;
use rocket::{delete, get, post, put, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// 服务ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务URL
    pub url: String,
    /// 服务权重（用于负载均衡）
    pub weight: u32,
    /// 服务状态（active, inactive, unhealthy）
    pub status: String,
    /// 健康状态（healthy, unhealthy, unknown）
    pub health: String,
    /// 最后检查时间
    pub last_check: Option<String>,
    /// 注册时间
    pub registered_at: String,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 服务注册请求
#[derive(Debug, Deserialize)]
pub struct ServiceRegisterRequest {
    /// 服务名称
    pub name: String,
    /// 服务URL
    pub url: String,
    /// 服务权重
    #[serde(default = "default_weight")]
    pub weight: u32,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 服务更新请求
#[derive(Debug, Deserialize)]
pub struct ServiceUpdateRequest {
    /// 服务名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 服务URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 服务权重（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,
    /// 服务状态（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 服务列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceListResponse {
    /// 服务列表
    pub services: Vec<ServiceInfo>,
    /// 总数
    pub total: usize,
    /// 活跃服务数
    pub active_count: usize,
    /// 健康服务数
    pub healthy_count: usize,
}

/// 服务详情响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceDetailResponse {
    /// 服务信息
    pub service: ServiceInfo,
    /// 统计信息
    pub stats: ServiceStats,
}

/// 服务统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 最后请求时间
    pub last_request_time: Option<String>,
}

/// 服务健康检查响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealthResponse {
    /// 服务ID
    pub service_id: String,
    /// 健康状态
    pub health: String,
    /// 响应时间（毫秒）
    pub response_time_ms: Option<f64>,
    /// 检查时间
    pub checked_at: String,
    /// 消息
    pub message: String,
}

/// 服务管理器
#[derive(Debug, Clone)]
pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    stats: Arc<RwLock<HashMap<String, ServiceStats>>>,
}

#[allow(dead_code)]
impl ServiceManager {
    /// 创建新的服务管理器
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册服务
    pub fn register_service(&self, request: ServiceRegisterRequest) -> Result<ServiceInfo, String> {
        let service_id = format!("service_{}", Utc::now().timestamp_millis());

        let service = ServiceInfo {
            id: service_id.clone(),
            name: request.name,
            url: request.url,
            weight: request.weight,
            status: "active".to_string(),
            health: "unknown".to_string(),
            last_check: None,
            registered_at: Utc::now().to_rfc3339(),
            metadata: request.metadata,
        };

        let mut services = self.services.write().unwrap();
        services.insert(service_id.clone(), service.clone());

        // 初始化统计信息
        let mut stats = self.stats.write().unwrap();
        stats.insert(
            service_id,
            ServiceStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time_ms: 0.0,
                last_request_time: None,
            },
        );

        Ok(service)
    }

    /// 更新服务
    pub fn update_service(
        &self,
        service_id: &str,
        request: ServiceUpdateRequest,
    ) -> Result<ServiceInfo, String> {
        let mut services = self.services.write().unwrap();

        let service = services
            .get_mut(service_id)
            .ok_or_else(|| "Service not found".to_string())?;

        if let Some(name) = request.name {
            service.name = name;
        }
        if let Some(url) = request.url {
            service.url = url;
        }
        if let Some(weight) = request.weight {
            service.weight = weight;
        }
        if let Some(status) = request.status {
            service.status = status;
        }
        if let Some(metadata) = request.metadata {
            service.metadata = Some(metadata);
        }

        Ok(service.clone())
    }

    /// 删除服务
    pub fn delete_service(&self, service_id: &str) -> Result<(), String> {
        let mut services = self.services.write().unwrap();
        services
            .remove(service_id)
            .ok_or_else(|| "Service not found".to_string())?;

        let mut stats = self.stats.write().unwrap();
        stats.remove(service_id);

        Ok(())
    }

    /// 获取服务列表
    pub fn list_services(&self) -> ServiceListResponse {
        let services = self.services.read().unwrap();
        let services_vec: Vec<ServiceInfo> = services.values().cloned().collect();

        let active_count = services_vec.iter().filter(|s| s.status == "active").count();
        let healthy_count = services_vec
            .iter()
            .filter(|s| s.health == "healthy")
            .count();

        ServiceListResponse {
            total: services_vec.len(),
            active_count,
            healthy_count,
            services: services_vec,
        }
    }

    /// 获取服务详情
    pub fn get_service(&self, service_id: &str) -> Option<ServiceDetailResponse> {
        let services = self.services.read().unwrap();
        let service = services.get(service_id)?.clone();
        drop(services);

        let stats = self.stats.read().unwrap();
        let stats = stats.get(service_id)?.clone();

        Some(ServiceDetailResponse { service, stats })
    }

    /// 检查服务健康状态
    pub async fn check_service_health(&self, service_id: &str) -> ServiceHealthResponse {
        let start_time = Instant::now();
        // 用作用域确保 RwLockReadGuard 不会跨越 await，避免 future 非 Send
        let service = {
            let services = self.services.read().unwrap();
            match services.get(service_id) {
                Some(s) => s.clone(),
                None => {
                    return ServiceHealthResponse {
                        service_id: service_id.to_string(),
                        health: "unknown".to_string(),
                        response_time_ms: None,
                        checked_at: Utc::now().to_rfc3339(),
                        message: "Service not found".to_string(),
                    };
                }
            }
        };

        // 执行健康检查
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let health_check_url = format!("{}/health", service.url.trim_end_matches('/'));
        let result = client.get(&health_check_url).send().await;

        let response_time = start_time.elapsed().as_millis() as f64;
        let (health, message) = match result {
            Ok(response) => {
                if response.status().is_success() {
                    ("healthy".to_string(), "Service is healthy".to_string())
                } else {
                    (
                        "unhealthy".to_string(),
                        format!("Service returned status: {}", response.status()),
                    )
                }
            }
            Err(e) => (
                "unhealthy".to_string(),
                format!("Health check failed: {}", e),
            ),
        };

        // 更新服务健康状态
        let mut services = self.services.write().unwrap();
        if let Some(s) = services.get_mut(service_id) {
            s.health = health.clone();
            s.last_check = Some(Utc::now().to_rfc3339());
        }
        drop(services);

        // 记录一次健康检查的“请求”统计（便于观测）
        self.record_request(service_id, health == "healthy", response_time);

        ServiceHealthResponse {
            service_id: service_id.to_string(),
            health,
            response_time_ms: Some(response_time),
            checked_at: Utc::now().to_rfc3339(),
            message,
        }
    }

    /// 记录请求统计
    pub fn record_request(&self, service_id: &str, success: bool, response_time_ms: f64) {
        let mut stats = self.stats.write().unwrap();
        if let Some(stat) = stats.get_mut(service_id) {
            stat.total_requests += 1;
            if success {
                stat.successful_requests += 1;
            } else {
                stat.failed_requests += 1;
            }

            // 更新平均响应时间（简化计算）
            let total = stat.total_requests as f64;
            stat.avg_response_time_ms =
                (stat.avg_response_time_ms * (total - 1.0) + response_time_ms) / total;

            stat.last_request_time = Some(Utc::now().to_rfc3339());
        }
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

fn default_weight() -> u32 {
    1
}

/// GET 端点：获取服务列表
#[get("/api/service/list")]
pub fn list_services(service_manager: &State<ServiceManager>) -> Json<ServiceListResponse> {
    Json(service_manager.list_services())
}

/// GET 端点：获取服务详情
#[get("/api/service/<service_id>")]
pub fn get_service(
    service_id: &str,
    service_manager: &State<ServiceManager>,
) -> Result<Json<ServiceDetailResponse>, Json<String>> {
    service_manager
        .get_service(service_id)
        .map(Json)
        .ok_or_else(|| Json("Service not found".to_string()))
}

/// POST 端点：注册服务
#[post("/api/service/register", data = "<request>")]
pub fn register_service(
    request: Json<ServiceRegisterRequest>,
    service_manager: &State<ServiceManager>,
) -> Result<Json<ServiceInfo>, Json<String>> {
    service_manager
        .register_service(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// PUT 端点：更新服务
#[put("/api/service/<service_id>", data = "<request>")]
pub fn update_service(
    service_id: &str,
    request: Json<ServiceUpdateRequest>,
    service_manager: &State<ServiceManager>,
) -> Result<Json<ServiceInfo>, Json<String>> {
    service_manager
        .update_service(service_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// DELETE 端点：删除服务
#[delete("/api/service/<service_id>")]
pub fn delete_service(
    service_id: &str,
    service_manager: &State<ServiceManager>,
) -> Result<Json<String>, Json<String>> {
    service_manager
        .delete_service(service_id)
        .map(|_| Json("Service deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// GET 端点：检查服务健康状态
#[get("/api/service/<service_id>/health")]
pub async fn check_service_health(
    service_id: &str,
    service_manager: &State<ServiceManager>,
) -> Json<ServiceHealthResponse> {
    Json(service_manager.check_service_health(service_id).await)
}

/// GET 端点：获取示例服务列表
#[get("/api/service/example")]
pub fn service_example(service_manager: &State<ServiceManager>) -> Json<ServiceListResponse> {
    // 如果服务列表为空，创建一些示例服务
    let services = service_manager.list_services();
    if services.total == 0 {
        // 注册示例服务
        let _ = service_manager.register_service(ServiceRegisterRequest {
            name: "User Service".to_string(),
            url: "http://localhost:8001".to_string(),
            weight: 10,
            metadata: Some({
                let mut m = HashMap::new();
                m.insert("version".to_string(), "1.0.0".to_string());
                m.insert("environment".to_string(), "production".to_string());
                m
            }),
        });

        let _ = service_manager.register_service(ServiceRegisterRequest {
            name: "Order Service".to_string(),
            url: "http://localhost:8002".to_string(),
            weight: 5,
            metadata: None,
        });

        let _ = service_manager.register_service(ServiceRegisterRequest {
            name: "Payment Service".to_string(),
            url: "http://localhost:8003".to_string(),
            weight: 8,
            metadata: None,
        });
    }

    Json(service_manager.list_services())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_manager_register() {
        let manager = ServiceManager::new();
        let request = ServiceRegisterRequest {
            name: "Test Service".to_string(),
            url: "http://localhost:8080".to_string(),
            weight: 10,
            metadata: None,
        };

        let service = manager.register_service(request).unwrap();
        assert_eq!(service.name, "Test Service");
        assert_eq!(service.url, "http://localhost:8080");
        assert_eq!(service.weight, 10);
        assert_eq!(service.status, "active");
    }

    #[test]
    fn test_service_manager_list() {
        let manager = ServiceManager::new();
        let request = ServiceRegisterRequest {
            name: "Test Service".to_string(),
            url: "http://localhost:8080".to_string(),
            weight: 10,
            metadata: None,
        };

        manager.register_service(request).unwrap();
        let list = manager.list_services();
        assert_eq!(list.total, 1);
        assert_eq!(list.active_count, 1);
    }

    #[test]
    fn test_service_manager_update() {
        let manager = ServiceManager::new();
        let request = ServiceRegisterRequest {
            name: "Test Service".to_string(),
            url: "http://localhost:8080".to_string(),
            weight: 10,
            metadata: None,
        };

        let service = manager.register_service(request).unwrap();
        let update_request = ServiceUpdateRequest {
            name: Some("Updated Service".to_string()),
            url: None,
            weight: Some(20),
            status: None,
            metadata: None,
        };

        let updated = manager.update_service(&service.id, update_request).unwrap();
        assert_eq!(updated.name, "Updated Service");
        assert_eq!(updated.weight, 20);
    }

    #[test]
    fn test_service_manager_delete() {
        let manager = ServiceManager::new();
        let request = ServiceRegisterRequest {
            name: "Test Service".to_string(),
            url: "http://localhost:8080".to_string(),
            weight: 10,
            metadata: None,
        };

        let service = manager.register_service(request).unwrap();
        manager.delete_service(&service.id).unwrap();

        let list = manager.list_services();
        assert_eq!(list.total, 0);
    }

    #[test]
    fn test_record_request() {
        let manager = ServiceManager::new();
        let request = ServiceRegisterRequest {
            name: "Test Service".to_string(),
            url: "http://localhost:8080".to_string(),
            weight: 10,
            metadata: None,
        };

        let service = manager.register_service(request).unwrap();
        manager.record_request(&service.id, true, 25.5);
        manager.record_request(&service.id, false, 50.0);

        let detail = manager.get_service(&service.id).unwrap();
        assert_eq!(detail.stats.total_requests, 2);
        assert_eq!(detail.stats.successful_requests, 1);
        assert_eq!(detail.stats.failed_requests, 1);
    }
}
