use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 饼图数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieDataItem {
    /// 标签名称
    pub label: String,
    /// 数值
    pub value: f64,
    /// 百分比（0-100）
    pub percentage: f64,
    /// 角度（0-360度）
    pub angle: f64,
    /// 颜色（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// 饼图数据响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PieChartResponse {
    /// 数据项列表
    pub items: Vec<PieDataItem>,
    /// 总和
    pub total: f64,
    /// 是否包含负数
    pub has_negative: bool,
}

/// 饼图计算错误
#[derive(Debug, Clone)]
pub enum PieError {
    EmptyData,
    NegativeValue(String),
    InvalidData(String),
}

impl fmt::Display for PieError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PieError::EmptyData => write!(f, "数据不能为空"),
            PieError::NegativeValue(label) => {
                write!(f, "数据项 {} 的值不能为负数", label)
            }
            PieError::InvalidData(msg) => write!(f, "无效的数据: {}", msg),
        }
    }
}

impl std::error::Error for PieError {}

/// 饼图数据请求
#[derive(Debug, Deserialize)]
pub struct PieChartRequest {
    /// 数据项列表，每个项包含标签和值
    pub data: Vec<PieDataItemInput>,
    /// 是否允许负数（默认false）
    #[serde(default)]
    pub allow_negative: bool,
}

/// 饼图数据输入项
#[derive(Debug, Deserialize)]
pub struct PieDataItemInput {
    /// 标签名称
    pub label: String,
    /// 数值
    pub value: f64,
    /// 颜色（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// 计算饼图数据
///
/// # 参数
/// * `data` - 输入数据项列表
/// * `allow_negative` - 是否允许负数
///
/// # 返回
/// 计算后的饼图数据，包含百分比和角度信息
///
/// # 错误
/// 如果数据为空或包含负数（且不允许负数），返回错误
pub fn calculate_pie_chart(
    data: &[PieDataItemInput],
    allow_negative: bool,
) -> Result<PieChartResponse, PieError> {
    if data.is_empty() {
        return Err(PieError::EmptyData);
    }

    // 检查是否有负数
    let has_negative = data.iter().any(|item| item.value < 0.0);
    if has_negative && !allow_negative {
        let negative_item = data
            .iter()
            .find(|item| item.value < 0.0)
            .map(|item| item.label.clone())
            .unwrap_or_default();
        return Err(PieError::NegativeValue(negative_item));
    }

    // 计算总和（如果允许负数，使用绝对值）
    let total: f64 = if allow_negative && has_negative {
        data.iter().map(|item| item.value.abs()).sum()
    } else {
        data.iter().map(|item| item.value).sum()
    };

    if total == 0.0 {
        return Err(PieError::InvalidData("所有值的总和不能为零".to_string()));
    }

    // 计算每个数据项的百分比和角度
    let items: Vec<PieDataItem> = data
        .iter()
        .map(|input| {
            let value = if allow_negative && has_negative {
                input.value.abs()
            } else {
                input.value
            };

            let percentage = (value / total) * 100.0;
            let angle = (value / total) * 360.0;

            PieDataItem {
                label: input.label.clone(),
                value: input.value, // 保留原始值
                percentage: round_to_decimal(percentage, 2),
                angle: round_to_decimal(angle, 2),
                color: input.color.clone(),
            }
        })
        .collect();

    Ok(PieChartResponse {
        items,
        total: round_to_decimal(total, 2),
        has_negative,
    })
}

/// 四舍五入到指定小数位
fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// 从简单的键值对创建饼图数据
///
/// # 参数
/// * `data` - 键值对映射（标签 -> 值）
///
/// # 返回
/// 饼图数据响应
pub fn create_pie_chart_from_map(
    data: std::collections::HashMap<String, f64>,
) -> Result<PieChartResponse, PieError> {
    let items: Vec<PieDataItemInput> = data
        .into_iter()
        .map(|(label, value)| PieDataItemInput {
            label,
            value,
            color: None,
        })
        .collect();

    calculate_pie_chart(&items, false)
}

/// 生成默认颜色列表
///
/// # 参数
/// * `count` - 需要的颜色数量
///
/// # 返回
/// 颜色十六进制字符串列表
pub fn generate_default_colors(count: usize) -> Vec<String> {
    let default_palette = vec![
        "#FF6384", "#36A2EB", "#FFCE56", "#4BC0C0", "#9966FF", "#FF9F40", "#FF6384", "#C9CBCF",
        "#4BC0C0", "#FF6384",
    ];

    (0..count)
        .map(|i| default_palette[i % default_palette.len()].to_string())
        .collect()
}

/// 为饼图数据项分配颜色
///
/// # 参数
/// * `response` - 饼图响应数据
///
/// # 返回
/// 更新后的饼图响应（包含颜色信息）
pub fn assign_colors(mut response: PieChartResponse) -> PieChartResponse {
    let colors = generate_default_colors(response.items.len());

    for (item, color) in response.items.iter_mut().zip(colors.iter()) {
        if item.color.is_none() {
            item.color = Some(color.clone());
        }
    }

    response
}

/// GET 端点：计算饼图数据
///
/// 使用查询参数传递数据，格式：?data=label1:value1,label2:value2
#[get("/api/pie/calculate?<data>")]
pub fn calculate_pie_get(data: Option<&str>) -> Result<Json<PieChartResponse>, Json<String>> {
    let data_str = data.ok_or_else(|| Json("缺少数据参数".to_string()))?;

    // 解析查询参数格式：label1:value1,label2:value2
    let items: Result<Vec<PieDataItemInput>, _> = data_str
        .split(',')
        .map(|pair| {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() != 2 {
                return Err("格式错误：应为 label:value".to_string());
            }
            let label = parts[0].trim().to_string();
            let value = parts[1]
                .trim()
                .parse::<f64>()
                .map_err(|_| "无效的数值".to_string())?;

            Ok(PieDataItemInput {
                label,
                value,
                color: None,
            })
        })
        .collect();

    let items = items.map_err(|e| Json(e))?;
    let result = calculate_pie_chart(&items, false).map_err(|e| Json(e.to_string()))?;

    Ok(Json(assign_colors(result)))
}

/// POST 端点：计算饼图数据
///
/// 接收 JSON 格式的饼图数据请求
#[post("/api/pie/calculate", data = "<request>")]
pub fn calculate_pie_post(
    request: Json<PieChartRequest>,
) -> Result<Json<PieChartResponse>, Json<String>> {
    let result = calculate_pie_chart(&request.data, request.allow_negative)
        .map_err(|e| Json(e.to_string()))?;

    Ok(Json(assign_colors(result)))
}

/// GET 端点：获取示例饼图数据
#[get("/api/pie/example")]
pub fn pie_example() -> Json<PieChartResponse> {
    let example_data = vec![
        PieDataItemInput {
            label: "苹果".to_string(),
            value: 30.0,
            color: Some("#FF6384".to_string()),
        },
        PieDataItemInput {
            label: "香蕉".to_string(),
            value: 25.0,
            color: Some("#36A2EB".to_string()),
        },
        PieDataItemInput {
            label: "橙子".to_string(),
            value: 20.0,
            color: Some("#FFCE56".to_string()),
        },
        PieDataItemInput {
            label: "葡萄".to_string(),
            value: 15.0,
            color: Some("#4BC0C0".to_string()),
        },
        PieDataItemInput {
            label: "其他".to_string(),
            value: 10.0,
            color: Some("#9966FF".to_string()),
        },
    ];

    let result = calculate_pie_chart(&example_data, false).expect("示例数据应该有效");

    Json(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_pie_chart() {
        let data = vec![
            PieDataItemInput {
                label: "A".to_string(),
                value: 50.0,
                color: None,
            },
            PieDataItemInput {
                label: "B".to_string(),
                value: 30.0,
                color: None,
            },
            PieDataItemInput {
                label: "C".to_string(),
                value: 20.0,
                color: None,
            },
        ];

        let result = calculate_pie_chart(&data, false).unwrap();
        assert_eq!(result.total, 100.0);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].percentage, 50.0);
        assert_eq!(result.items[0].angle, 180.0);
    }

