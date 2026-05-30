use axum::Router;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        let db_url = std::env::var("DATABASE_URL")?;
        let db = PgPool::connect(&db_url).await?;

        Ok(Self { db })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        dotenv::dotenv()?;

        let state = AppState::new().await?;

        info!("Starting server...");

        let listener = TcpListener::bind("127.0.0.1:3000").await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .merge(routes::frontend::router())
            .with_state(state);

        axum::serve(listener, router).await?;
        Ok(())
    }
}
