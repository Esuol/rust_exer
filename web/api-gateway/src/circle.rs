use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 圆形计算错误
#[derive(Debug, Clone)]
pub enum CircleError {
    InvalidRadius,
    InvalidAngle,
}

impl fmt::Display for CircleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircleError::InvalidRadius => write!(f, "半径必须大于 0"),
            CircleError::InvalidAngle => write!(f, "角度必须在 0 到 360 之间"),
        }
    }
}

impl std::error::Error for CircleError {}

/// 圆形计算请求
#[derive(Debug, Clone, Deserialize)]
pub struct CircleRequest {
    /// 半径
    pub radius: f64,
    /// 角度（可选，用于弧长/扇形面积）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle_degrees: Option<f64>,
}

/// 圆形计算响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleResponse {
    /// 半径
    pub radius: f64,
    /// 直径
    pub diameter: f64,
    /// 周长
    pub circumference: f64,
    /// 面积
    pub area: f64,
    /// 弧长（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arc_length: Option<f64>,
    /// 扇形面积（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_area: Option<f64>,
}

/// 校验半径
fn validate_radius(radius: f64) -> Result<(), CircleError> {
    if radius <= 0.0 {
        return Err(CircleError::InvalidRadius);
    }
    Ok(())
}

/// 校验角度
fn validate_angle(angle: f64) -> Result<(), CircleError> {
    if !(0.0..=360.0).contains(&angle) {
        return Err(CircleError::InvalidAngle);
    }
    Ok(())
}

/// 四舍五入到指定小数位
fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// 圆形基础计算
pub fn calculate_circle(radius: f64) -> Result<CircleResponse, CircleError> {
    validate_radius(radius)?;

    let diameter = radius * 2.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let area = std::f64::consts::PI * radius * radius;

    Ok(CircleResponse {
        radius: round_to_decimal(radius, 4),
        diameter: round_to_decimal(diameter, 4),
        circumference: round_to_decimal(circumference, 4),
        area: round_to_decimal(area, 4),
        arc_length: None,
        sector_area: None,
    })
}

/// 圆形扩展计算（含弧长/扇形面积）
pub fn calculate_circle_with_angle(
    radius: f64,
    angle_degrees: f64,
) -> Result<CircleResponse, CircleError> {
    validate_radius(radius)?;
    validate_angle(angle_degrees)?;

    let mut base = calculate_circle(radius)?;
    let ratio = angle_degrees / 360.0;
    let arc_length = base.circumference * ratio;
    let sector_area = base.area * ratio;

    base.arc_length = Some(round_to_decimal(arc_length, 4));
    base.sector_area = Some(round_to_decimal(sector_area, 4));
    Ok(base)
}

/// GET 端点：基础圆形计算
#[get("/api/circle/calculate?<radius>")]
pub fn circle_calculate_get(radius: Option<f64>) -> Result<Json<CircleResponse>, Json<String>> {
    let radius = radius.ok_or_else(|| Json("缺少 radius 参数".to_string()))?;
    let result = calculate_circle(radius).map_err(|e| Json(e.to_string()))?;
    Ok(Json(result))
}

/// GET 端点：带角度的圆形计算
#[get("/api/circle/calculate-with-angle?<radius>&<angle>")]
pub fn circle_calculate_with_angle_get(
    radius: Option<f64>,
    angle: Option<f64>,
) -> Result<Json<CircleResponse>, Json<String>> {
    let radius = radius.ok_or_else(|| Json("缺少 radius 参数".to_string()))?;
    let angle = angle.ok_or_else(|| Json("缺少 angle 参数".to_string()))?;
    let result = calculate_circle_with_angle(radius, angle).map_err(|e| Json(e.to_string()))?;
    Ok(Json(result))
}

/// POST 端点：圆形计算
#[post("/api/circle/calculate", data = "<request>")]
pub fn circle_calculate_post(
    request: Json<CircleRequest>,
) -> Result<Json<CircleResponse>, Json<String>> {
    let req = request.into_inner();
    let result = if let Some(angle) = req.angle_degrees {
        calculate_circle_with_angle(req.radius, angle)
    } else {
        calculate_circle(req.radius)
    }
    .map_err(|e| Json(e.to_string()))?;

    Ok(Json(result))
}

/// GET 端点：圆形计算示例
#[get("/api/circle/example")]
pub fn circle_example() -> Json<CircleResponse> {
    let result = calculate_circle_with_angle(10.0, 60.0).expect("示例数据应该有效");
    Json(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_circle() {
        let res = calculate_circle(10.0).unwrap();
        assert_eq!(res.radius, 10.0);
        assert_eq!(res.diameter, 20.0);
        assert!(res.area > 314.0 && res.area < 314.3);
    }

    #[test]
    fn test_calculate_circle_with_angle() {
        let res = calculate_circle_with_angle(10.0, 90.0).unwrap();
        assert!(res.arc_length.is_some());
        assert!(res.sector_area.is_some());
    }

    #[test]
    fn test_invalid_radius() {
        assert!(calculate_circle(0.0).is_err());
        assert!(calculate_circle(-1.0).is_err());
    }

    #[test]
    fn test_invalid_angle() {
        assert!(calculate_circle_with_angle(10.0, -1.0).is_err());
        assert!(calculate_circle_with_angle(10.0, 361.0).is_err());
    }
}
