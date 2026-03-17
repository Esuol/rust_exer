use chrono::Utc;
use rocket::{delete, get, post, put, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 发货/物流状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipmentStatus {
    /// 待发货
    Pending,
    /// 已发货
    Shipped,
    /// 运输中
    InTransit,
    /// 派送中
    OutForDelivery,
    /// 已签收
    Delivered,
    /// 已取消
    Cancelled,
    /// 异常
    Exception,
}

impl ShipmentStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ShipmentStatus::Pending => "pending",
            ShipmentStatus::Shipped => "shipped",
            ShipmentStatus::InTransit => "in_transit",
            ShipmentStatus::OutForDelivery => "out_for_delivery",
            ShipmentStatus::Delivered => "delivered",
            ShipmentStatus::Cancelled => "cancelled",
            ShipmentStatus::Exception => "exception",
        }
    }
}

/// 物流轨迹事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    /// 事件时间（RFC3339）
    pub timestamp: String,
    /// 当前位置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// 事件描述
    pub message: String,
    /// 事件代码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// 收件/寄件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// 姓名
    pub name: String,
    /// 电话（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// 地址（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// 发货单（Shipment）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    /// 发货单ID
    pub id: String,
    /// 外部订单号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// 承运商（如 SF/UPS/FedEx）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier: Option<String>,
    /// 运单号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,
    /// 当前状态
    pub status: ShipmentStatus,
    /// 寄件人（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<ContactInfo>,
    /// 收件人（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<ContactInfo>,
    /// 物流轨迹
    pub events: Vec<TrackingEvent>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// 创建时间（RFC3339）
    pub created_at: String,
    /// 更新时间（RFC3339）
    pub updated_at: String,
}

