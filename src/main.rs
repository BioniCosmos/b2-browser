use anyhow::{Error, Result};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;

use crate::{api::B2API, repo::EntryRepo, route::AppState, service::Svc};

mod api;
mod repo;
mod route;
mod service;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    log_init();

    let listen = dotenvy::var("LISTEN").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = dotenvy::var("DATABASE_URL")?;
    let b2_id = dotenvy::var("B2_ID")?;
    let b2_key = dotenvy::var("B2_KEY")?;
    let bucket_id = dotenvy::var("BUCKET_ID")?;
    let file_base_url = dotenvy::var("FILE_BASE_URL")?;

    let db = PgPool::connect(&database_url).await?;
    let entry_repo = EntryRepo::new(db.clone());
    let b2_api = B2API::init(&b2_id, &b2_key).await?;
    let svc = Svc::new(db, entry_repo, b2_api, bucket_id, file_base_url);
    let app = route::init(AppState::new(svc));

    let listener = TcpListener::bind(listen).await?;
    info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await.map_err(Error::new)
}

fn log_init() {
    use std::env::VarError::NotPresent;

    use dotenvy::Error::EnvVar;
    use tracing_subscriber::EnvFilter;

    let filter = if let Err(EnvVar(NotPresent)) = dotenvy::var(EnvFilter::DEFAULT_ENV) {
        EnvFilter::new(format!(
            "{}=trace,tower_http=debug,axum::rejection=trace",
            env!("CARGO_CRATE_NAME")
        ))
    } else {
        EnvFilter::from_default_env()
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .pretty()
        .init();
}
