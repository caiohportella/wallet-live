use sqlx::types::Json;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Clone)]
pub struct AssetModel {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
}

pub struct UserModel {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct PurchaseHistoryModel {
    #[serde(with = "time::serde::iso8601")]
    pub bought_at: OffsetDateTime,
    pub bought_for: f64,
    pub quantity_bought: f64,
    pub value_delta: f64,
}

#[derive(Serialize)]
pub struct OwnedAssetModel {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub value_delta: f64,
    pub quantity_owned: f64,
    pub purchase_history: Json<Vec<PurchaseHistoryModel>>,
}
