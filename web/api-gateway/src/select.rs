use crate::table;
use rocket::{get, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 查询请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectRequest {
    /// 表格ID
    pub table_id: String,
    /// 选择的字段（如果为空则选择所有字段）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    /// 查询条件（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<WhereClause>,
    /// 排序条件（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<OrderBy>>,
    /// 聚合函数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<Vec<AggregateFunction>>,
    /// 分页参数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
}

/// WHERE 子句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    /// 字段名
    pub field: String,
    /// 操作符（eq, ne, gt, gte, lt, lte, like, in, not_in）
    pub operator: String,
    /// 值
    pub value: serde_json::Value,
}

/// 排序条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    /// 字段名
    pub field: String,
    /// 排序方向（asc, desc）
    pub direction: String,
}

/// 聚合函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateFunction {
    /// 函数名（count, sum, avg, max, min）
    pub function: String,
    /// 字段名
    pub field: String,
    /// 别名（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// 页码（从1开始）
    pub page: usize,
    /// 每页大小
    pub page_size: usize,
}

/// 查询响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectResponse {
    /// 查询结果
    pub data: Vec<HashMap<String, serde_json::Value>>,
    /// 聚合结果（如果有聚合函数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregates: Option<HashMap<String, serde_json::Value>>,
    /// 总数
    pub total: usize,
    /// 分页信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
}

/// 分页信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// 当前页
    pub page: usize,
    /// 每页大小
    pub page_size: usize,
    /// 总页数
    pub total_pages: usize,
    /// 是否有上一页
    pub has_previous: bool,
    /// 是否有下一页
    pub has_next: bool,
}

