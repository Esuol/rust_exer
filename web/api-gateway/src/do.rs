use chrono::Utc;
use rocket::{delete, get, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// HTTP 请求任务
    HttpRequest {
        method: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<serde_json::Value>,
    },
    /// 数据转换任务
    DataTransform {
        data: serde_json::Value,
        transform_type: String,
    },
    /// 数据验证任务
    DataValidation {
        data: serde_json::Value,
        rules: serde_json::Value,
    },
    /// 自定义任务
    Custom {
        action: String,
        params: HashMap<String, serde_json::Value>,
    },
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 创建时间
    pub created_at: String,
    /// 开始时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// 完成时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// 执行时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<f64>,
    /// 重试次数
    #[serde(default)]
    pub retry_count: u32,
    /// 最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

/// 创建任务请求
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    /// 任务名称
    pub name: String,
    /// 任务描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 任务类型
    pub task_type: TaskType,
    /// 最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

/// 任务列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskListResponse {
    /// 任务列表
    pub tasks: Vec<Task>,
    /// 总数
    pub total: usize,
    /// 按状态统计
    pub by_status: HashMap<String, usize>,
}

/// 任务执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    /// 任务ID
    pub task_id: String,
    /// 执行状态
    pub status: String,
    /// 结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: f64,
}

/// 任务管理器
#[derive(Debug, Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建任务
    pub fn create_task(&self, request: CreateTaskRequest) -> Result<Task, String> {
        let task_id = format!("task_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        let task = Task {
            id: task_id.clone(),
            name: request.name,
            description: request.description,
            task_type: request.task_type,
            status: TaskStatus::Pending,
            result: None,
            error: None,
            created_at: now,
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            retry_count: 0,
            max_retries: request.max_retries,
        };

        let mut tasks = self.tasks.write().unwrap();
        tasks.insert(task_id, task.clone());

        Ok(task)
    }

    /// 获取任务列表
    pub fn list_tasks(&self, status_filter: Option<TaskStatus>) -> TaskListResponse {
        let tasks = self.tasks.read().unwrap();
        let mut tasks_vec: Vec<Task> = tasks.values().cloned().collect();

        // 应用状态过滤
        if let Some(status) = status_filter {
            tasks_vec.retain(|t| t.status == status);
        }

        // 统计各状态数量
        let mut by_status: HashMap<String, usize> = HashMap::new();
        for task in tasks.values() {
            let status_str = format!("{:?}", task.status);
            *by_status.entry(status_str).or_insert(0) += 1;
        }

        TaskListResponse {
            total: tasks_vec.len(),
            by_status,
            tasks: tasks_vec,
        }
    }

    /// 获取任务
    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().unwrap();
        tasks.get(task_id).cloned()
    }

    /// 删除任务
    pub fn delete_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().unwrap();
        tasks
            .remove(task_id)
            .ok_or_else(|| "Task not found".to_string())?;
        Ok(())
    }

    /// 执行任务
    pub async fn execute_task(&self, task_id: &str) -> Result<TaskExecutionResult, String> {
        let task = {
            let tasks = self.tasks.read().unwrap();
            tasks
                .get(task_id)
                .ok_or_else(|| "Task not found".to_string())?
                .clone()
        };

        // 更新任务状态为运行中
        {
            let mut tasks = self.tasks.write().unwrap();
            if let Some(t) = tasks.get_mut(task_id) {
                t.status = TaskStatus::Running;
                t.started_at = Some(Utc::now().to_rfc3339());
            }
        }

        let start_time = Instant::now();
        let result = match &task.task_type {
            TaskType::HttpRequest {
                method,
                url,
                headers,
                body,
            } => execute_http_task(method, url, headers, body).await,
            TaskType::DataTransform {
                data,
                transform_type,
            } => execute_transform_task(data, transform_type),
            TaskType::DataValidation { data, rules } => execute_validation_task(data, rules),
            TaskType::Custom { action, params } => execute_custom_task(action, params).await,
        };

        let execution_time = start_time.elapsed().as_millis() as f64;

        // 更新任务状态和结果
        {
            let mut tasks = self.tasks.write().unwrap();
            if let Some(t) = tasks.get_mut(task_id) {
                match result {
                    Ok(value) => {
                        t.status = TaskStatus::Completed;
                        t.result = Some(value);
                        t.completed_at = Some(Utc::now().to_rfc3339());
                    }
                    Err(e) => {
                        if t.retry_count < t.max_retries {
                            t.status = TaskStatus::Pending;
                            t.retry_count += 1;
                        } else {
                            t.status = TaskStatus::Failed;
                            t.error = Some(e.clone());
                            t.completed_at = Some(Utc::now().to_rfc3339());
                        }
                    }
                }
                t.execution_time_ms = Some(execution_time);
            }
        }

        let final_task = self.get_task(task_id).unwrap();
        Ok(TaskExecutionResult {
            task_id: task_id.to_string(),
            status: format!("{:?}", final_task.status),
            result: final_task.result,
            error: final_task.error,
            execution_time_ms: execution_time,
        })
    }

    /// 取消任务
    pub fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().unwrap();
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| "Task not found".to_string())?;

        if task.status == TaskStatus::Running {
            return Err("Cannot cancel a running task".to_string());
        }

        if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
            return Err("Cannot cancel a completed or failed task".to_string());
        }

        task.status = TaskStatus::Cancelled;
        task.completed_at = Some(Utc::now().to_rfc3339());

        Ok(())
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

