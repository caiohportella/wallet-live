use crate::app::App;

mod app;
mod models;
mod routes;
mod auth;
mod errors;
mod repository;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    App::start().await?;
    Ok(())
}