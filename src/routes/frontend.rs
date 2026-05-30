use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;

use crate::{
    app::AppState,
    auth::users::{UnauthenticatedUser, User},
    errors::AppError,
    models::{AssetModel, OwnedAssetModel, PurchaseHistoryModel},
    repository::Repository,
};

// ── Template structs ──────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    username: String,
    owned_assets: Vec<AssetView>,
    has_owned_assets: bool,
    #[allow(dead_code)]
    available_assets: Vec<AssetModel>,
    #[allow(dead_code)]
    has_available_assets: bool,
}

// ── Dashboard view models ─────────────────────────────────────────────────────

struct AssetView {
    id: i64,
    name: String,
    qty: String,
    price: String,
    delta: String,
    positive: bool,
    history: Vec<HistoryView>,
    has_history: bool,
}

struct HistoryView {
    date: String,
    price: String,
    qty: String,
    delta: String,
    positive: bool,
}

impl From<PurchaseHistoryModel> for HistoryView {
    fn from(p: PurchaseHistoryModel) -> Self {
        Self {
            date: p
                .bought_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "Invalid date".to_string()),
            price: format!("${:.2}", p.bought_for),
            qty: format!("{:.4}", p.quantity_bought),
            delta: format!("${:.2}", p.value_delta.abs()),
            positive: p.value_delta >= 0.0,
        }
    }
}

impl From<OwnedAssetModel> for AssetView {
    fn from(asset: OwnedAssetModel) -> Self {
        let history: Vec<HistoryView> = asset
            .purchase_history
            .0
            .into_iter()
            .map(Into::into)
            .collect();

        Self {
            id: asset.id,
            name: asset.name,
            qty: format!("{:.4}", asset.quantity_owned),
            price: format!("${:.2}", asset.unit_value),
            delta: format!("${:.2}", asset.value_delta.abs()),
            positive: asset.value_delta >= 0.0,
            has_history: !history.is_empty(),
            history,
        }
    }
}

// ── Form request types ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct PurchaseAssetForm {
    asset_id: i64,
    unity_value: f64,
    quantity: f64,
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/dashboard", get(dashboard).post(purchase_asset))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .route("/register", get(register_page).post(register))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn home() -> Result<Html<String>, AppError> {
    Ok(Html(IndexPage.render()?))
}

async fn login_page() -> Result<Html<String>, AppError> {
    Ok(Html(LoginPage.render()?))
}

async fn register_page() -> Result<Html<String>, AppError> {
    Ok(Html(RegisterPage.render()?))
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(req): Form<LoginForm>,
) -> Result<Response, AppError> {
    let unauth_user = UnauthenticatedUser::new(req.username, req.password);

    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::InvalidCredentials) => {
            return Ok(Redirect::to("/login?error=invalid_credentials").into_response());
        }
        Err(err) => return Err(err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .path("/")
        .build();

    Ok((jar.add(cookie), Redirect::to("/dashboard")).into_response())
}

async fn register(
    repository: Repository,
    Form(req): Form<RegisterForm>,
) -> Result<Response, AppError> {
    let unauth_user = UnauthenticatedUser::new(req.username, req.password);

    match unauth_user.register(&repository).await {
        Ok(_) => Ok(Redirect::to("/login?success=account_created").into_response()),
        Err(AppError::UsernameTaken) => {
            Ok(Redirect::to("/register?error=username_taken").into_response())
        }
        Err(err) => Err(err),
    }
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let removal = Cookie::build(("token", "")).path("/").build();
    (jar.remove(removal), Redirect::to("/login"))
}

async fn dashboard(repository: Repository, user: User) -> Result<Html<String>, AppError> {
    let available_assets = repository.list_assets().await?;
    let owned_assets: Vec<AssetView> = repository
        .list_owned_assets(user.id())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    let html = DashboardPage {
        username: user.username.clone(),
        has_owned_assets: !owned_assets.is_empty(),
        has_available_assets: !available_assets.is_empty(),
        owned_assets,
        available_assets,
    }
    .render()?;

    Ok(Html(html))
}

async fn purchase_asset(
    repository: Repository,
    user: User,
    Form(req): Form<PurchaseAssetForm>,
) -> Result<Redirect, AppError> {
    repository
        .insert_owned_asset(user.id(), req.asset_id, req.quantity, req.unity_value)
        .await?;

    Ok(Redirect::to("/dashboard"))
}
