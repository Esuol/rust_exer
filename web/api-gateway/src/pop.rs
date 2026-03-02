use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 人口统计数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopDataItem {
    /// 地区/类别名称
    pub label: String,
    /// 人口数量
    pub population: u64,
    /// 占比百分比（0-100）
    pub percentage: f64,
    /// 增长率（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub growth_rate: Option<f64>,
    /// 备注（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// 人口统计响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PopStatsResponse {
    /// 数据项列表
    pub items: Vec<PopDataItem>,
    /// 总人口数
    pub total: u64,
    /// 平均人口数
    pub average: f64,
    /// 最大人口数
    pub max: u64,
    /// 最小人口数
    pub min: u64,
    /// 数据项数量
    pub count: usize,
}

/// 人口统计错误
#[derive(Debug, Clone)]
pub enum PopError {
    EmptyData,
    InvalidData(String),
    #[allow(dead_code)]
    InvalidPopulation(String),
}

impl fmt::Display for PopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PopError::EmptyData => write!(f, "数据不能为空"),
            PopError::InvalidData(msg) => write!(f, "无效的数据: {}", msg),
            PopError::InvalidPopulation(label) => {
                write!(f, "数据项 {} 的人口数无效", label)
            }
        }
    }
}

impl std::error::Error for PopError {}

/// 人口统计数据请求
#[derive(Debug, Deserialize)]
pub struct PopStatsRequest {
    /// 数据项列表，每个项包含标签和人口数
    pub data: Vec<PopDataItemInput>,
}