    #[test]
    fn test_calculate_pie_chart_v1() {
        let data = vec![
            PieDataItemInput {
                label: "A".to_string(),
                value: 50.0,
                color: None,
            },
            PieDataItemInput {
                label: "B".to_string(),
                value: 30.0,
                color: None,
            },
            PieDataItemInput {
                label: "C".to_string(),
                value: 20.0,
                color: None,
            },
        ];

        let result = calculate_pie_chart(&data, false).unwrap();
        assert_eq!(result.total, 100.0);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].percentage, 50.0);
        assert_eq!(result.items[0].angle, 180.0);
    }

    #[test]
    fn test_calculate_pie_chart_v2() {
        let data = vec![
            PieDataItemInput {
                label: "A".to_string(),
                value: 50.0,
                color: None,
            },
            PieDataItemInput {
                label: "B".to_string(),
                value: 30.0,
                color: None,
            },
            PieDataItemInput {
                label: "C".to_string(),
                value: 20.0,
                color: None,
            },
        ];

        let result = calculate_pie_chart(&data, false).unwrap();
        assert_eq!(result.total, 100.0);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].percentage, 50.0);
        assert_eq!(result.items[0].angle, 180.0);
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data_v1() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data_v2() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data_v4() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data_v5() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data_v6() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data_v5() {
        let data = vec![];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_value() {
        let data = vec![PieDataItemInput {
            label: "A".to_string(),
            value: -10.0,
            color: None,
        }];
        let result = calculate_pie_chart(&data, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_default_colors() {
        let colors = generate_default_colors(5);
        assert_eq!(colors.len(), 5);
        assert!(colors[0].starts_with('#'));
    }
}
