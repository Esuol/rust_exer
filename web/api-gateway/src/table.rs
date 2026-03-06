use chrono::Utc;
use rocket::{delete, get, post, put, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 表格行数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// 行ID
    pub id: String,
    /// 行数据（键值对）
    pub data: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 表格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// 表格ID
    pub id: String,
    /// 表格名称
    pub name: String,
    /// 表格描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 列定义
    pub columns: Vec<ColumnDefinition>,
    /// 行数
    pub row_count: usize,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 列定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// 列名
    pub name: String,
    /// 列类型（string, number, boolean, date）
    pub column_type: String,
    /// 是否必填
    #[serde(default)]
    pub required: bool,
    /// 默认值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}

/// 表格创建请求
#[derive(Debug, Deserialize)]
pub struct CreateTableRequest {
    /// 表格名称
    pub name: String,
    /// 表格描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 列定义
    pub columns: Vec<ColumnDefinition>,
}

/// 表格更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateTableRequest {
    /// 表格名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 表格描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 行数据创建请求
#[derive(Debug, Deserialize)]
pub struct CreateRowRequest {
    /// 行数据
    pub data: HashMap<String, serde_json::Value>,
}

/// 行数据更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateRowRequest {
    /// 行数据
    pub data: HashMap<String, serde_json::Value>,
}

/// 表格列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TableListResponse {
    /// 表格列表
    pub tables: Vec<TableInfo>,
    /// 总数
    pub total: usize,
}

/// 表格数据响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TableDataResponse {
    /// 表格信息
    pub table: TableInfo,
    /// 行数据列表
    pub rows: Vec<TableRow>,
    /// 总数
    pub total: usize,
    /// 当前页
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    /// 每页大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<usize>,
}

/// 表格统计响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TableStatsResponse {
    /// 表格ID
    pub table_id: String,
    /// 总行数
    pub total_rows: usize,
    /// 列统计
    pub column_stats: HashMap<String, ColumnStats>,
}

/// 列统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnStats {
    /// 非空值数量
    pub non_null_count: usize,
    /// 空值数量
    pub null_count: usize,
    /// 唯一值数量
    pub unique_count: usize,
    /// 最小值（如果是数字类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<serde_json::Value>,
    /// 最大值（如果是数字类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<serde_json::Value>,
}

/// 表格管理器
#[derive(Debug, Clone)]
pub struct TableManager {
    tables: Arc<RwLock<HashMap<String, TableInfo>>>,
    table_data: Arc<RwLock<HashMap<String, Vec<TableRow>>>>,
}

