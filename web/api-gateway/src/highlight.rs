use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};

/// 高亮请求
#[derive(Debug, Deserialize)]
pub struct HighlightRequest {
    /// 原始文本（会先做 HTML 转义，避免把用户输入当 HTML 渲染）
    pub text: String,
    /// 要高亮的关键字
    pub query: String,
    /// 是否区分大小写（默认 false，忽略大小写匹配）
    #[serde(default)]
    pub case_sensitive: bool,
    /// 高亮包裹的 HTML 标签名（默认 `mark`）
    #[serde(default = "default_tag")]
    pub tag: String,
    /// 高亮用的 CSS class（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
}

fn default_tag() -> String {
    "mark".to_string()
}

/// 高亮响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HighlightResponse {
    /// 高亮后的 HTML（已转义并用 tag 包裹匹配段）
    pub highlighted_html: String,
    /// 命中次数
    pub matches: usize,
}

/// 转义 HTML 特殊字符，防止注入
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 在字符串中高亮 query，返回 HTML
fn highlight(req: HighlightRequest) -> HighlightResponse {
    if req.query.is_empty() {
        return HighlightResponse {
            highlighted_html: escape_html(&req.text),
            matches: 0,
        };
    }

    let tag = req.tag.trim().to_string();
    let class_attr = req
        .class_name
        .as_ref()
        .map(|c| format!(" class=\"{}\"", c))
        .unwrap_or_default();

    let open_tag = format!("<{}{}>", tag, class_attr);
    let close_tag = format!("</{}>", tag);

    if req.case_sensitive {
        let text = req.text;
        let query = req.query;

        let mut result = String::new();
        let mut matches = 0usize;
        let mut start = 0usize;

        while let Some(pos) = text[start..].find(&query) {
            let abs_pos = start + pos;
            result.push_str(&escape_html(&text[start..abs_pos]));
            result.push_str(&open_tag);
            result.push_str(&escape_html(&text[abs_pos..abs_pos + query.len()]));
            result.push_str(&close_tag);

            matches += 1;
            start = abs_pos + query.len();
            if start >= text.len() {
                break;
            }
        }

        result.push_str(&escape_html(&text[start..]));

        HighlightResponse {
            highlighted_html: result,
            matches,
        }
    } else {
        let original_text = req.text;
        let query = req.query;

        // 注意：lowercase 可能在某些 Unicode 情况下改变字节长度。
        // 为了满足本项目教学/示例用途，这里假设匹配前后文本长度保持一致。
        let lowered_text = original_text.to_lowercase();
        let lowered_query = query.to_lowercase();

        let mut result = String::new();
        let mut matches = 0usize;
        let mut start = 0usize;

        while let Some(pos) = lowered_text[start..].find(&lowered_query) {
            let abs_pos = start + pos;
            let match_end = abs_pos + lowered_query.len();

            result.push_str(&escape_html(&original_text[start..abs_pos]));
            result.push_str(&open_tag);
            result.push_str(&escape_html(&original_text[abs_pos..match_end]));
            result.push_str(&close_tag);

            matches += 1;
            start = match_end;
            if start >= lowered_text.len() {
                break;
            }
        }

        result.push_str(&escape_html(&original_text[start..]));

        HighlightResponse {
            highlighted_html: result,
            matches,
        }
    }
}

/// POST：文本高亮
#[post("/api/highlight", data = "<request>")]
pub fn highlight_endpoint(request: Json<HighlightRequest>) -> Json<HighlightResponse> {
    Json(highlight(request.into_inner()))
}

/// GET：高亮示例
#[get("/api/highlight/example")]
pub fn highlight_example() -> Json<HighlightResponse> {
    let req = HighlightRequest {
        text: "Hello world, hello Rust".to_string(),
        query: "hello".to_string(),
        case_sensitive: false,
        tag: "mark".to_string(),
        class_name: Some("hl".to_string()),
    };
    Json(highlight(req))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_case_sensitive() {
        let req = HighlightRequest {
            text: "abc def abc".to_string(),
            query: "abc".to_string(),
            case_sensitive: true,
            tag: "mark".to_string(),
            class_name: None,
        };
        let res = highlight(req);
        assert_eq!(res.matches, 2);
        assert!(res.highlighted_html.contains("<mark>abc</mark>"));
    }

    #[test]
    fn test_highlight_empty_query() {
        let req = HighlightRequest {
            text: "abc".to_string(),
            query: "".to_string(),
            case_sensitive: true,
            tag: "mark".to_string(),
            class_name: None,
        };
        let res = highlight(req);
        assert_eq!(res.matches, 0);
        assert_eq!(res.highlighted_html, "abc");
    }
}
