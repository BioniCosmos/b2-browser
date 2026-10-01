use anyhow::{Error, Result};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    api::B2API,
    repo::{FileRepo, UserRepo},
    route::AppState,
    service::{FileSvc, UserSvc},
};

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
    let jwt_secret = dotenvy::var("JWT_SECRET")?;

    let db = PgPool::connect(&database_url).await?;
    let file_repo = FileRepo::new(db.clone());
    let user_repo = UserRepo::new(db.clone());
    let b2_api = B2API::init(&b2_id, &b2_key).await?;
    let file_svc = FileSvc::new(db, file_repo, b2_api, bucket_id, file_base_url);
    let user_svc = UserSvc::new(user_repo);
    let app = route::init(AppState::new(file_svc, user_svc, jwt_secret));

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