/// 创建发货单请求
#[derive(Debug, Deserialize)]
pub struct CreateShipmentRequest {
    /// 外部订单号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// 承运商（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier: Option<String>,
    /// 运单号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,
    /// 寄件人（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<ContactInfo>,
    /// 收件人（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<ContactInfo>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 更新发货单请求
#[derive(Debug, Deserialize)]
pub struct UpdateShipmentRequest {
    /// 承运商（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier: Option<String>,
    /// 运单号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,
    /// 寄件人（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<ContactInfo>,
    /// 收件人（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<ContactInfo>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 变更状态请求
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    /// 新状态：pending/shipped/in_transit/out_for_delivery/delivered/cancelled/exception
    pub status: String,
    /// 备注（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 位置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// 追加轨迹事件请求
#[derive(Debug, Deserialize)]
pub struct AddEventRequest {
    /// 位置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// 描述
    pub message: String,
    /// 事件代码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// 发货单列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ShipmentListResponse {
    /// 发货单列表
    pub shipments: Vec<Shipment>,
    /// 总数
    pub total: usize,
    /// 按状态统计
    pub by_status: HashMap<String, usize>,
}

/// 发货单统计
#[derive(Debug, Serialize, Deserialize)]
pub struct ShipmentStatsResponse {
    /// 发货单ID
    pub shipment_id: String,
    /// 状态
    pub status: String,
    /// 轨迹事件数量
    pub event_count: usize,
    /// 最近一次轨迹（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_event: Option<TrackingEvent>,
}

fn parse_status(s: &str) -> Option<ShipmentStatus> {
    match s {
        "pending" => Some(ShipmentStatus::Pending),
        "shipped" => Some(ShipmentStatus::Shipped),
        "in_transit" => Some(ShipmentStatus::InTransit),
        "out_for_delivery" => Some(ShipmentStatus::OutForDelivery),
        "delivered" => Some(ShipmentStatus::Delivered),
        "cancelled" => Some(ShipmentStatus::Cancelled),
        "exception" => Some(ShipmentStatus::Exception),
        _ => None,
    }
}

/// 发货/物流管理器（内存版）
#[derive(Debug, Clone)]
pub struct ShipManager {
    shipments: Arc<RwLock<HashMap<String, Shipment>>>,
}

impl ShipManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            shipments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建发货单
    pub fn create_shipment(&self, request: CreateShipmentRequest) -> Shipment {
        let id = format!("ship_{}", Utc::now().timestamp_millis());
        let now = Utc::now().to_rfc3339();

        let mut events = Vec::new();
        events.push(TrackingEvent {
            timestamp: now.clone(),
            location: None,
            message: "创建发货单".to_string(),
            code: Some("CREATED".to_string()),
        });

        let shipment = Shipment {
            id: id.clone(),
            order_id: request.order_id,
            carrier: request.carrier,
            tracking_number: request.tracking_number,
            status: ShipmentStatus::Pending,
            sender: request.sender,
            receiver: request.receiver,
            events,
            metadata: request.metadata,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut shipments = self.shipments.write().unwrap();
        shipments.insert(id, shipment.clone());

        shipment
    }

    /// 列表查询（可按 status / order_id / tracking_number 过滤）
    pub fn list_shipments(
        &self,
        status: Option<&str>,
        order_id: Option<&str>,
        tracking_number: Option<&str>,
    ) -> ShipmentListResponse {
        let shipments = self.shipments.read().unwrap();
        let mut list: Vec<Shipment> = shipments.values().cloned().collect();

        if let Some(status) = status {
            if let Some(st) = parse_status(status) {
                list.retain(|s| s.status == st);
            }
        }
        if let Some(order_id) = order_id {
            list.retain(|s| s.order_id.as_deref() == Some(order_id));
        }
        if let Some(tracking_number) = tracking_number {
            list.retain(|s| s.tracking_number.as_deref() == Some(tracking_number));
        }

        let mut by_status: HashMap<String, usize> = HashMap::new();
        for s in shipments.values() {
            *by_status.entry(s.status.as_str().to_string()).or_insert(0) += 1;
        }

        ShipmentListResponse {
            total: list.len(),
            by_status,
            shipments: list,
        }
    }

    /// 获取发货单
    pub fn get_shipment(&self, shipment_id: &str) -> Option<Shipment> {
        let shipments = self.shipments.read().unwrap();
        shipments.get(shipment_id).cloned()
    }

    /// 更新发货单（不改状态）
    pub fn update_shipment(
        &self,
        shipment_id: &str,
        request: UpdateShipmentRequest,
    ) -> Result<Shipment, String> {
        let mut shipments = self.shipments.write().unwrap();
        let shipment = shipments
            .get_mut(shipment_id)
            .ok_or_else(|| "Shipment not found".to_string())?;

        if let Some(carrier) = request.carrier {
            shipment.carrier = Some(carrier);
        }
        if let Some(tracking_number) = request.tracking_number {
            shipment.tracking_number = Some(tracking_number);
        }
        if let Some(sender) = request.sender {
            shipment.sender = Some(sender);
        }
        if let Some(receiver) = request.receiver {
            shipment.receiver = Some(receiver);
        }
        if let Some(metadata) = request.metadata {
            shipment.metadata = Some(metadata);
        }

        shipment.updated_at = Utc::now().to_rfc3339();
        shipment.events.push(TrackingEvent {
            timestamp: shipment.updated_at.clone(),
            location: None,
            message: "更新发货单信息".to_string(),
            code: Some("UPDATED".to_string()),
        });

        Ok(shipment.clone())
    }

    /// 更新状态（会追加轨迹）
    pub fn update_status(
        &self,
        shipment_id: &str,
        request: UpdateStatusRequest,
    ) -> Result<Shipment, String> {
        let new_status = parse_status(&request.status)
            .ok_or_else(|| format!("Invalid status: {}", request.status))?;

        let mut shipments = self.shipments.write().unwrap();
        let shipment = shipments
            .get_mut(shipment_id)
            .ok_or_else(|| "Shipment not found".to_string())?;

        shipment.status = new_status;
        shipment.updated_at = Utc::now().to_rfc3339();
        shipment.events.push(TrackingEvent {
            timestamp: shipment.updated_at.clone(),
            location: request.location,
            message: request
                .message
                .unwrap_or_else(|| format!("状态变更为 {}", new_status.as_str())),
            code: Some("STATUS".to_string()),
        });

        Ok(shipment.clone())
    }

    /// 追加轨迹事件
    pub fn add_event(
        &self,
        shipment_id: &str,
        request: AddEventRequest,
    ) -> Result<Shipment, String> {
        let mut shipments = self.shipments.write().unwrap();
        let shipment = shipments
            .get_mut(shipment_id)
            .ok_or_else(|| "Shipment not found".to_string())?;

        shipment.updated_at = Utc::now().to_rfc3339();
        shipment.events.push(TrackingEvent {
            timestamp: shipment.updated_at.clone(),
            location: request.location,
            message: request.message,
            code: request.code,
        });

        Ok(shipment.clone())
    }

    /// 删除发货单
    pub fn delete_shipment(&self, shipment_id: &str) -> Result<(), String> {
        let mut shipments = self.shipments.write().unwrap();
        shipments
            .remove(shipment_id)
            .ok_or_else(|| "Shipment not found".to_string())?;
        Ok(())
    }

    /// 获取统计
    pub fn stats(&self, shipment_id: &str) -> Result<ShipmentStatsResponse, String> {
        let shipment = self
            .get_shipment(shipment_id)
            .ok_or_else(|| "Shipment not found".to_string())?;

        Ok(ShipmentStatsResponse {
            shipment_id: shipment.id.clone(),
            status: shipment.status.as_str().to_string(),
            event_count: shipment.events.len(),
            latest_event: shipment.events.last().cloned(),
        })
    }
}

impl Default for ShipManager {
    fn default() -> Self {
        Self::new()
    }
}

/// POST：创建发货单
#[post("/api/ship", data = "<request>")]
pub fn create_ship(
    request: Json<CreateShipmentRequest>,
    ship_manager: &State<ShipManager>,
) -> Json<Shipment> {
    Json(ship_manager.create_shipment(request.into_inner()))
}

/// GET：发货单列表
#[get("/api/ship/list?<status>&<order_id>&<tracking_number>")]
pub fn list_ships(
    status: Option<&str>,
    order_id: Option<&str>,
    tracking_number: Option<&str>,
    ship_manager: &State<ShipManager>,
) -> Json<ShipmentListResponse> {
    Json(ship_manager.list_shipments(status, order_id, tracking_number))
}

/// GET：发货单详情
#[get("/api/ship/<shipment_id>")]
pub fn get_ship(
    shipment_id: &str,
    ship_manager: &State<ShipManager>,
) -> Result<Json<Shipment>, Json<String>> {
    ship_manager
        .get_shipment(shipment_id)
        .map(Json)
        .ok_or_else(|| Json("Shipment not found".to_string()))
}

/// PUT：更新发货单信息（不改状态）
#[put("/api/ship/<shipment_id>", data = "<request>")]
pub fn update_ship(
    shipment_id: &str,
    request: Json<UpdateShipmentRequest>,
    ship_manager: &State<ShipManager>,
) -> Result<Json<Shipment>, Json<String>> {
    ship_manager
        .update_shipment(shipment_id, request.into_inner())
        .map(Json)
        .map_err(Json)
}

/// PUT：更新发货单状态
#[put("/api/ship/<shipment_id>/status", data = "<request>")]
pub fn update_ship_status(
    shipment_id: &str,
    request: Json<UpdateStatusRequest>,
    ship_manager: &State<ShipManager>,
) -> Result<Json<Shipment>, Json<String>> {
    ship_manager
        .update_status(shipment_id, request.into_inner())
        .map(Json)
        .map_err(Json)
}

/// POST：追加轨迹
#[post("/api/ship/<shipment_id>/event", data = "<request>")]
pub fn add_ship_event(
    shipment_id: &str,
    request: Json<AddEventRequest>,
    ship_manager: &State<ShipManager>,
) -> Result<Json<Shipment>, Json<String>> {
    ship_manager
        .add_event(shipment_id, request.into_inner())
        .map(Json)
        .map_err(Json)
}

/// GET：统计
#[get("/api/ship/<shipment_id>/stats")]
pub fn ship_stats(
    shipment_id: &str,
    ship_manager: &State<ShipManager>,
) -> Result<Json<ShipmentStatsResponse>, Json<String>> {
    ship_manager.stats(shipment_id).map(Json).map_err(Json)
}

/// DELETE：删除发货单
#[delete("/api/ship/<shipment_id>")]
pub fn delete_ship(
    shipment_id: &str,
    ship_manager: &State<ShipManager>,
) -> Result<Json<String>, Json<String>> {
    ship_manager
        .delete_shipment(shipment_id)
        .map(|_| Json("Shipment deleted successfully".to_string()))
        .map_err(Json)
}

/// GET：示例发货单
#[get("/api/ship/example")]
pub fn ship_example(ship_manager: &State<ShipManager>) -> Json<Shipment> {
    let shipment = ship_manager.create_shipment(CreateShipmentRequest {
        order_id: Some("ORDER-10001".to_string()),
        carrier: Some("SF".to_string()),
        tracking_number: Some("SF1234567890".to_string()),
        sender: Some(ContactInfo {
            name: "仓库A".to_string(),
            phone: None,
            address: Some("上海市浦东新区".to_string()),
        }),
        receiver: Some(ContactInfo {
            name: "张三".to_string(),
            phone: Some("13800000000".to_string()),
            address: Some("北京市朝阳区".to_string()),
        }),
        metadata: Some({
            let mut m = HashMap::new();
            m.insert("channel".to_string(), "online".to_string());
            m
        }),
    });

    let _ = ship_manager.update_status(
        &shipment.id,
        UpdateStatusRequest {
            status: "shipped".to_string(),
            message: Some("已交接承运商".to_string()),
            location: Some("上海".to_string()),
        },
    );

    Json(
        ship_manager
            .get_shipment(&shipment.id)
            .expect("shipment should exist"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get() {
        let mgr = ShipManager::new();
        let s = mgr.create_shipment(CreateShipmentRequest {
            order_id: Some("O1".to_string()),
            carrier: None,
            tracking_number: None,
            sender: None,
            receiver: None,
            metadata: None,
        });
        let got = mgr.get_shipment(&s.id).unwrap();
        assert_eq!(got.order_id.as_deref(), Some("O1"));
        assert_eq!(got.status, ShipmentStatus::Pending);
    }

    #[test]
    fn test_update_status() {
        let mgr = ShipManager::new();
        let s = mgr.create_shipment(CreateShipmentRequest {
            order_id: None,
            carrier: None,
            tracking_number: None,
            sender: None,
            receiver: None,
            metadata: None,
        });
        let updated = mgr
            .update_status(
                &s.id,
                UpdateStatusRequest {
                    status: "in_transit".to_string(),
                    message: None,
                    location: Some("杭州".to_string()),
                },
            )
            .unwrap();
        assert_eq!(updated.status, ShipmentStatus::InTransit);
        assert!(updated.events.len() >= 2);
    }
}
