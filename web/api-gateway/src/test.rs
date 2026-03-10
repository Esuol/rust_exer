use chrono::Utc;
use rocket::{delete, get, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 测试用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// 测试用例ID
    pub id: String,
    /// 测试用例名称
    pub name: String,
    /// 测试用例描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// HTTP 方法
    pub method: String,
    /// 请求URL
    pub url: String,
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// 请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
    /// 预期状态码
    pub expected_status: u16,
    /// 预期响应体（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_response: Option<serde_json::Value>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// 测试结果ID
    pub id: String,
    /// 测试用例ID
    pub test_case_id: String,
    /// 测试状态（passed, failed, error）
    pub status: String,
    /// 实际状态码
    pub actual_status: Option<u16>,
    /// 实际响应体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_response: Option<serde_json::Value>,
    /// 响应时间（毫秒）
    pub response_time_ms: f64,
    /// 错误信息（如果有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// 执行时间
    pub executed_at: String,
}

/// 测试套件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// 测试套件ID
    pub id: String,
    /// 测试套件名称
    pub name: String,
    /// 测试套件描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 测试用例ID列表
    pub test_case_ids: Vec<String>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 测试套件执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteResult {
    /// 测试套件ID
    pub suite_id: String,
    /// 测试结果列表
    pub results: Vec<TestResult>,
    /// 总测试数
    pub total: usize,
    /// 通过数
    pub passed: usize,
    /// 失败数
    pub failed: usize,
    /// 错误数
    pub error: usize,
    /// 总执行时间（毫秒）
    pub total_time_ms: f64,
    /// 执行时间
    pub executed_at: String,
}

/// 创建测试用例请求
#[derive(Debug, Deserialize)]
pub struct CreateTestCaseRequest {
    /// 测试用例名称
    pub name: String,
    /// 测试用例描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// HTTP 方法
    pub method: String,
    /// 请求URL
    pub url: String,
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// 请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
    /// 预期状态码
    pub expected_status: u16,
    /// 预期响应体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_response: Option<serde_json::Value>,
}

/// 创建测试套件请求
#[derive(Debug, Deserialize)]
pub struct CreateTestSuiteRequest {
    /// 测试套件名称
    pub name: String,
    /// 测试套件描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 测试用例ID列表
    pub test_case_ids: Vec<String>,
}

/// 测试用例列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TestCaseListResponse {
    /// 测试用例列表
    pub test_cases: Vec<TestCase>,
    /// 总数
    pub total: usize,
}

/// 测试套件列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteListResponse {
    /// 测试套件列表
    pub test_suites: Vec<TestSuite>,
    /// 总数
    pub total: usize,
}

/// 测试管理器
#[derive(Debug, Clone)]
pub struct TestManager {
    test_cases: Arc<RwLock<HashMap<String, TestCase>>>,
    test_suites: Arc<RwLock<HashMap<String, TestSuite>>>,
    test_results: Arc<RwLock<Vec<TestResult>>>,
}

