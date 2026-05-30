use axum::{Json, Router, routing::get};
use serde::Deserialize;

use crate::{
    app::AppState, auth::admin::Admin, errors::AppError, models::AssetModel, repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/assets",
        get(list_assets).post(create_asset).patch(update_asset),
    )
}

#[tracing::instrument(skip_all)]
async fn list_assets(repository: Repository) -> Result<Json<Vec<AssetModel>>, AppError> {
    let assets = repository.list_assets().await?;

    Ok(Json(assets))
}

#[derive(Deserialize)]
struct CreateAssetReq {
    name: String,
    unit_value: f64,
}

#[tracing::instrument(skip_all)]
async fn create_asset(
    _: Admin,
    repository: Repository,
    Json(req): Json<CreateAssetReq>,
) -> Result<Json<AssetModel>, AppError> {
    let new_asset = repository
        .create_asset(AssetModel {
            id: 0,
            name: req.name,
            unit_value: req.unit_value,
        })
        .await?;

    Ok(Json(new_asset))
}

#[derive(Deserialize)]
struct UpdateAssetReq {
    id: i64,
    name: Option<String>,
    unit_value: Option<f64>,
}

#[tracing::instrument(skip_all)]
async fn update_asset(
    _: Admin,
    repository: Repository,
    Json(req): Json<UpdateAssetReq>,
) -> Result<Json<AssetModel>, AppError> {
    match repository
        .update_asset(req.id, req.name, req.unit_value)
        .await?
    {
        Some(updated_asset) => Ok(Json(updated_asset)),
        None => Err(AppError::AssetDoesNotExist),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures(path = "fixtures", scripts("bitcoin.asset.sql")))]
    async fn test_list_assets(db: sqlx::PgPool) {
        let Json(assets) = list_assets(db.into()).await.expect("success");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, 1);
        assert_eq!(assets[0].name, "Bitcoin");
        assert_eq!(assets[0].unit_value, 10.0);

        insta::assert_json_snapshot!(assets);
    }

    #[sqlx::test]
    async fn test_create_asset(db: sqlx::PgPool) {
        let req = Json(CreateAssetReq {
            name: "Bitcoin".to_string(),
            unit_value: 10.0,
        });

        let Json(new_asset) = create_asset(Admin, db.into(), req).await.expect("success");

        assert_eq!(new_asset.id, 1);
        assert_eq!(new_asset.name, "Bitcoin");
        assert_eq!(new_asset.unit_value, 10.0);

        insta::assert_json_snapshot!(new_asset);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("bitcoin.asset.sql")))]
    async fn test_update_asset(db: sqlx::PgPool) {
        let req = Json(UpdateAssetReq {
            id: 1,
            name: Some("Ethereum".to_string()),
            unit_value: Some(20.0),
        });

        let Json(updated_asset) = update_asset(Admin, db.into(), req).await.expect("success");

        assert_eq!(updated_asset.id, 1);
        assert_eq!(updated_asset.name, "Ethereum");
        assert_eq!(updated_asset.unit_value, 20.0);

        insta::assert_json_snapshot!(updated_asset);
    }
}