fn default_max_retries() -> u32 {
    3
}

/// 执行 HTTP 任务
async fn execute_http_task(
    method: &str,
    url: &str,
    headers: &Option<HashMap<String, String>>,
    body: &Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    };

    // 添加请求头
    if let Some(headers) = headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }

    // 添加请求体
    if let Some(body) = body {
        request_builder = request_builder.json(body);
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status().as_u16();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let json_response: serde_json::Value = serde_json::json!({
        "status": status,
        "body": response_text,
    });

    Ok(json_response)
}

/// 执行数据转换任务
fn execute_transform_task(
    data: &serde_json::Value,
    transform_type: &str,
) -> Result<serde_json::Value, String> {
    match transform_type {
        "uppercase" => {
            if let serde_json::Value::String(s) = data {
                Ok(serde_json::Value::String(s.to_uppercase()))
            } else {
                Err("Data must be a string for uppercase transformation".to_string())
            }
        }
        "lowercase" => {
            if let serde_json::Value::String(s) = data {
                Ok(serde_json::Value::String(s.to_lowercase()))
            } else {
                Err("Data must be a string for lowercase transformation".to_string())
            }
        }
        "reverse" => {
            if let serde_json::Value::String(s) = data {
                Ok(serde_json::Value::String(s.chars().rev().collect()))
            } else if let serde_json::Value::Array(arr) = data {
                let mut reversed = arr.clone();
                reversed.reverse();
                Ok(serde_json::Value::Array(reversed))
            } else {
                Err("Data must be a string or array for reverse transformation".to_string())
            }
        }
        _ => Err(format!("Unknown transform type: {}", transform_type)),
    }
}

/// 执行数据验证任务
fn execute_validation_task(
    data: &serde_json::Value,
    _rules: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    // 简化的验证逻辑
    if data.is_null() {
        return Err("Data cannot be null".to_string());
    }

    Ok(serde_json::json!({
        "valid": true,
        "message": "Validation passed"
    }))
}

/// 执行自定义任务
async fn execute_custom_task(
    action: &str,
    params: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    match action {
        "echo" => Ok(serde_json::json!({
            "action": "echo",
            "params": params
        })),
        "delay" => {
            let delay_ms = params
                .get("delay_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(1000);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            Ok(serde_json::json!({
                "action": "delay",
                "delay_ms": delay_ms
            }))
        }
        _ => Err(format!("Unknown custom action: {}", action)),
    }
}

