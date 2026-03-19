use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};

/// 跳过/截取请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipRequest {
    /// 原始列表
    pub items: Vec<serde_json::Value>,
    /// 跳过前 N 个元素
    pub skip: usize,
    /// 最多返回多少个元素（不填则返回直到结束）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// 跳过/截取响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipResponse {
    /// 原始总数
    pub total: usize,
    /// 实际跳过数量
    pub skipped: usize,
    /// 返回数量
    pub returned: usize,
    /// 返回的列表
    pub items: Vec<serde_json::Value>,
}

/// 将 `items` 的前 `skip` 个元素丢弃，再应用 `limit`（若有）
fn apply_skip(req: SkipRequest) -> SkipResponse {
    let total = req.items.len();
    let start = req.skip.min(total);
    let mut out: Vec<serde_json::Value> = req.items[start..].to_vec();

    if let Some(limit) = req.limit {
        out.truncate(limit);
    }

    SkipResponse {
        total,
        skipped: start,
        returned: out.len(),
        items: out,
    }
}

/// POST：应用 skip/limit
#[post("/api/skip/apply", data = "<request>")]
pub fn skip_apply(request: Json<SkipRequest>) -> Json<SkipResponse> {
    Json(apply_skip(request.into_inner()))
}

/// GET：示例（返回应用后的列表）
#[get("/api/skip/example")]
pub fn skip_example() -> Json<SkipResponse> {
    let req = SkipRequest {
        items: vec![
            serde_json::json!(1),
            serde_json::json!(2),
            serde_json::json!(3),
            serde_json::json!(4),
            serde_json::json!(5),
        ],
        skip: 2,
        limit: Some(2),
    };
    Json(apply_skip(req))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_skip_no_limit() {
        let req = SkipRequest {
            items: vec![
                serde_json::json!(1),
                serde_json::json!(2),
                serde_json::json!(3),
            ],
            skip: 1,
            limit: None,
        };
        let res = apply_skip(req);
        assert_eq!(res.total, 3);
        assert_eq!(res.skipped, 1);
        assert_eq!(res.returned, 2);
    }

    #[test]
    fn test_apply_skip_with_limit() {
        let req = SkipRequest {
            items: vec![
                serde_json::json!("a"),
                serde_json::json!("b"),
                serde_json::json!("c"),
                serde_json::json!("d"),
            ],
            skip: 1,
            limit: Some(2),
        };
        let res = apply_skip(req);
        assert_eq!(res.skipped, 1);
        assert_eq!(res.returned, 2);
    }

    #[test]
    fn test_apply_skip_overflow() {
        let req = SkipRequest {
            items: vec![serde_json::json!(1)],
            skip: 10,
            limit: Some(5),
        };
        let res = apply_skip(req);
        assert_eq!(res.skipped, 1);
        assert_eq!(res.returned, 0);
    }
}