impl TableManager {
    /// 创建新的表格管理器
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            table_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建表格
    pub fn create_table(&self, request: CreateTableRequest) -> Result<TableInfo, String> {
        let table_id = format!("table_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        let table = TableInfo {
            id: table_id.clone(),
            name: request.name,
            description: request.description,
            columns: request.columns,
            row_count: 0,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut tables = self.tables.write().unwrap();
        tables.insert(table_id.clone(), table.clone());

        let mut table_data = self.table_data.write().unwrap();
        table_data.insert(table_id, Vec::new());

        Ok(table)
    }

    /// 更新表格
    pub fn update_table(
        &self,
        table_id: &str,
        request: UpdateTableRequest,
    ) -> Result<TableInfo, String> {
        let mut tables = self.tables.write().unwrap();

        let table = tables
            .get_mut(table_id)
            .ok_or_else(|| "Table not found".to_string())?;

        if let Some(name) = request.name {
            table.name = name;
        }
        if let Some(description) = request.description {
            table.description = Some(description);
        }
        table.updated_at = Utc::now().to_rfc3339();

        Ok(table.clone())
    }

    /// 删除表格
    pub fn delete_table(&self, table_id: &str) -> Result<(), String> {
        let mut tables = self.tables.write().unwrap();
        tables
            .remove(table_id)
            .ok_or_else(|| "Table not found".to_string())?;

        let mut table_data = self.table_data.write().unwrap();
        table_data.remove(table_id);

        Ok(())
    }

    /// 获取表格列表
    pub fn list_tables(&self) -> TableListResponse {
        let tables = self.tables.read().unwrap();
        let tables_vec: Vec<TableInfo> = tables.values().cloned().collect();

        TableListResponse {
            total: tables_vec.len(),
            tables: tables_vec,
        }
    }

    /// 获取表格信息
    pub fn get_table(&self, table_id: &str) -> Option<TableInfo> {
        let tables = self.tables.read().unwrap();
        tables.get(table_id).cloned()
    }

    /// 获取表格数据
    pub fn get_table_data(
        &self,
        table_id: &str,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Option<TableDataResponse> {
        let tables = self.tables.read().unwrap();
        let table = tables.get(table_id)?.clone();
        drop(tables);

        let table_data = self.table_data.read().unwrap();
        let mut rows = table_data.get(table_id)?.clone();
        drop(table_data);

        let total = rows.len();

        // 分页处理
        if let (Some(page), Some(size)) = (page, page_size) {
            let start = (page - 1) * size;
            let end = std::cmp::min(start + size, rows.len());
            rows = rows[start..end].to_vec();
        }

        Some(TableDataResponse {
            table,
            rows,
            total,
            page,
            page_size,
        })
    }

    /// 添加行数据
    pub fn add_row(&self, table_id: &str, request: CreateRowRequest) -> Result<TableRow, String> {
        let tables = self.tables.read().unwrap();
        let table = tables
            .get(table_id)
            .ok_or_else(|| "Table not found".to_string())?;

        // 验证数据
        for column in &table.columns {
            if column.required {
                if !request.data.contains_key(&column.name) {
                    return Err(format!("Required column '{}' is missing", column.name));
                }
            }
        }
        drop(tables);

        let row_id = format!("row_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        let row = TableRow {
            id: row_id.clone(),
            data: request.data,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut table_data = self.table_data.write().unwrap();
        let rows = table_data
            .get_mut(table_id)
            .ok_or_else(|| "Table not found".to_string())?;
        rows.push(row.clone());

        // 更新表格行数
        let mut tables = self.tables.write().unwrap();
        if let Some(table) = tables.get_mut(table_id) {
            table.row_count = rows.len();
            table.updated_at = Utc::now().to_rfc3339();
        }

        Ok(row)
    }

    /// 更新行数据
    pub fn update_row(
        &self,
        table_id: &str,
        row_id: &str,
        request: UpdateRowRequest,
    ) -> Result<TableRow, String> {
        let mut table_data = self.table_data.write().unwrap();
        let rows = table_data
            .get_mut(table_id)
            .ok_or_else(|| "Table not found".to_string())?;

        let row = rows
            .iter_mut()
            .find(|r| r.id == row_id)
            .ok_or_else(|| "Row not found".to_string())?;

        // 更新数据
        for (key, value) in request.data {
            row.data.insert(key, value);
        }
        row.updated_at = Utc::now().to_rfc3339();

        // 更新表格更新时间
        let mut tables = self.tables.write().unwrap();
        if let Some(table) = tables.get_mut(table_id) {
            table.updated_at = Utc::now().to_rfc3339();
        }

        Ok(row.clone())
    }

    /// 删除行数据
    pub fn delete_row(&self, table_id: &str, row_id: &str) -> Result<(), String> {
        let mut table_data = self.table_data.write().unwrap();
        let rows = table_data
            .get_mut(table_id)
            .ok_or_else(|| "Table not found".to_string())?;

        let index = rows
            .iter()
            .position(|r| r.id == row_id)
            .ok_or_else(|| "Row not found".to_string())?;

        rows.remove(index);

        // 更新表格行数
        let mut tables = self.tables.write().unwrap();
        if let Some(table) = tables.get_mut(table_id) {
            table.row_count = rows.len();
            table.updated_at = Utc::now().to_rfc3339();
        }

        Ok(())
    }

    /// 获取表格统计信息
    pub fn get_table_stats(&self, table_id: &str) -> Option<TableStatsResponse> {
        let tables = self.tables.read().unwrap();
        let table = tables.get(table_id)?.clone();
        drop(tables);

        let table_data = self.table_data.read().unwrap();
        let rows = table_data.get(table_id)?.clone();
        drop(table_data);

        let mut column_stats: HashMap<String, ColumnStats> = HashMap::new();

        for column in &table.columns {
            let mut non_null_count = 0;
            let mut null_count = 0;
            let mut unique_values = std::collections::HashSet::new();
            let mut numeric_values: Vec<f64> = Vec::new();

            for row in &rows {
                if let Some(value) = row.data.get(&column.name) {
                    if value.is_null() {
                        null_count += 1;
                    } else {
                        non_null_count += 1;
                        unique_values.insert(value.clone());

                        // 如果是数字类型，收集数值
                        if column.column_type == "number" {
                            if let Some(num) = value.as_f64() {
                                numeric_values.push(num);
                            }
                        }
                    }
                } else {
                    null_count += 1;
                }
            }

            let min_value = if numeric_values.is_empty() {
                None
            } else {
                numeric_values
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .map(|v| serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap()))
            };

            let max_value = if numeric_values.is_empty() {
                None
            } else {
                numeric_values
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .map(|v| serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap()))
            };

            column_stats.insert(
                column.name.clone(),
                ColumnStats {
                    non_null_count,
                    null_count,
                    unique_count: unique_values.len(),
                    min_value,
                    max_value,
                },
            );
        }

        Some(TableStatsResponse {
            table_id: table_id.to_string(),
            total_rows: rows.len(),
            column_stats,
        })
    }
}

impl Default for TableManager {
    fn default() -> Self {
        Self::new()
    }
}

/// GET 端点：获取表格列表
#[get("/api/table/list")]
pub fn list_tables(table_manager: &State<TableManager>) -> Json<TableListResponse> {
    Json(table_manager.list_tables())
}

/// GET 端点：获取表格信息
#[get("/api/table/<table_id>")]
pub fn get_table(
    table_id: &str,
    table_manager: &State<TableManager>,
) -> Result<Json<TableInfo>, Json<String>> {
    table_manager
        .get_table(table_id)
        .map(Json)
        .ok_or_else(|| Json("Table not found".to_string()))
}

/// GET 端点：获取表格数据
#[get("/api/table/<table_id>/data?<page>&<page_size>")]
pub fn get_table_data(
    table_id: &str,
    page: Option<usize>,
    page_size: Option<usize>,
    table_manager: &State<TableManager>,
) -> Result<Json<TableDataResponse>, Json<String>> {
    table_manager
        .get_table_data(table_id, page, page_size)
        .map(Json)
        .ok_or_else(|| Json("Table not found".to_string()))
}

/// GET 端点：获取表格统计信息
#[get("/api/table/<table_id>/stats")]
pub fn get_table_stats(
    table_id: &str,
    table_manager: &State<TableManager>,
) -> Result<Json<TableStatsResponse>, Json<String>> {
    table_manager
        .get_table_stats(table_id)
        .map(Json)
        .ok_or_else(|| Json("Table not found".to_string()))
}

/// POST 端点：创建表格
#[post("/api/table/create", data = "<request>")]
pub fn create_table(
    request: Json<CreateTableRequest>,
    table_manager: &State<TableManager>,
) -> Result<Json<TableInfo>, Json<String>> {
    table_manager
        .create_table(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// PUT 端点：更新表格
#[put("/api/table/<table_id>", data = "<request>")]
pub fn update_table(
    table_id: &str,
    request: Json<UpdateTableRequest>,
    table_manager: &State<TableManager>,
) -> Result<Json<TableInfo>, Json<String>> {
    table_manager
        .update_table(table_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// DELETE 端点：删除表格
#[delete("/api/table/<table_id>")]
pub fn delete_table(
    table_id: &str,
    table_manager: &State<TableManager>,
) -> Result<Json<String>, Json<String>> {
    table_manager
        .delete_table(table_id)
        .map(|_| Json("Table deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// POST 端点：添加行数据
#[post("/api/table/<table_id>/row", data = "<request>")]
pub fn add_row(
    table_id: &str,
    request: Json<CreateRowRequest>,
    table_manager: &State<TableManager>,
) -> Result<Json<TableRow>, Json<String>> {
    table_manager
        .add_row(table_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// PUT 端点：更新行数据
#[put("/api/table/<table_id>/row/<row_id>", data = "<request>")]
pub fn update_row(
    table_id: &str,
    row_id: &str,
    request: Json<UpdateRowRequest>,
    table_manager: &State<TableManager>,
) -> Result<Json<TableRow>, Json<String>> {
    table_manager
        .update_row(table_id, row_id, request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// DELETE 端点：删除行数据
#[delete("/api/table/<table_id>/row/<row_id>")]
pub fn delete_row(
    table_id: &str,
    row_id: &str,
    table_manager: &State<TableManager>,
) -> Result<Json<String>, Json<String>> {
    table_manager
        .delete_row(table_id, row_id)
        .map(|_| Json("Row deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// GET 端点：获取示例表格
#[get("/api/table/example")]
pub fn table_example(table_manager: &State<TableManager>) -> Json<TableInfo> {
    // 创建示例表格
    let request = CreateTableRequest {
        name: "用户表".to_string(),
        description: Some("用户信息表".to_string()),
        columns: vec![
            ColumnDefinition {
                name: "name".to_string(),
                column_type: "string".to_string(),
                required: true,
                default_value: None,
            },
            ColumnDefinition {
                name: "age".to_string(),
                column_type: "number".to_string(),
                required: false,
                default_value: Some(serde_json::Value::Number(serde_json::Number::from(0))),
            },
            ColumnDefinition {
                name: "email".to_string(),
                column_type: "string".to_string(),
                required: true,
                default_value: None,
            },
        ],
    };

    let table = table_manager.create_table(request).unwrap();

    // 添加示例数据
    let _ = table_manager.add_row(
        &table.id,
        CreateRowRequest {
            data: {
                let mut m = HashMap::new();
                m.insert(
                    "name".to_string(),
                    serde_json::Value::String("张三".to_string()),
                );
                m.insert(
                    "age".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(25)),
                );
                m.insert(
                    "email".to_string(),
                    serde_json::Value::String("zhangsan@example.com".to_string()),
                );
                m
            },
        },
    );

    let _ = table_manager.add_row(
        &table.id,
        CreateRowRequest {
            data: {
                let mut m = HashMap::new();
                m.insert(
                    "name".to_string(),
                    serde_json::Value::String("李四".to_string()),
                );
                m.insert(
                    "age".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(30)),
                );
                m.insert(
                    "email".to_string(),
                    serde_json::Value::String("lisi@example.com".to_string()),
                );
                m
            },
        },
    );

    Json(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table() {
        let manager = TableManager::new();
        let request = CreateTableRequest {
            name: "Test Table".to_string(),
            description: Some("Test Description".to_string()),
            columns: vec![ColumnDefinition {
                name: "col1".to_string(),
                column_type: "string".to_string(),
                required: true,
                default_value: None,
            }],
        };

        let table = manager.create_table(request).unwrap();
        assert_eq!(table.name, "Test Table");
        assert_eq!(table.row_count, 0);
    }

    #[test]
    fn test_add_row() {
        let manager = TableManager::new();
        let request = CreateTableRequest {
            name: "Test Table".to_string(),
            description: None,
            columns: vec![ColumnDefinition {
                name: "col1".to_string(),
                column_type: "string".to_string(),
                required: true,
                default_value: None,
            }],
        };

        let table = manager.create_table(request).unwrap();

        let row_request = CreateRowRequest {
            data: {
                let mut m = HashMap::new();
                m.insert(
                    "col1".to_string(),
                    serde_json::Value::String("value1".to_string()),
                );
                m
            },
        };

        let row = manager.add_row(&table.id, row_request).unwrap();
        assert_eq!(row.data.get("col1").unwrap().as_str().unwrap(), "value1");

        let updated_table = manager.get_table(&table.id).unwrap();
        assert_eq!(updated_table.row_count, 1);
    }

    #[test]
    fn test_update_row() {
        let manager = TableManager::new();
        let request = CreateTableRequest {
            name: "Test Table".to_string(),
            description: None,
            columns: vec![ColumnDefinition {
                name: "col1".to_string(),
                column_type: "string".to_string(),
                required: true,
                default_value: None,
            }],
        };

        let table = manager.create_table(request).unwrap();

        let row_request = CreateRowRequest {
            data: {
                let mut m = HashMap::new();
                m.insert(
                    "col1".to_string(),
                    serde_json::Value::String("value1".to_string()),
                );
                m
            },
        };

        let row = manager.add_row(&table.id, row_request).unwrap();

        let update_request = UpdateRowRequest {
            data: {
                let mut m = HashMap::new();
                m.insert(
                    "col1".to_string(),
                    serde_json::Value::String("updated_value".to_string()),
                );
                m
            },
        };

        let updated_row = manager
            .update_row(&table.id, &row.id, update_request)
            .unwrap();
        assert_eq!(
            updated_row.data.get("col1").unwrap().as_str().unwrap(),
            "updated_value"
        );
    }

    #[test]
    fn test_delete_row() {
        let manager = TableManager::new();
        let request = CreateTableRequest {
            name: "Test Table".to_string(),
            description: None,
            columns: vec![ColumnDefinition {
                name: "col1".to_string(),
                column_type: "string".to_string(),
                required: true,
                default_value: None,
            }],
        };

        let table = manager.create_table(request).unwrap();

        let row_request = CreateRowRequest {
            data: {
                let mut m = HashMap::new();
                m.insert(
                    "col1".to_string(),
                    serde_json::Value::String("value1".to_string()),
                );
                m
            },
        };

        let row = manager.add_row(&table.id, row_request).unwrap();
        manager.delete_row(&table.id, &row.id).unwrap();

        let updated_table = manager.get_table(&table.id).unwrap();
        assert_eq!(updated_table.row_count, 0);
    }
}
