use chrono::Utc;
use rocket::{delete, get, post, put, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    /// 键
    pub key: String,
    /// 值
    pub value: serde_json::Value,
    /// 数据类型
    pub data_type: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 过期时间（可选，Unix 时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 数据存储请求
#[derive(Debug, Deserialize)]
pub struct StoreDataRequest {
    /// 键
    pub key: String,
    /// 值
    pub value: serde_json::Value,
    /// 过期时间（秒，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 数据更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateDataRequest {
    /// 值
    pub value: serde_json::Value,
    /// 过期时间（秒，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 批量存储请求
#[derive(Debug, Deserialize)]
pub struct BatchStoreRequest {
    /// 数据项列表
    pub items: Vec<StoreDataRequest>,
}

/// 数据列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct DataListResponse {
    /// 数据项列表
    pub items: Vec<DataItem>,
    /// 总数
    pub total: usize,
}

/// 数据统计响应
#[derive(Debug, Serialize, Deserialize)]
pub struct DataStatsResponse {
    /// 总数据项数
    pub total_items: usize,
    /// 按类型统计
    pub by_type: HashMap<String, usize>,
    /// 已过期项数
    pub expired_items: usize,
    /// 有效项数
    pub active_items: usize,
}

/// 数据转换请求
#[derive(Debug, Deserialize)]
pub struct TransformDataRequest {
    /// 源数据
    pub data: serde_json::Value,
    /// 转换类型（uppercase, lowercase, reverse, json_stringify, json_parse）
    pub transform_type: String,
}

/// 数据验证请求
#[derive(Debug, Deserialize)]
pub struct ValidateDataRequest {
    /// 数据
    pub data: serde_json::Value,
    /// 验证规则
    pub rules: ValidationRules,
}

/// 验证规则
#[derive(Debug, Deserialize)]
pub struct ValidationRules {
    /// 是否必填
    #[serde(default)]
    pub required: bool,
    /// 最小长度（字符串或数组）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// 最大长度（字符串或数组）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// 最小值（数字）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    /// 最大值（数字）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    /// 正则表达式（字符串）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// 验证结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub valid: bool,
    /// 错误信息列表
    pub errors: Vec<String>,
}

/// 数据管理器
#[derive(Debug, Clone)]
pub struct DataManager {
    data: Arc<RwLock<HashMap<String, DataItem>>>,
}

impl DataManager {
    /// 创建新的数据管理器
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 存储数据
    pub fn store_data(&self, request: StoreDataRequest) -> Result<DataItem, String> {
        let now = Utc::now();
        let expires_at = request
            .ttl
            .map(|ttl| (now + chrono::Duration::seconds(ttl as i64)).timestamp());

        let data_type = match request.value {
            serde_json::Value::String(_) => "string",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
            serde_json::Value::Null => "null",
        }
        .to_string();

        let data_item = DataItem {
            key: request.key.clone(),
            value: request.value,
            data_type,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expires_at,
            metadata: request.metadata,
        };

        let mut data = self.data.write().unwrap();
        data.insert(request.key, data_item.clone());

        Ok(data_item)
    }

    /// 获取数据
    pub fn get_data(&self, key: &str) -> Option<DataItem> {
        let data = self.data.read().unwrap();
        let item = data.get(key)?.clone();
        drop(data);

        // 检查是否过期
        if let Some(expires_at) = item.expires_at {
            if Utc::now().timestamp() > expires_at {
                // 数据已过期，删除它
                let mut data = self.data.write().unwrap();
                data.remove(key);
                return None;
            }
        }

        Some(item)
    }

    /// 更新数据
    pub fn update_data(&self, key: &str, request: UpdateDataRequest) -> Result<DataItem, String> {
        let mut data = self.data.write().unwrap();
        let item = data
            .get_mut(key)
            .ok_or_else(|| "Data not found".to_string())?;

        let now = Utc::now();
        let expires_at = request
            .ttl
            .map(|ttl| (now + chrono::Duration::seconds(ttl as i64)).timestamp());

        item.value = request.value;
        item.updated_at = now.to_rfc3339();
        if let Some(expires_at) = expires_at {
            item.expires_at = Some(expires_at);
        }
        if let Some(metadata) = request.metadata {
            item.metadata = Some(metadata);
        }

        Ok(item.clone())
    }

    /// 删除数据
    pub fn delete_data(&self, key: &str) -> Result<(), String> {
        let mut data = self.data.write().unwrap();
        data.remove(key)
            .ok_or_else(|| "Data not found".to_string())?;
        Ok(())
    }

    /// 批量存储数据
    pub fn batch_store(&self, request: BatchStoreRequest) -> Result<Vec<DataItem>, String> {
        let mut results = Vec::new();
        for item_request in request.items {
            match self.store_data(item_request) {
                Ok(item) => results.push(item),
                Err(e) => return Err(format!("Failed to store item: {}", e)),
            }
        }
        Ok(results)
    }

    /// 获取数据列表
    pub fn list_data(&self, pattern: Option<&str>) -> DataListResponse {
        let data = self.data.read().unwrap();
        let now = Utc::now().timestamp();

        let items: Vec<DataItem> = data
            .values()
            .filter(|item| {
                // 过滤过期项
                if let Some(expires_at) = item.expires_at {
                    if now > expires_at {
                        return false;
                    }
                }

                // 应用模式匹配
                if let Some(pattern) = pattern {
                    item.key.contains(pattern)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        DataListResponse {
            total: items.len(),
            items,
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> DataStatsResponse {
        let data = self.data.read().unwrap();
        let now = Utc::now().timestamp();

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut expired_items = 0;
        let mut active_items = 0;

        for item in data.values() {
            let is_expired = if let Some(expires_at) = item.expires_at {
                now > expires_at
            } else {
                false
            };

            if is_expired {
                expired_items += 1;
            } else {
                active_items += 1;
                *by_type.entry(item.data_type.clone()).or_insert(0) += 1;
            }
        }

        DataStatsResponse {
            total_items: data.len(),
            by_type,
            expired_items,
            active_items,
        }
    }

    /// 清理过期数据
    pub fn cleanup_expired(&self) -> usize {
        let mut data = self.data.write().unwrap();
        let now = Utc::now().timestamp();
        let mut removed = 0;

        data.retain(|_, item| {
            if let Some(expires_at) = item.expires_at {
                if now > expires_at {
                    removed += 1;
                    return false;
                }
            }
            true
        });

        removed
    }
}

impl Default for DataManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 转换数据
pub fn transform_data(request: TransformDataRequest) -> Result<serde_json::Value, String> {
    match request.transform_type.as_str() {
        "uppercase" => {
            if let serde_json::Value::String(s) = request.data {
                Ok(serde_json::Value::String(s.to_uppercase()))
            } else {
                Err("Data must be a string for uppercase transformation".to_string())
            }
        }
        "lowercase" => {
            if let serde_json::Value::String(s) = request.data {
                Ok(serde_json::Value::String(s.to_lowercase()))
            } else {
                Err("Data must be a string for lowercase transformation".to_string())
            }
        }
        "reverse" => {
            if let serde_json::Value::String(s) = request.data {
                Ok(serde_json::Value::String(s.chars().rev().collect()))
            } else if let serde_json::Value::Array(arr) = request.data {
                let mut reversed = arr.clone();
                reversed.reverse();
                Ok(serde_json::Value::Array(reversed))
            } else {
                Err("Data must be a string or array for reverse transformation".to_string())
            }
        }
        "json_stringify" => Ok(serde_json::Value::String(
            serde_json::to_string(&request.data)
                .map_err(|e| format!("Failed to stringify JSON: {}", e))?,
        )),
        "json_parse" => {
            if let serde_json::Value::String(s) = request.data {
                serde_json::from_str::<serde_json::Value>(&s)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))
            } else {
                Err("Data must be a string for JSON parse transformation".to_string())
            }
        }
        _ => Err(format!(
            "Unknown transform type: {}",
            request.transform_type
        )),
    }
}

/// 验证数据
pub fn validate_data(request: ValidateDataRequest) -> ValidationResult {
    let mut errors = Vec::new();

    // 检查必填
    if request.rules.required && request.data.is_null() {
        errors.push("Field is required".to_string());
        return ValidationResult {
            valid: false,
            errors,
        };
    }

    if request.data.is_null() {
        return ValidationResult {
            valid: true,
            errors,
        };
    }

    // 检查长度（字符串）
    if let serde_json::Value::String(s) = &request.data {
        if let Some(min) = request.rules.min_length {
            if s.len() < min {
                errors.push(format!("String length must be at least {}", min));
            }
        }
        if let Some(max) = request.rules.max_length {
            if s.len() > max {
                errors.push(format!("String length must be at most {}", max));
            }
        }
        if let Some(pattern) = &request.rules.pattern {
            // 简单的字符串包含匹配（如果需要完整正则表达式，需要添加 regex crate）
            if !s.contains(pattern) {
                errors.push(format!("String does not contain pattern: {}", pattern));
            }
        }
    }

    // 检查长度（数组）
    if let serde_json::Value::Array(arr) = &request.data {
        if let Some(min) = request.rules.min_length {
            if arr.len() < min {
                errors.push(format!("Array length must be at least {}", min));
            }
        }
        if let Some(max) = request.rules.max_length {
            if arr.len() > max {
                errors.push(format!("Array length must be at most {}", max));
            }
        }
    }

    // 检查数值范围
    if let Some(num) = request.data.as_f64() {
        if let Some(min) = request.rules.min_value {
            if num < min {
                errors.push(format!("Value must be at least {}", min));
            }
        }
        if let Some(max) = request.rules.max_value {
            if num > max {
                errors.push(format!("Value must be at most {}", max));
            }
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

/// POST 端点：存储数据
#[post("/api/data/store", data = "<request>")]
pub fn store_data(
    request: Json<StoreDataRequest>,
    data_manager: &State<DataManager>,
) -> Result<Json<DataItem>, Json<String>> {
    data_manager
        .store_data(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取数据
#[get("/api/data/<key>")]
pub fn get_data(
    key: &str,
    data_manager: &State<DataManager>,
) -> Result<Json<DataItem>, Json<String>> {
    data_manager
        .get_data(key)
        .map(Json)
        .ok_or_else(|| Json("Data not found".to_string()))
}

/// PUT 端点：更新数据
#[put("/api/data/<key>", data = "<request>")]
pub fn update_data(
    key: &str,
    request: Json<UpdateDataRequest>,
    data_manager: &State<DataManager>,
) -> Result<Json<DataItem>, Json<String>> {
    data_manager
        .update_data(key, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// DELETE 端点：删除数据
#[delete("/api/data/<key>")]
pub fn delete_data(
    key: &str,
    data_manager: &State<DataManager>,
) -> Result<Json<String>, Json<String>> {
    data_manager
        .delete_data(key)
        .map(|_| Json("Data deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// POST 端点：批量存储数据
#[post("/api/data/batch", data = "<request>")]
pub fn batch_store(
    request: Json<BatchStoreRequest>,
    data_manager: &State<DataManager>,
) -> Result<Json<Vec<DataItem>>, Json<String>> {
    data_manager
        .batch_store(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取数据列表
#[get("/api/data/list?<pattern>")]
pub fn list_data(
    pattern: Option<&str>,
    data_manager: &State<DataManager>,
) -> Json<DataListResponse> {
    Json(data_manager.list_data(pattern))
}

/// GET 端点：获取统计信息
#[get("/api/data/stats")]
pub fn get_stats(data_manager: &State<DataManager>) -> Json<DataStatsResponse> {
    Json(data_manager.get_stats())
}

/// POST 端点：清理过期数据
#[post("/api/data/cleanup")]
pub fn cleanup_expired(data_manager: &State<DataManager>) -> Json<HashMap<String, usize>> {
    let removed = data_manager.cleanup_expired();
    let mut result = HashMap::new();
    result.insert("removed".to_string(), removed);
    Json(result)
}

/// POST 端点：转换数据
#[post("/api/data/transform", data = "<request>")]
pub fn transform_data_endpoint(
    request: Json<TransformDataRequest>,
) -> Result<Json<serde_json::Value>, Json<String>> {
    transform_data(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// POST 端点：验证数据
#[post("/api/data/validate", data = "<request>")]
pub fn validate_data_endpoint(request: Json<ValidateDataRequest>) -> Json<ValidationResult> {
    Json(validate_data(request.into_inner()))
}

/// GET 端点：获取示例数据
#[get("/api/data/example")]
pub fn data_example(data_manager: &State<DataManager>) -> Json<DataItem> {
    let request = StoreDataRequest {
        key: "example_key".to_string(),
        value: serde_json::json!({
            "name": "示例数据",
            "value": 123,
            "active": true
        }),
        ttl: Some(3600),
        metadata: Some({
            let mut m = HashMap::new();
            m.insert("source".to_string(), "example".to_string());
            m
        }),
    };

    let data_item = data_manager.store_data(request).unwrap();
    Json(data_item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_get_data() {
        let manager = DataManager::new();
        let request = StoreDataRequest {
            key: "test_key".to_string(),
            value: serde_json::Value::String("test_value".to_string()),
            ttl: None,
            metadata: None,
        };

        let stored = manager.store_data(request).unwrap();
        assert_eq!(stored.key, "test_key");

        let retrieved = manager.get_data("test_key").unwrap();
        assert_eq!(
            retrieved.value,
            serde_json::Value::String("test_value".to_string())
        );
    }

    #[test]
    fn test_update_data() {
        let manager = DataManager::new();
        let request = StoreDataRequest {
            key: "test_key".to_string(),
            value: serde_json::Value::String("old_value".to_string()),
            ttl: None,
            metadata: None,
        };

        manager.store_data(request).unwrap();

        let update_request = UpdateDataRequest {
            value: serde_json::Value::String("new_value".to_string()),
            ttl: None,
            metadata: None,
        };

        let updated = manager.update_data("test_key", update_request).unwrap();
        assert_eq!(
            updated.value,
            serde_json::Value::String("new_value".to_string())
        );
    }

    #[test]
    fn test_delete_data() {
        let manager = DataManager::new();
        let request = StoreDataRequest {
            key: "test_key".to_string(),
            value: serde_json::Value::String("test_value".to_string()),
            ttl: None,
            metadata: None,
        };

        manager.store_data(request).unwrap();
        manager.delete_data("test_key").unwrap();

        assert!(manager.get_data("test_key").is_none());
    }

    #[test]
    fn test_transform_data_uppercase() {
        let request = TransformDataRequest {
            data: serde_json::Value::String("hello".to_string()),
            transform_type: "uppercase".to_string(),
        };

        let result = transform_data(request).unwrap();
        assert_eq!(result, serde_json::Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_validate_data() {
        let request = ValidateDataRequest {
            data: serde_json::Value::String("test".to_string()),
            rules: ValidationRules {
                required: true,
                min_length: Some(3),
                max_length: Some(10),
                min_value: None,
                max_value: None,
                pattern: None,
            },
        };

        let result = validate_data(request);
        assert!(result.valid);
    }
}
