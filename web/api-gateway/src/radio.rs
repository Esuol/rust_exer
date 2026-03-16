use chrono::Utc;
use rocket::{delete, get, post, put, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 选项项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioOption {
    /// 选项ID
    pub id: String,
    /// 选项标签
    pub label: String,
    /// 选项值
    pub value: serde_json::Value,
    /// 是否禁用
    #[serde(default)]
    pub disabled: bool,
    /// 是否默认选中
    #[serde(default)]
    pub default: bool,
    /// 描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 选项组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioGroup {
    /// 组ID
    pub id: String,
    /// 组名称
    pub name: String,
    /// 组描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 选项列表
    pub options: Vec<RadioOption>,
    /// 当前选中的选项ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_option_id: Option<String>,
    /// 是否允许多选（false 表示单选）
    #[serde(default)]
    pub multiple: bool,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 创建选项组请求
#[derive(Debug, Deserialize)]
pub struct CreateRadioGroupRequest {
    /// 组名称
    pub name: String,
    /// 组描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 选项列表
    pub options: Vec<CreateRadioOptionRequest>,
    /// 是否允许多选
    #[serde(default)]
    pub multiple: bool,
}

/// 创建选项请求
#[derive(Debug, Deserialize)]
pub struct CreateRadioOptionRequest {
    /// 选项标签
    pub label: String,
    /// 选项值
    pub value: serde_json::Value,
    /// 是否禁用
    #[serde(default)]
    pub disabled: bool,
    /// 是否默认选中
    #[serde(default)]
    pub default: bool,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 更新选项组请求
#[derive(Debug, Deserialize)]
pub struct UpdateRadioGroupRequest {
    /// 组名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 组描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 是否允许多选（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple: Option<bool>,
}

/// 选择选项请求
#[derive(Debug, Deserialize)]
pub struct SelectOptionRequest {
    /// 选项ID（单选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_id: Option<String>,
    /// 选项ID列表（多选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_ids: Option<Vec<String>>,
}

/// 选项组列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RadioGroupListResponse {
    /// 选项组列表
    pub groups: Vec<RadioGroup>,
    /// 总数
    pub total: usize,
}

/// 选项组详情响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RadioGroupDetailResponse {
    /// 选项组信息
    pub group: RadioGroup,
    /// 统计信息
    pub stats: RadioGroupStats,
}

/// 选项组统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct RadioGroupStats {
    /// 总选项数
    pub total_options: usize,
    /// 可用选项数
    pub available_options: usize,
    /// 禁用选项数
    pub disabled_options: usize,
    /// 已选中选项数
    pub selected_count: usize,
}

/// 选项管理器
#[derive(Debug, Clone)]
pub struct RadioManager {
    groups: Arc<RwLock<HashMap<String, RadioGroup>>>,
    selections: Arc<RwLock<HashMap<String, Vec<String>>>>, // group_id -> selected_option_ids
}