/// POST 端点：创建任务
#[post("/api/do/task", data = "<request>")]
pub fn create_task(
    request: Json<CreateTaskRequest>,
    task_manager: &State<TaskManager>,
) -> Result<Json<Task>, Json<String>> {
    task_manager
        .create_task(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取任务列表
#[get("/api/do/task/list?<status>")]
pub fn list_tasks(
    status: Option<&str>,
    task_manager: &State<TaskManager>,
) -> Json<TaskListResponse> {
    let status_filter = status.and_then(|s| match s {
        "pending" => Some(TaskStatus::Pending),
        "running" => Some(TaskStatus::Running),
        "completed" => Some(TaskStatus::Completed),
        "failed" => Some(TaskStatus::Failed),
        "cancelled" => Some(TaskStatus::Cancelled),
        _ => None,
    });

    Json(task_manager.list_tasks(status_filter))
}

/// GET 端点：获取任务
#[get("/api/do/task/<task_id>")]
pub fn get_task(
    task_id: &str,
    task_manager: &State<TaskManager>,
) -> Result<Json<Task>, Json<String>> {
    task_manager
        .get_task(task_id)
        .map(Json)
        .ok_or_else(|| Json("Task not found".to_string()))
}

/// DELETE 端点：删除任务
#[delete("/api/do/task/<task_id>")]
pub fn delete_task(
    task_id: &str,
    task_manager: &State<TaskManager>,
) -> Result<Json<String>, Json<String>> {
    task_manager
        .delete_task(task_id)
        .map(|_| Json("Task deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// POST 端点：执行任务
#[post("/api/do/task/<task_id>/execute")]
pub async fn execute_task(
    task_id: &str,
    task_manager: &State<TaskManager>,
) -> Result<Json<TaskExecutionResult>, Json<String>> {
    task_manager
        .execute_task(task_id)
        .await
        .map(Json)
        .map_err(|e| Json(e))
}

/// POST 端点：取消任务
#[post("/api/do/task/<task_id>/cancel")]
pub fn cancel_task(
    task_id: &str,
    task_manager: &State<TaskManager>,
) -> Result<Json<String>, Json<String>> {
    task_manager
        .cancel_task(task_id)
        .map(|_| Json("Task cancelled successfully".to_string()))
        .map_err(|e| Json(e))
}

/// GET 端点：获取示例任务
#[get("/api/do/example")]
pub fn do_example(task_manager: &State<TaskManager>) -> Json<Task> {
    let request = CreateTaskRequest {
        name: "示例 HTTP 请求任务".to_string(),
        description: Some("执行一个 GET 请求到健康检查端点".to_string()),
        task_type: TaskType::HttpRequest {
            method: "GET".to_string(),
            url: "http://localhost:8000/health".to_string(),
            headers: None,
            body: None,
        },
        max_retries: 3,
    };

    let task = task_manager.create_task(request).unwrap();
    Json(task)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let manager = TaskManager::new();
        let request = CreateTaskRequest {
            name: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            task_type: TaskType::Custom {
                action: "echo".to_string(),
                params: HashMap::new(),
            },
            max_retries: 3,
        };

        let task = manager.create_task(request).unwrap();
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_list_tasks() {
        let manager = TaskManager::new();
        let request = CreateTaskRequest {
            name: "Test Task".to_string(),
            description: None,
            task_type: TaskType::Custom {
                action: "echo".to_string(),
                params: HashMap::new(),
            },
            max_retries: 3,
        };

        manager.create_task(request).unwrap();
        let list = manager.list_tasks(None);
        assert_eq!(list.total, 1);
    }

    #[test]
    fn test_cancel_task() {
        let manager = TaskManager::new();
        let request = CreateTaskRequest {
            name: "Test Task".to_string(),
            description: None,
            task_type: TaskType::Custom {
                action: "echo".to_string(),
                params: HashMap::new(),
            },
            max_retries: 3,
        };

        let task = manager.create_task(request).unwrap();
        manager.cancel_task(&task.id).unwrap();

        let updated_task = manager.get_task(&task.id).unwrap();
        assert_eq!(updated_task.status, TaskStatus::Cancelled);
    }
}
