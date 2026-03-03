use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepDataItem {
    /// 标识符/键
    pub key: String,
    /// 值
    pub value: f64,
    /// 标签（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// 筛选条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    /// 大于
    GreaterThan(f64),
    /// 大于等于
    GreaterThanOrEqual(f64),
    /// 小于
    LessThan(f64),
    /// 小于等于
    LessThanOrEqual(f64),
    /// 等于
    Equal(f64),
    /// 不等于
    NotEqual(f64),
    /// 在范围内（最小值，最大值）
    InRange(f64, f64),
    /// 不在范围内（最小值，最大值）
    NotInRange(f64, f64),
}

/// 筛选响应
#[derive(Debug, Serialize, Deserialize)]
pub struct KeepFilterResponse {
    /// 保留的数据项列表
    pub kept: Vec<KeepDataItem>,
    /// 过滤掉的数据项列表
    pub filtered: Vec<KeepDataItem>,
    /// 保留的数量
    pub kept_count: usize,
    /// 过滤掉的数量
    pub filtered_count: usize,
    /// 原始总数
    pub total_count: usize,
    /// 保留率（百分比）
    pub keep_rate: f64,
}

/// 筛选错误
#[derive(Debug, Clone)]
pub enum KeepError {
    EmptyData,
    #[allow(dead_code)]
    InvalidCondition(String),
    #[allow(dead_code)]
    InvalidData(String),
}

impl fmt::Display for KeepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeepError::EmptyData => write!(f, "数据不能为空"),
            KeepError::InvalidCondition(msg) => write!(f, "无效的筛选条件: {}", msg),
            KeepError::InvalidData(msg) => write!(f, "无效的数据: {}", msg),
        }
    }
}

impl std::error::Error for KeepError {}

/// 筛选请求
#[derive(Debug, Deserialize)]
pub struct KeepFilterRequest {
    /// 数据项列表
    pub data: Vec<KeepDataItemInput>,
    /// 筛选条件
    pub condition: FilterConditionInput,
}

/// 数据项输入
#[derive(Debug, Deserialize)]
pub struct KeepDataItemInput {
    /// 标识符/键
    pub key: String,
    /// 值
    pub value: f64,
    /// 标签（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// 筛选条件输入（用于 JSON 反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FilterConditionInput {
    #[serde(rename = "gt")]
    GreaterThan(f64),
    #[serde(rename = "gte")]
    GreaterThanOrEqual(f64),
    #[serde(rename = "lt")]
    LessThan(f64),
    #[serde(rename = "lte")]
    LessThanOrEqual(f64),
    #[serde(rename = "eq")]
    Equal(f64),
    #[serde(rename = "ne")]
    NotEqual(f64),
    #[serde(rename = "range")]
    InRange(f64, f64),
    #[serde(rename = "not_range")]
    NotInRange(f64, f64),
}

impl From<FilterConditionInput> for FilterCondition {
    fn from(input: FilterConditionInput) -> Self {
        match input {
            FilterConditionInput::GreaterThan(v) => FilterCondition::GreaterThan(v),
            FilterConditionInput::GreaterThanOrEqual(v) => FilterCondition::GreaterThanOrEqual(v),
            FilterConditionInput::LessThan(v) => FilterCondition::LessThan(v),
            FilterConditionInput::LessThanOrEqual(v) => FilterCondition::LessThanOrEqual(v),
            FilterConditionInput::Equal(v) => FilterCondition::Equal(v),
            FilterConditionInput::NotEqual(v) => FilterCondition::NotEqual(v),
            FilterConditionInput::InRange(min, max) => FilterCondition::InRange(min, max),
            FilterConditionInput::NotInRange(min, max) => FilterCondition::NotInRange(min, max),
        }
    }
}

/// 检查值是否满足筛选条件
fn matches_condition(value: f64, condition: &FilterCondition) -> bool {
    match condition {
        FilterCondition::GreaterThan(threshold) => value > *threshold,
        FilterCondition::GreaterThanOrEqual(threshold) => value >= *threshold,
        FilterCondition::LessThan(threshold) => value < *threshold,
        FilterCondition::LessThanOrEqual(threshold) => value <= *threshold,
        FilterCondition::Equal(threshold) => (value - *threshold).abs() < f64::EPSILON,
        FilterCondition::NotEqual(threshold) => (value - *threshold).abs() >= f64::EPSILON,
        FilterCondition::InRange(min, max) => value >= *min && value <= *max,
        FilterCondition::NotInRange(min, max) => value < *min || value > *max,
    }
}

