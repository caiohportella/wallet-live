use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{AssetModel, OwnedAssetModel, UserModel},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<AssetModel>> {
        sqlx::query_as!(AssetModel, "SELECT * FROM assets;")
            .fetch_all(&self.db)
            .await
    }

    pub async fn create_asset(&self, asset: AssetModel) -> sqlx::Result<AssetModel> {
        sqlx::query_as!(
            AssetModel,
            "INSERT INTO assets (name, unit_value) VALUES ($1, $2) RETURNING id, name, unit_value;",
            asset.name,
            asset.unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<AssetModel>> {
        sqlx::query_as!(
            AssetModel,
            "UPDATE assets

            SET name = COALESCE($1, name), unit_value = COALESCE($2, unit_value)
            WHERE id = $3
            RETURNING id, name, unit_value;",
            name,
            unit_value,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> sqlx::Result<UserModel> {
        sqlx::query_as!(
            UserModel,
            "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username, password_hash;",
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> sqlx::Result<Option<UserModel>> {
        sqlx::query_as!(
            UserModel,
            "SELECT id, username, password_hash FROM users WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn insert_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: f64,
        unit_value: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO owned_assets (user_id, asset_id, quantity_owned, bought_for) VALUES ($1, $2, $3, $4);",
            user_id,
            asset_id,
            quantity,
            unit_value
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn list_owned_assets(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAssetModel>> {
        sqlx::query_as!(
            OwnedAssetModel,
            r#"
            SELECT
                asset.id,
                asset.name,
                asset.unit_value,
                SUM((asset.unit_value - owned_asset.bought_for) * owned_asset.quantity_owned) AS "value_delta!",
                SUM(owned_asset.quantity_owned) AS "quantity_owned!",
                JSON_AGG(
                    JSON_BUILD_OBJECT(
                        'bought_at', owned_asset.timestamp,
                        'bought_for', owned_asset.bought_for,
                        'quantity_bought', owned_asset.quantity_owned,
                        'value_delta', (asset.unit_value - owned_asset.bought_for) * owned_asset.quantity_owned
                    )
                    ORDER BY owned_asset.timestamp DESC
                ) AS "purchase_history!: _"
            FROM assets AS asset
            JOIN owned_assets AS owned_asset
                ON owned_asset.asset_id = asset.id
            WHERE owned_asset.user_id = $1
            GROUP BY asset.id
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