impl RadioManager {
    /// 创建新的选项管理器
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            selections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建选项组
    pub fn create_group(&self, request: CreateRadioGroupRequest) -> Result<RadioGroup, String> {
        let group_id = format!("radio_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        let mut options = Vec::new();
        let mut selected_option_id = None;

        for opt_request in request.options {
            let option_id = format!("opt_{}", Utc::now().timestamp_millis());
            let option = RadioOption {
                id: option_id.clone(),
                label: opt_request.label,
                value: opt_request.value,
                disabled: opt_request.disabled,
                default: opt_request.default,
                description: opt_request.description,
                metadata: opt_request.metadata,
            };

            if option.default && selected_option_id.is_none() {
                selected_option_id = Some(option_id.clone());
            }

            options.push(option);
        }

        let selected_id_for_selection = selected_option_id.clone();

        let group = RadioGroup {
            id: group_id.clone(),
            name: request.name,
            description: request.description,
            options,
            selected_option_id,
            multiple: request.multiple,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut groups = self.groups.write().unwrap();
        groups.insert(group_id.clone(), group.clone());

        // 初始化选择记录
        if let Some(selected_id) = selected_id_for_selection {
            let mut selections = self.selections.write().unwrap();
            selections.insert(group_id, vec![selected_id]);
        }

        Ok(group)
    }

    /// 更新选项组
    pub fn update_group(
        &self,
        group_id: &str,
        request: UpdateRadioGroupRequest,
    ) -> Result<RadioGroup, String> {
        let mut groups = self.groups.write().unwrap();

        let group = groups
            .get_mut(group_id)
            .ok_or_else(|| "Radio group not found".to_string())?;

        if let Some(name) = request.name {
            group.name = name;
        }
        if let Some(description) = request.description {
            group.description = Some(description);
        }
        if let Some(multiple) = request.multiple {
            group.multiple = multiple;
        }
        group.updated_at = Utc::now().to_rfc3339();

        Ok(group.clone())
    }

    /// 删除选项组
    pub fn delete_group(&self, group_id: &str) -> Result<(), String> {
        let mut groups = self.groups.write().unwrap();
        groups
            .remove(group_id)
            .ok_or_else(|| "Radio group not found".to_string())?;

        let mut selections = self.selections.write().unwrap();
        selections.remove(group_id);

        Ok(())
    }

    /// 获取选项组列表
    pub fn list_groups(&self) -> RadioGroupListResponse {
        let groups = self.groups.read().unwrap();
        let groups_vec: Vec<RadioGroup> = groups.values().cloned().collect();

        RadioGroupListResponse {
            total: groups_vec.len(),
            groups: groups_vec,
        }
    }

    /// 获取选项组详情
    pub fn get_group(&self, group_id: &str) -> Option<RadioGroupDetailResponse> {
        let groups = self.groups.read().unwrap();
        let group = groups.get(group_id)?.clone();
        drop(groups);

        let selections = self.selections.read().unwrap();
        let selected_ids = selections.get(group_id).cloned().unwrap_or_default();
        drop(selections);

        // 更新选中状态
        let mut group = group;
        if !selected_ids.is_empty() {
            if group.multiple {
                // 多选模式
                for option in &mut group.options {
                    if selected_ids.contains(&option.id) {
                        // 标记为选中（通过更新 selected_option_id 或使用其他方式）
                    }
                }
            } else {
                // 单选模式
                group.selected_option_id = selected_ids.first().cloned();
            }
        }

        let stats = RadioGroupStats {
            total_options: group.options.len(),
            available_options: group.options.iter().filter(|o| !o.disabled).count(),
            disabled_options: group.options.iter().filter(|o| o.disabled).count(),
            selected_count: selected_ids.len(),
        };

        Some(RadioGroupDetailResponse { group, stats })
    }

    /// 添加选项
    pub fn add_option(
        &self,
        group_id: &str,
        request: CreateRadioOptionRequest,
    ) -> Result<RadioOption, String> {
        let mut groups = self.groups.write().unwrap();
        let group = groups
            .get_mut(group_id)
            .ok_or_else(|| "Radio group not found".to_string())?;

        let option_id = format!("opt_{}", Utc::now().timestamp_millis());
        let option = RadioOption {
            id: option_id.clone(),
            label: request.label,
            value: request.value,
            disabled: request.disabled,
            default: request.default,
            description: request.description,
            metadata: request.metadata,
        };

        if option.default && group.selected_option_id.is_none() {
            group.selected_option_id = Some(option_id.clone());
        }

        group.options.push(option.clone());
        group.updated_at = Utc::now().to_rfc3339();

        Ok(option)
    }

    /// 选择选项
    pub fn select_option(
        &self,
        group_id: &str,
        request: SelectOptionRequest,
    ) -> Result<RadioGroup, String> {
        let mut groups = self.groups.write().unwrap();
        let group = groups
            .get_mut(group_id)
            .ok_or_else(|| "Radio group not found".to_string())?;

        let mut selections = self.selections.write().unwrap();

        if group.multiple {
            // 多选模式
            if let Some(option_ids) = request.option_ids {
                // 验证选项是否存在且未禁用
                for option_id in &option_ids {
                    let option = group
                        .options
                        .iter()
                        .find(|o| o.id == *option_id && !o.disabled);
                    if option.is_none() {
                        return Err(format!("Option {} not found or disabled", option_id));
                    }
                }
                selections.insert(group_id.to_string(), option_ids);
            } else {
                return Err("Multiple selection requires option_ids".to_string());
            }
        } else {
            // 单选模式
            if let Some(option_id) = request.option_id {
                // 验证选项是否存在且未禁用
                let option = group
                    .options
                    .iter()
                    .find(|o| o.id == option_id && !o.disabled);
                if option.is_none() {
                    return Err(format!("Option {} not found or disabled", option_id));
                }
                group.selected_option_id = Some(option_id.clone());
                selections.insert(group_id.to_string(), vec![option_id]);
            } else {
                return Err("Single selection requires option_id".to_string());
            }
        }

        group.updated_at = Utc::now().to_rfc3339();

        Ok(group.clone())
    }

    /// 获取选中的选项
    pub fn get_selected_options(&self, group_id: &str) -> Option<Vec<RadioOption>> {
        let groups = self.groups.read().unwrap();
        let group = groups.get(group_id)?;
        let selected_ids = if let Some(selected_id) = &group.selected_option_id {
            vec![selected_id.clone()]
        } else {
            return Some(Vec::new());
        };
        drop(groups);

        let selections = self.selections.read().unwrap();
        let selected_ids = selections.get(group_id).cloned().unwrap_or(selected_ids);
        drop(selections);

        let groups = self.groups.read().unwrap();
        let group = groups.get(group_id)?;
        let selected_options: Vec<RadioOption> = group
            .options
            .iter()
            .filter(|opt| selected_ids.contains(&opt.id))
            .cloned()
            .collect();

        Some(selected_options)
    }
}

impl Default for RadioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// POST 端点：创建选项组
#[post("/api/radio/group", data = "<request>")]
pub fn create_group(
    request: Json<CreateRadioGroupRequest>,
    radio_manager: &State<RadioManager>,
) -> Result<Json<RadioGroup>, Json<String>> {
    radio_manager
        .create_group(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取选项组列表
#[get("/api/radio/group/list")]
pub fn list_groups(radio_manager: &State<RadioManager>) -> Json<RadioGroupListResponse> {
    Json(radio_manager.list_groups())
}

/// GET 端点：获取选项组详情
#[get("/api/radio/group/<group_id>")]
pub fn get_group(
    group_id: &str,
    radio_manager: &State<RadioManager>,
) -> Result<Json<RadioGroupDetailResponse>, Json<String>> {
    radio_manager
        .get_group(group_id)
        .map(Json)
        .ok_or_else(|| Json("Radio group not found".to_string()))
}

/// PUT 端点：更新选项组
#[put("/api/radio/group/<group_id>", data = "<request>")]
pub fn update_group(
    group_id: &str,
    request: Json<UpdateRadioGroupRequest>,
    radio_manager: &State<RadioManager>,
) -> Result<Json<RadioGroup>, Json<String>> {
    radio_manager
        .update_group(group_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// DELETE 端点：删除选项组
#[delete("/api/radio/group/<group_id>")]
pub fn delete_group(
    group_id: &str,
    radio_manager: &State<RadioManager>,
) -> Result<Json<String>, Json<String>> {
    radio_manager
        .delete_group(group_id)
        .map(|_| Json("Radio group deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// POST 端点：添加选项
#[post("/api/radio/group/<group_id>/option", data = "<request>")]
pub fn add_option(
    group_id: &str,
    request: Json<CreateRadioOptionRequest>,
    radio_manager: &State<RadioManager>,
) -> Result<Json<RadioOption>, Json<String>> {
    radio_manager
        .add_option(group_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// POST 端点：选择选项
#[post("/api/radio/group/<group_id>/select", data = "<request>")]
pub fn select_option(
    group_id: &str,
    request: Json<SelectOptionRequest>,
    radio_manager: &State<RadioManager>,
) -> Result<Json<RadioGroup>, Json<String>> {
    radio_manager
        .select_option(group_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取选中的选项
#[get("/api/radio/group/<group_id>/selected")]
pub fn get_selected_options(
    group_id: &str,
    radio_manager: &State<RadioManager>,
) -> Result<Json<Vec<RadioOption>>, Json<String>> {
    radio_manager
        .get_selected_options(group_id)
        .map(Json)
        .ok_or_else(|| Json("Radio group not found".to_string()))
}

/// GET 端点：获取示例选项组
#[get("/api/radio/example")]
pub fn radio_example(radio_manager: &State<RadioManager>) -> Json<RadioGroup> {
    let request = CreateRadioGroupRequest {
        name: "颜色选择".to_string(),
        description: Some("选择一个颜色".to_string()),
        multiple: false,
        options: vec![
            CreateRadioOptionRequest {
                label: "红色".to_string(),
                value: serde_json::Value::String("#FF0000".to_string()),
                disabled: false,
                default: true,
                description: Some("鲜艳的红色".to_string()),
                metadata: None,
            },
            CreateRadioOptionRequest {
                label: "绿色".to_string(),
                value: serde_json::Value::String("#00FF00".to_string()),
                disabled: false,
                default: false,
                description: Some("清新的绿色".to_string()),
                metadata: None,
            },
            CreateRadioOptionRequest {
                label: "蓝色".to_string(),
                value: serde_json::Value::String("#0000FF".to_string()),
                disabled: false,
                default: false,
                description: Some("深邃的蓝色".to_string()),
                metadata: None,
            },
        ],
    };

    let group = radio_manager.create_group(request).unwrap();
    Json(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_group() {
        let manager = RadioManager::new();
        let request = CreateRadioGroupRequest {
            name: "Test Group".to_string(),
            description: Some("Test Description".to_string()),
            multiple: false,
            options: vec![CreateRadioOptionRequest {
                label: "Option 1".to_string(),
                value: serde_json::Value::String("value1".to_string()),
                disabled: false,
                default: true,
                description: None,
                metadata: None,
            }],
        };

        let group = manager.create_group(request).unwrap();
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.options.len(), 1);
        assert!(group.selected_option_id.is_some());
    }

    #[test]
    fn test_select_option() {
        let manager = RadioManager::new();
        let request = CreateRadioGroupRequest {
            name: "Test Group".to_string(),
            description: None,
            multiple: false,
            options: vec![
                CreateRadioOptionRequest {
                    label: "Option 1".to_string(),
                    value: serde_json::Value::String("value1".to_string()),
                    disabled: false,
                    default: false,
                    description: None,
                    metadata: None,
                },
                CreateRadioOptionRequest {
                    label: "Option 2".to_string(),
                    value: serde_json::Value::String("value2".to_string()),
                    disabled: false,
                    default: false,
                    description: None,
                    metadata: None,
                },
            ],
        };

        let group = manager.create_group(request).unwrap();
        let option_id = group.options[1].id.clone();

        let select_request = SelectOptionRequest {
            option_id: Some(option_id.clone()),
            option_ids: None,
        };

        let updated_group = manager.select_option(&group.id, select_request).unwrap();
        assert_eq!(updated_group.selected_option_id, Some(option_id));
    }

    #[test]
    fn test_get_selected_options() {
        let manager = RadioManager::new();
        let request = CreateRadioGroupRequest {
            name: "Test Group".to_string(),
            description: None,
            multiple: false,
            options: vec![CreateRadioOptionRequest {
                label: "Option 1".to_string(),
                value: serde_json::Value::String("value1".to_string()),
                disabled: false,
                default: true,
                description: None,
                metadata: None,
            }],
        };

        let group = manager.create_group(request).unwrap();
        let selected = manager.get_selected_options(&group.id).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].label, "Option 1");
    }
}