/// 筛选数据，保留满足条件的数据项
///
/// # 参数
/// * `data` - 输入数据项列表
/// * `condition` - 筛选条件
///
/// # 返回
/// 筛选结果，包含保留和过滤掉的数据项
///
/// # 错误
/// 如果数据为空或条件无效，返回错误
pub fn filter_keep_data(
    data: &[KeepDataItemInput],
    condition: FilterCondition,
) -> Result<KeepFilterResponse, KeepError> {
    if data.is_empty() {
        return Err(KeepError::EmptyData);
    }

    let mut kept = Vec::new();
    let mut filtered = Vec::new();

    for item in data {
        let data_item = KeepDataItem {
            key: item.key.clone(),
            value: item.value,
            label: item.label.clone(),
            metadata: item.metadata.clone(),
        };

        if matches_condition(item.value, &condition) {
            kept.push(data_item);
        } else {
            filtered.push(data_item);
        }
    }

    let total_count = data.len();
    let kept_count = kept.len();
    let filtered_count = filtered.len();
    let keep_rate = if total_count > 0 {
        (kept_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    Ok(KeepFilterResponse {
        kept,
        filtered,
        kept_count,
        filtered_count,
        total_count,
        keep_rate: round_to_decimal(keep_rate, 2),
    })
}

/// 四舍五入到指定小数位
fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// GET 端点：筛选数据
///
/// 使用查询参数传递数据，格式：?data=key1:value1,key2:value2&condition=gt:10
#[get("/api/keep/filter?<data>&<condition>")]
pub fn filter_keep_get(
    data: Option<&str>,
    condition: Option<&str>,
) -> Result<Json<KeepFilterResponse>, Json<String>> {
    let data_str = data.ok_or_else(|| Json("缺少数据参数".to_string()))?;
    let condition_str = condition.ok_or_else(|| Json("缺少筛选条件参数".to_string()))?;

    // 解析数据：key1:value1,key2:value2
    let items: Result<Vec<KeepDataItemInput>, _> = data_str
        .split(',')
        .map(|pair| {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() != 2 {
                return Err("格式错误：应为 key:value".to_string());
            }
            let key = parts[0].trim().to_string();
            let value = parts[1]
                .trim()
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;

            Ok(KeepDataItemInput {
                key,
                value,
                label: None,
                metadata: None,
            })
        })
        .collect();

    let items = items.map_err(|e| Json(e))?;

    // 解析条件：type:value 或 type:min:max
    let condition = parse_condition_from_str(condition_str)
        .map_err(|e| Json(format!("无效的筛选条件: {}", e)))?;

    let result = filter_keep_data(&items, condition).map_err(|e| Json(e.to_string()))?;

    Ok(Json(result))
}

/// 从字符串解析筛选条件
fn parse_condition_from_str(s: &str) -> Result<FilterCondition, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.is_empty() {
        return Err("条件格式错误".to_string());
    }

    let condition_type = parts[0].trim();
    match condition_type {
        "gt" => {
            if parts.len() != 2 {
                return Err("gt 条件需要1个值".to_string());
            }
            let value = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;
            Ok(FilterCondition::GreaterThan(value))
        }
        "gte" => {
            if parts.len() != 2 {
                return Err("gte 条件需要1个值".to_string());
            }
            let value = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;
            Ok(FilterCondition::GreaterThanOrEqual(value))
        }
        "lt" => {
            if parts.len() != 2 {
                return Err("lt 条件需要1个值".to_string());
            }
            let value = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;
            Ok(FilterCondition::LessThan(value))
        }
        "lte" => {
            if parts.len() != 2 {
                return Err("lte 条件需要1个值".to_string());
            }
            let value = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;
            Ok(FilterCondition::LessThanOrEqual(value))
        }
        "eq" => {
            if parts.len() != 2 {
                return Err("eq 条件需要1个值".to_string());
            }
            let value = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;
            Ok(FilterCondition::Equal(value))
        }
        "ne" => {
            if parts.len() != 2 {
                return Err("ne 条件需要1个值".to_string());
            }
            let value = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;
            Ok(FilterCondition::NotEqual(value))
        }
        "range" => {
            if parts.len() != 3 {
                return Err("range 条件需要2个值（最小值:最大值）".to_string());
            }
            let min = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的最小值".to_string())?;
            let max = parts[2]
                .parse::<f64>()
                .map_err(|_| "无效的最大值".to_string())?;
            Ok(FilterCondition::InRange(min, max))
        }
        "not_range" => {
            if parts.len() != 3 {
                return Err("not_range 条件需要2个值（最小值:最大值）".to_string());
            }
            let min = parts[1]
                .parse::<f64>()
                .map_err(|_| "无效的最小值".to_string())?;
            let max = parts[2]
                .parse::<f64>()
                .map_err(|_| "无效的最大值".to_string())?;
            Ok(FilterCondition::NotInRange(min, max))
        }
        _ => Err(format!("未知的筛选条件类型: {}", condition_type)),
    }
}

/// POST 端点：筛选数据
///
/// 接收 JSON 格式的筛选请求
#[post("/api/keep/filter", data = "<request>")]
pub fn filter_keep_post(
    request: Json<KeepFilterRequest>,
) -> Result<Json<KeepFilterResponse>, Json<String>> {
    let condition: FilterCondition = request.condition.clone().into();
    let result = filter_keep_data(&request.data, condition).map_err(|e| Json(e.to_string()))?;

    Ok(Json(result))
}

/// GET 端点：获取示例数据
#[get("/api/keep/example")]
pub fn keep_example() -> Json<KeepFilterResponse> {
    let example_data = vec![
        KeepDataItemInput {
            key: "item1".to_string(),
            value: 15.5,
            label: Some("项目A".to_string()),
            metadata: None,
        },
        KeepDataItemInput {
            key: "item2".to_string(),
            value: 25.0,
            label: Some("项目B".to_string()),
            metadata: None,
        },
        KeepDataItemInput {
            key: "item3".to_string(),
            value: 8.3,
            label: Some("项目C".to_string()),
            metadata: None,
        },
        KeepDataItemInput {
            key: "item4".to_string(),
            value: 30.7,
            label: Some("项目D".to_string()),
            metadata: None,
        },
        KeepDataItemInput {
            key: "item5".to_string(),
            value: 12.1,
            label: Some("项目E".to_string()),
            metadata: None,
        },
    ];

    // 使用大于等于 15 的条件作为示例
    let condition = FilterCondition::GreaterThanOrEqual(15.0);
    let result = filter_keep_data(&example_data, condition).expect("示例数据应该有效");

    Json(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_keep_data_greater_than() {
        let data = vec![
            KeepDataItemInput {
                key: "a".to_string(),
                value: 10.0,
                label: None,
                metadata: None,
            },
            KeepDataItemInput {
                key: "b".to_string(),
                value: 20.0,
                label: None,
                metadata: None,
            },
            KeepDataItemInput {
                key: "c".to_string(),
                value: 5.0,
                label: None,
                metadata: None,
            },
        ];

        let condition = FilterCondition::GreaterThan(10.0);
        let result = filter_keep_data(&data, condition).unwrap();

        assert_eq!(result.kept_count, 1);
        assert_eq!(result.filtered_count, 2);
        assert_eq!(result.kept[0].key, "b");
        assert!((result.keep_rate - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_filter_keep_data_in_range() {
        let data = vec![
            KeepDataItemInput {
                key: "a".to_string(),
                value: 15.0,
                label: None,
                metadata: None,
            },
            KeepDataItemInput {
                key: "b".to_string(),
                value: 25.0,
                label: None,
                metadata: None,
            },
            KeepDataItemInput {
                key: "c".to_string(),
                value: 30.0,
                label: None,
                metadata: None,
            },
        ];

        let condition = FilterCondition::InRange(20.0, 30.0);
        let result = filter_keep_data(&data, condition).unwrap();

        assert_eq!(result.kept_count, 2);
        assert_eq!(result.filtered_count, 1);
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let condition = FilterCondition::GreaterThan(10.0);
        let result = filter_keep_data(&data, condition);
        assert!(result.is_err());
    }

    #[test]
    fn test_matches_condition() {
        assert!(matches_condition(15.0, &FilterCondition::GreaterThan(10.0)));
        assert!(!matches_condition(5.0, &FilterCondition::GreaterThan(10.0)));
        assert!(matches_condition(
            10.0,
            &FilterCondition::GreaterThanOrEqual(10.0)
        ));
        assert!(matches_condition(5.0, &FilterCondition::LessThan(10.0)));
        assert!(matches_condition(10.0, &FilterCondition::Equal(10.0)));
        assert!(matches_condition(
            15.0,
            &FilterCondition::InRange(10.0, 20.0)
        ));
        assert!(!matches_condition(
            25.0,
            &FilterCondition::InRange(10.0, 20.0)
        ));
    }
}