impl TestManager {
    /// 创建新的测试管理器
    pub fn new() -> Self {
        Self {
            test_cases: Arc::new(RwLock::new(HashMap::new())),
            test_suites: Arc::new(RwLock::new(HashMap::new())),
            test_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 创建测试用例
    pub fn create_test_case(&self, request: CreateTestCaseRequest) -> Result<TestCase, String> {
        let test_case_id = format!("test_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        let test_case = TestCase {
            id: test_case_id.clone(),
            name: request.name,
            description: request.description,
            method: request.method,
            url: request.url,
            headers: request.headers,
            body: request.body,
            expected_status: request.expected_status,
            expected_response: request.expected_response,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut test_cases = self.test_cases.write().unwrap();
        test_cases.insert(test_case_id, test_case.clone());

        Ok(test_case)
    }

    /// 获取测试用例列表
    pub fn list_test_cases(&self) -> TestCaseListResponse {
        let test_cases = self.test_cases.read().unwrap();
        let test_cases_vec: Vec<TestCase> = test_cases.values().cloned().collect();

        TestCaseListResponse {
            total: test_cases_vec.len(),
            test_cases: test_cases_vec,
        }
    }

    /// 获取测试用例
    pub fn get_test_case(&self, test_case_id: &str) -> Option<TestCase> {
        let test_cases = self.test_cases.read().unwrap();
        test_cases.get(test_case_id).cloned()
    }

    /// 删除测试用例
    pub fn delete_test_case(&self, test_case_id: &str) -> Result<(), String> {
        let mut test_cases = self.test_cases.write().unwrap();
        test_cases
            .remove(test_case_id)
            .ok_or_else(|| "Test case not found".to_string())?;
        Ok(())
    }

    /// 创建测试套件
    pub fn create_test_suite(&self, request: CreateTestSuiteRequest) -> Result<TestSuite, String> {
        let suite_id = format!("suite_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        // 验证测试用例是否存在
        let test_cases = self.test_cases.read().unwrap();
        for test_case_id in &request.test_case_ids {
            if !test_cases.contains_key(test_case_id) {
                return Err(format!("Test case {} not found", test_case_id));
            }
        }
        drop(test_cases);

        let test_suite = TestSuite {
            id: suite_id.clone(),
            name: request.name,
            description: request.description,
            test_case_ids: request.test_case_ids,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut test_suites = self.test_suites.write().unwrap();
        test_suites.insert(suite_id, test_suite.clone());

        Ok(test_suite)
    }

    /// 获取测试套件列表
    pub fn list_test_suites(&self) -> TestSuiteListResponse {
        let test_suites = self.test_suites.read().unwrap();
        let test_suites_vec: Vec<TestSuite> = test_suites.values().cloned().collect();

        TestSuiteListResponse {
            total: test_suites_vec.len(),
            test_suites: test_suites_vec,
        }
    }

    /// 执行测试用例
    pub async fn run_test_case(&self, test_case_id: &str) -> Result<TestResult, String> {
        let test_case = self
            .get_test_case(test_case_id)
            .ok_or_else(|| "Test case not found".to_string())?;

        let start_time = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let mut request_builder = match test_case.method.as_str() {
            "GET" => client.get(&test_case.url),
            "POST" => client.post(&test_case.url),
            "PUT" => client.put(&test_case.url),
            "DELETE" => client.delete(&test_case.url),
            "PATCH" => client.patch(&test_case.url),
            _ => return Err(format!("Unsupported HTTP method: {}", test_case.method)),
        };

        // 添加请求头
        if let Some(headers) = &test_case.headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // 添加请求体
        if let Some(body) = &test_case.body {
            request_builder = request_builder.json(body);
        }

        let result = request_builder.send().await;
        let response_time = start_time.elapsed().as_millis() as f64;

        let (status, actual_response, error_message) = match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let response_text = response.text().await.ok();
                let json_response = response_text
                    .as_ref()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok());

                (Some(status), json_response, None)
            }
            Err(e) => (None, None, Some(e.to_string())),
        };

        let test_status = if error_message.is_some() {
            "error"
        } else if let Some(actual_status) = status {
            if actual_status == test_case.expected_status {
                "passed"
            } else {
                "failed"
            }
        } else {
            "error"
        };

        let result_id = format!("result_{}", Utc::now().timestamp_millis());
        let test_result = TestResult {
            id: result_id.clone(),
            test_case_id: test_case_id.to_string(),
            status: test_status.to_string(),
            actual_status: status,
            actual_response,
            response_time_ms: response_time,
            error_message,
            executed_at: Utc::now().to_rfc3339(),
        };

        // 保存测试结果
        let mut test_results = self.test_results.write().unwrap();
        test_results.push(test_result.clone());

        Ok(test_result)
    }

    /// 执行测试套件
    pub async fn run_test_suite(&self, suite_id: &str) -> Result<TestSuiteResult, String> {
        let test_suite = {
            let test_suites = self.test_suites.read().unwrap();
            test_suites
                .get(suite_id)
                .ok_or_else(|| "Test suite not found".to_string())?
                .clone()
        };

        let start_time = Instant::now();
        let mut results = Vec::new();

        for test_case_id in &test_suite.test_case_ids {
            match self.run_test_case(test_case_id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(TestResult {
                        id: format!("result_{}", Utc::now().timestamp_millis()),
                        test_case_id: test_case_id.clone(),
                        status: "error".to_string(),
                        actual_status: None,
                        actual_response: None,
                        response_time_ms: 0.0,
                        error_message: Some(e),
                        executed_at: Utc::now().to_rfc3339(),
                    });
                }
            }
        }

        let total_time = start_time.elapsed().as_millis() as f64;
        let passed = results.iter().filter(|r| r.status == "passed").count();
        let failed = results.iter().filter(|r| r.status == "failed").count();
        let error = results.iter().filter(|r| r.status == "error").count();

        Ok(TestSuiteResult {
            suite_id: suite_id.to_string(),
            results,
            total: test_suite.test_case_ids.len(),
            passed,
            failed,
            error,
            total_time_ms: total_time,
            executed_at: Utc::now().to_rfc3339(),
        })
    }

    /// 获取测试结果历史
    pub fn get_test_results(&self, limit: Option<usize>) -> Vec<TestResult> {
        let test_results = self.test_results.read().unwrap();
        let mut results: Vec<TestResult> = test_results.iter().cloned().collect();
        results.reverse(); // 最新的在前

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results
    }
}

impl Default for TestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// POST 端点：创建测试用例
#[post("/api/test/case", data = "<request>")]
pub fn create_test_case(
    request: Json<CreateTestCaseRequest>,
    test_manager: &State<TestManager>,
) -> Result<Json<TestCase>, Json<String>> {
    test_manager
        .create_test_case(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取测试用例列表
#[get("/api/test/case/list")]
pub fn list_test_cases(test_manager: &State<TestManager>) -> Json<TestCaseListResponse> {
    Json(test_manager.list_test_cases())
}

/// GET 端点：获取测试用例
#[get("/api/test/case/<test_case_id>")]
pub fn get_test_case(
    test_case_id: &str,
    test_manager: &State<TestManager>,
) -> Result<Json<TestCase>, Json<String>> {
    test_manager
        .get_test_case(test_case_id)
        .map(Json)
        .ok_or_else(|| Json("Test case not found".to_string()))
}

/// DELETE 端点：删除测试用例
#[delete("/api/test/case/<test_case_id>")]
pub fn delete_test_case(
    test_case_id: &str,
    test_manager: &State<TestManager>,
) -> Result<Json<String>, Json<String>> {
    test_manager
        .delete_test_case(test_case_id)
        .map(|_| Json("Test case deleted successfully".to_string()))
        .map_err(|e| Json(e))
}

/// POST 端点：执行测试用例
#[post("/api/test/case/<test_case_id>/run")]
pub async fn run_test_case(
    test_case_id: &str,
    test_manager: &State<TestManager>,
) -> Result<Json<TestResult>, Json<String>> {
    test_manager
        .run_test_case(test_case_id)
        .await
        .map(Json)
        .map_err(|e| Json(e))
}

/// POST 端点：创建测试套件
#[post("/api/test/suite", data = "<request>")]
pub fn create_test_suite(
    request: Json<CreateTestSuiteRequest>,
    test_manager: &State<TestManager>,
) -> Result<Json<TestSuite>, Json<String>> {
    test_manager
        .create_test_suite(request.into_inner())
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取测试套件列表
#[get("/api/test/suite/list")]
pub fn list_test_suites(test_manager: &State<TestManager>) -> Json<TestSuiteListResponse> {
    Json(test_manager.list_test_suites())
}

/// POST 端点：执行测试套件
#[post("/api/test/suite/<suite_id>/run")]
pub async fn run_test_suite(
    suite_id: &str,
    test_manager: &State<TestManager>,
) -> Result<Json<TestSuiteResult>, Json<String>> {
    test_manager
        .run_test_suite(suite_id)
        .await
        .map(Json)
        .map_err(|e| Json(e))
}

/// GET 端点：获取测试结果历史
#[get("/api/test/results?<limit>")]
pub fn get_test_results(
    limit: Option<usize>,
    test_manager: &State<TestManager>,
) -> Json<Vec<TestResult>> {
    Json(test_manager.get_test_results(limit))
}

/// GET 端点：获取测试示例
#[get("/api/test/example")]
pub fn test_example(test_manager: &State<TestManager>) -> Json<TestCase> {
    // 创建示例测试用例
    let request = CreateTestCaseRequest {
        name: "健康检查测试".to_string(),
        description: Some("测试健康检查端点".to_string()),
        method: "GET".to_string(),
        url: "http://localhost:8000/health".to_string(),
        headers: None,
        body: None,
        expected_status: 200,
        expected_response: None,
    };

    let test_case = test_manager.create_test_case(request).unwrap();
    Json(test_case)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_case() {
        let manager = TestManager::new();
        let request = CreateTestCaseRequest {
            name: "Test Case".to_string(),
            description: Some("Test Description".to_string()),
            method: "GET".to_string(),
            url: "http://example.com".to_string(),
            headers: None,
            body: None,
            expected_status: 200,
            expected_response: None,
        };

        let test_case = manager.create_test_case(request).unwrap();
        assert_eq!(test_case.name, "Test Case");
        assert_eq!(test_case.method, "GET");
        assert_eq!(test_case.expected_status, 200);
    }

    #[test]
    fn test_list_test_cases() {
        let manager = TestManager::new();
        let request = CreateTestCaseRequest {
            name: "Test Case".to_string(),
            description: None,
            method: "GET".to_string(),
            url: "http://example.com".to_string(),
            headers: None,
            body: None,
            expected_status: 200,
            expected_response: None,
        };

        manager.create_test_case(request).unwrap();
        let list = manager.list_test_cases();
        assert_eq!(list.total, 1);
    }

    #[test]
    fn test_create_test_suite() {
        let manager = TestManager::new();
        let request = CreateTestCaseRequest {
            name: "Test Case".to_string(),
            description: None,
            method: "GET".to_string(),
            url: "http://example.com".to_string(),
            headers: None,
            body: None,
            expected_status: 200,
            expected_response: None,
        };

        let test_case = manager.create_test_case(request).unwrap();

        let suite_request = CreateTestSuiteRequest {
            name: "Test Suite".to_string(),
            description: None,
            test_case_ids: vec![test_case.id],
        };

        let suite = manager.create_test_suite(suite_request).unwrap();
        assert_eq!(suite.name, "Test Suite");
        assert_eq!(suite.test_case_ids.len(), 1);
    }
}