/// 检查行是否满足 WHERE 条件
fn matches_where_clause(
    row: &HashMap<String, serde_json::Value>,
    where_clause: &WhereClause,
) -> bool {
    let field_value = match row.get(&where_clause.field) {
        Some(v) => v,
        None => return false,
    };

    match where_clause.operator.as_str() {
        "eq" => field_value == &where_clause.value,
        "ne" => field_value != &where_clause.value,
        "gt" => {
            if let (Some(a), Some(b)) = (field_value.as_f64(), where_clause.value.as_f64()) {
                a > b
            } else {
                false
            }
        }
        "gte" => {
            if let (Some(a), Some(b)) = (field_value.as_f64(), where_clause.value.as_f64()) {
                a >= b
            } else {
                false
            }
        }
        "lt" => {
            if let (Some(a), Some(b)) = (field_value.as_f64(), where_clause.value.as_f64()) {
                a < b
            } else {
                false
            }
        }
        "lte" => {
            if let (Some(a), Some(b)) = (field_value.as_f64(), where_clause.value.as_f64()) {
                a <= b
            } else {
                false
            }
        }
        "like" => {
            if let (Some(a), Some(b)) = (field_value.as_str(), where_clause.value.as_str()) {
                a.contains(b)
            } else {
                false
            }
        }
        "in" => {
            if let Some(arr) = where_clause.value.as_array() {
                arr.contains(field_value)
            } else {
                false
            }
        }
        "not_in" => {
            if let Some(arr) = where_clause.value.as_array() {
                !arr.contains(field_value)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// 执行查询
///
/// # 参数
/// * `table_manager` - 表格管理器
/// * `request` - 查询请求
///
/// # 返回
/// 查询结果
pub fn execute_select(
    table_manager: &table::TableManager,
    request: SelectRequest,
) -> Result<SelectResponse, String> {
    // 获取表格数据
    let table_data = table_manager
        .get_table_data(&request.table_id, None, None)
        .ok_or_else(|| "Table not found".to_string())?;

    let mut rows: Vec<HashMap<String, serde_json::Value>> =
        table_data.rows.iter().map(|row| row.data.clone()).collect();

    // 应用 WHERE 条件
    if let Some(where_clause) = &request.where_clause {
        rows.retain(|row| matches_where_clause(row, where_clause));
    }

    // 应用排序
    if let Some(order_by_list) = &request.order_by {
        for order_by in order_by_list.iter().rev() {
            rows.sort_by(|a, b| {
                let a_val = a.get(&order_by.field);
                let b_val = b.get(&order_by.field);

                let cmp = match (a_val, b_val) {
                    (Some(a), Some(b)) => {
                        if let (Some(a_num), Some(b_num)) = (a.as_f64(), b.as_f64()) {
                            a_num
                                .partial_cmp(&b_num)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        } else if let (Some(a_str), Some(b_str)) = (a.as_str(), b.as_str()) {
                            a_str.cmp(b_str)
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                };

                if order_by.direction == "desc" {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
    }

    // 计算聚合函数
    let aggregates = if let Some(agg_functions) = &request.aggregate {
        let mut agg_results: HashMap<String, serde_json::Value> = HashMap::new();

        for agg in agg_functions {
            let alias = agg
                .alias
                .clone()
                .unwrap_or_else(|| format!("{}_{}", agg.function, agg.field));

            let result = match agg.function.as_str() {
                "count" => {
                    let count = rows.len();
                    serde_json::Value::Number(serde_json::Number::from(count))
                }
                "sum" => {
                    let sum: f64 = rows
                        .iter()
                        .filter_map(|row| row.get(&agg.field).and_then(|v| v.as_f64()))
                        .sum();
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(sum)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                }
                "avg" => {
                    let values: Vec<f64> = rows
                        .iter()
                        .filter_map(|row| row.get(&agg.field).and_then(|v| v.as_f64()))
                        .collect();
                    if values.is_empty() {
                        serde_json::Value::Number(serde_json::Number::from(0))
                    } else {
                        let avg = values.iter().sum::<f64>() / values.len() as f64;
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(avg)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        )
                    }
                }
                "max" => {
                    let max = rows
                        .iter()
                        .filter_map(|row| row.get(&agg.field).and_then(|v| v.as_f64()))
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    if let Some(max_val) = max {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(max_val)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        )
                    } else {
                        serde_json::Value::Null
                    }
                }
                "min" => {
                    let min = rows
                        .iter()
                        .filter_map(|row| row.get(&agg.field).and_then(|v| v.as_f64()))
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    if let Some(min_val) = min {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(min_val)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        )
                    } else {
                        serde_json::Value::Null
                    }
                }
                _ => serde_json::Value::Null,
            };

            agg_results.insert(alias, result);
        }

        Some(agg_results)
    } else {
        None
    };

    // 选择字段
    if let Some(fields) = &request.fields {
        rows = rows
            .into_iter()
            .map(|mut row| {
                let mut selected: HashMap<String, serde_json::Value> = HashMap::new();
                for field in fields {
                    if let Some(value) = row.remove(field) {
                        selected.insert(field.clone(), value);
                    }
                }
                selected
            })
            .collect();
    }

    let total = rows.len();

    // 应用分页
    let pagination_info = if let Some(pagination) = &request.pagination {
        let page = pagination.page.max(1);
        let page_size = pagination.page_size.max(1);
        let start = (page - 1) * page_size;
        let end = std::cmp::min(start + page_size, rows.len());
        let total_pages = (total + page_size - 1) / page_size;

        rows = rows[start..end].to_vec();

        Some(PaginationInfo {
            page,
            page_size,
            total_pages,
            has_previous: page > 1,
            has_next: page < total_pages,
        })
    } else {
        None
    };

    Ok(SelectResponse {
        data: rows,
        aggregates,
        total,
        pagination: pagination_info,
    })
}

/// POST 端点：执行查询
#[post("/api/select/query", data = "<request>")]
pub fn select_query(
    request: Json<SelectRequest>,
    table_manager: &State<table::TableManager>,
) -> Result<Json<SelectResponse>, Json<String>> {
    execute_select(table_manager, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：简单查询（通过查询参数）
#[get("/api/select/query?<table_id>&<field>&<operator>&<value>&<order_by>&<page>&<page_size>")]
pub fn select_query_get(
    table_id: Option<&str>,
    field: Option<&str>,
    operator: Option<&str>,
    value: Option<&str>,
    order_by: Option<&str>,
    page: Option<usize>,
    page_size: Option<usize>,
    table_manager: &State<table::TableManager>,
) -> Result<Json<SelectResponse>, Json<String>> {
    let table_id = table_id.ok_or_else(|| Json("缺少 table_id 参数".to_string()))?;

    let where_clause = if let (Some(field), Some(operator), Some(value)) = (field, operator, value)
    {
        // 尝试解析值为数字或字符串
        let json_value = if let Ok(num) = value.parse::<f64>() {
            serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap())
        } else {
            serde_json::Value::String(value.to_string())
        };

        Some(WhereClause {
            field: field.to_string(),
            operator: operator.to_string(),
            value: json_value,
        })
    } else {
        None
    };

    let order_by_list = if let Some(order_by_str) = order_by {
        // 解析排序：field:asc 或 field:desc
        order_by_str
            .split(',')
            .filter_map(|s| {
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() == 2 {
                    Some(OrderBy {
                        field: parts[0].trim().to_string(),
                        direction: parts[1].trim().to_string(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into()
    } else {
        None
    };

    let pagination = if let (Some(page), Some(size)) = (page, page_size) {
        Some(Pagination {
            page,
            page_size: size,
        })
    } else {
        None
    };

    let request = SelectRequest {
        table_id: table_id.to_string(),
        fields: None,
        where_clause,
        order_by: order_by_list,
        aggregate: None,
        pagination,
    };

    execute_select(table_manager, request)
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取查询示例
#[get("/api/select/example")]
pub fn select_example() -> Json<SelectRequest> {
    Json(SelectRequest {
        table_id: "table_example".to_string(),
        fields: Some(vec!["name".to_string(), "age".to_string()]),
        where_clause: Some(WhereClause {
            field: "age".to_string(),
            operator: "gte".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(18)),
        }),
        order_by: Some(vec![OrderBy {
            field: "age".to_string(),
            direction: "desc".to_string(),
        }]),
        aggregate: Some(vec![AggregateFunction {
            function: "count".to_string(),
            field: "*".to_string(),
            alias: Some("total_count".to_string()),
        }]),
        pagination: Some(Pagination {
            page: 1,
            page_size: 10,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_where_clause() {
        let mut row = HashMap::new();
        row.insert(
            "age".to_string(),
            serde_json::Value::Number(serde_json::Number::from(25)),
        );

        let where_clause = WhereClause {
            field: "age".to_string(),
            operator: "gte".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(18)),
        };

        assert!(matches_where_clause(&row, &where_clause));

        let where_clause2 = WhereClause {
            field: "age".to_string(),
            operator: "lt".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(20)),
        };

        assert!(!matches_where_clause(&row, &where_clause2));
    }

    #[test]
    fn test_matches_where_clause_string() {
        let mut row = HashMap::new();
        row.insert(
            "name".to_string(),
            serde_json::Value::String("John".to_string()),
        );

        let where_clause = WhereClause {
            field: "name".to_string(),
            operator: "like".to_string(),
            value: serde_json::Value::String("Jo".to_string()),
        };

        assert!(matches_where_clause(&row, &where_clause));
    }
}