/// 人口统计数据输入项
#[derive(Debug, Deserialize)]
pub struct PopDataItemInput {
    /// 地区/类别名称
    pub label: String,
    /// 人口数量
    pub population: u64,
    /// 增长率（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub growth_rate: Option<f64>,
    /// 备注（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// 计算人口统计数据
///
/// # 参数
/// * `data` - 输入数据项列表
///
/// # 返回
/// 计算后的人口统计数据，包含百分比、平均值、最大值、最小值等信息
///
/// # 错误
/// 如果数据为空或包含无效数据，返回错误
pub fn calculate_pop_stats(data: &[PopDataItemInput]) -> Result<PopStatsResponse, PopError> {
    if data.is_empty() {
        return Err(PopError::EmptyData);
    }

    // 计算总和
    let total: u64 = data.iter().map(|item| item.population).sum();

    if total == 0 {
        return Err(PopError::InvalidData("总人口数不能为零".to_string()));
    }

    // 计算统计数据
    let populations: Vec<u64> = data.iter().map(|item| item.population).collect();
    let max = *populations.iter().max().unwrap();
    let min = *populations.iter().min().unwrap();
    let average = total as f64 / data.len() as f64;

    // 计算每个数据项的百分比
    let items: Vec<PopDataItem> = data
        .iter()
        .map(|input| {
            let percentage = (input.population as f64 / total as f64) * 100.0;

            PopDataItem {
                label: input.label.clone(),
                population: input.population,
                percentage: round_to_decimal(percentage, 2),
                growth_rate: input.growth_rate,
                note: input.note.clone(),
            }
        })
        .collect();

    Ok(PopStatsResponse {
        items,
        total,
        average: round_to_decimal(average, 2),
        max,
        min,
        count: data.len(),
    })
}

/// 四舍五入到指定小数位
fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// 从简单的键值对创建人口统计数据
///
/// # 参数
/// * `data` - 键值对映射（标签 -> 人口数）
///
/// # 返回
/// 人口统计响应
#[allow(dead_code)]
pub fn create_pop_stats_from_map(
    data: std::collections::HashMap<String, u64>,
) -> Result<PopStatsResponse, PopError> {
    let items: Vec<PopDataItemInput> = data
        .into_iter()
        .map(|(label, population)| PopDataItemInput {
            label,
            population,
            growth_rate: None,
            note: None,
        })
        .collect();

    calculate_pop_stats(&items)
}

/// GET 端点：计算人口统计数据
///
/// 使用查询参数传递数据，格式：?data=label1:value1,label2:value2
#[get("/api/pop/calculate?<data>")]
pub fn calculate_pop_get(data: Option<&str>) -> Result<Json<PopStatsResponse>, Json<String>> {
    let data_str = data.ok_or_else(|| Json("缺少数据参数".to_string()))?;

    // 解析查询参数格式：label1:value1,label2:value2
    let items: Result<Vec<PopDataItemInput>, _> = data_str
        .split(',')
        .map(|pair| {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() != 2 {
                return Err("格式错误：应为 label:population".to_string());
            }
            let label = parts[0].trim().to_string();
            let population = parts[1]
                .trim()
                .parse::<u64>()
                .map_err(|_| "无效的人口数".to_string())?;

            Ok(PopDataItemInput {
                label,
                population,
                growth_rate: None,
                note: None,
            })
        })
        .collect();

    let items = items.map_err(|e| Json(e))?;
    let result = calculate_pop_stats(&items).map_err(|e| Json(e.to_string()))?;

    Ok(Json(result))
}

/// POST 端点：计算人口统计数据
///
/// 接收 JSON 格式的人口统计数据请求
#[post("/api/pop/calculate", data = "<request>")]
pub fn calculate_pop_post(
    request: Json<PopStatsRequest>,
) -> Result<Json<PopStatsResponse>, Json<String>> {
    let result = calculate_pop_stats(&request.data).map_err(|e| Json(e.to_string()))?;

    Ok(Json(result))
}

/// GET 端点：获取示例人口统计数据
#[get("/api/pop/example")]
pub fn pop_example() -> Json<PopStatsResponse> {
    let example_data = vec![
        PopDataItemInput {
            label: "北京".to_string(),
            population: 21540000,
            growth_rate: Some(1.2),
            note: Some("首都".to_string()),
        },
        PopDataItemInput {
            label: "上海".to_string(),
            population: 24870000,
            growth_rate: Some(0.8),
            note: Some("经济中心".to_string()),
        },
        PopDataItemInput {
            label: "广州".to_string(),
            population: 18810000,
            growth_rate: Some(1.5),
            note: None,
        },
        PopDataItemInput {
            label: "深圳".to_string(),
            population: 17560000,
            growth_rate: Some(2.1),
            note: Some("科技城市".to_string()),
        },
        PopDataItemInput {
            label: "杭州".to_string(),
            population: 12200000,
            growth_rate: Some(1.8),
            note: None,
        },
    ];

    let result = calculate_pop_stats(&example_data).expect("示例数据应该有效");

    Json(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_pop_stats() {
        let data = vec![
            PopDataItemInput {
                label: "城市A".to_string(),
                population: 1000000,
                growth_rate: None,
                note: None,
            },
            PopDataItemInput {
                label: "城市B".to_string(),
                population: 2000000,
                growth_rate: None,
                note: None,
            },
            PopDataItemInput {
                label: "城市C".to_string(),
                population: 3000000,
                growth_rate: None,
                note: None,
            },
        ];

        let result = calculate_pop_stats(&data).unwrap();
        assert_eq!(result.total, 6000000);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.count, 3);
        assert_eq!(result.max, 3000000);
        assert_eq!(result.min, 1000000);
        assert!((result.average - 2000000.0).abs() < 0.01);
        assert!((result.items[0].percentage - 16.67).abs() < 0.1);
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let result = calculate_pop_stats(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_total() {
        let data = vec![PopDataItemInput {
            label: "城市A".to_string(),
            population: 0,
            growth_rate: None,
            note: None,
        }];
        let result = calculate_pop_stats(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_percentage_calculation() {
        let data = vec![
            PopDataItemInput {
                label: "城市A".to_string(),
                population: 50,
                growth_rate: None,
                note: None,
            },
            PopDataItemInput {
                label: "城市B".to_string(),
                population: 30,
                growth_rate: None,
                note: None,
            },
            PopDataItemInput {
                label: "城市C".to_string(),
                population: 20,
                growth_rate: None,
                note: None,
            },
        ];

        let result = calculate_pop_stats(&data).unwrap();
        assert_eq!(result.total, 100);
        assert!((result.items[0].percentage - 50.0).abs() < 0.01);
        assert!((result.items[1].percentage - 30.0).abs() < 0.01);
        assert!((result.items[2].percentage - 20.0).abs() < 0.01);
    }
}
