use std::{env::VarError::NotPresent, sync::Arc};

use anyhow::{Context, Error, Result, bail};
use dotenvy::Error::EnvVar;
use sqlx::{PgPool, migrate};
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

    fn read_opt(key: &str) -> Result<Option<String>> {
        match dotenvy::var(key) {
            Ok(x) => Ok(Some(x)),
            Err(EnvVar(NotPresent)) => Ok(None),
            Err(e) => Err(e).with_context(|| format!("failed to read env `{key}`")),
        }
    }

    let listen = dotenvy::var("LISTEN").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = dotenvy::var("DATABASE_URL")?;
    let b2_id = Arc::from(dotenvy::var("B2_ID")?);
    let b2_key = Arc::from(dotenvy::var("B2_KEY")?);
    let (bucket_id, bucket_name) = match (read_opt("BUCKET_ID")?, read_opt("BUCKET_NAME")?) {
        (None, None) => bail!("both env `BUCKET_ID` and `BUCKET_NAME` not found"),
        x => x,
    };
    let file_base_url = dotenvy::var("FILE_BASE_URL")?;
    let jwt_secret = dotenvy::var("JWT_SECRET")?;

    let db = PgPool::connect(&database_url).await?;
    migrate!().run(&db).await?;

    let file_repo = FileRepo::new(db.clone());
    let user_repo = UserRepo::new(db.clone());
    let b2_api = B2API::new(b2_id, b2_key);
    let file_svc = FileSvc::new(db, file_repo, b2_api, bucket_id, bucket_name, file_base_url);

    let user_svc = UserSvc::new(user_repo);
    user_svc.ensure_init().await?;

    let app = route::init(AppState::new(file_svc, user_svc, jwt_secret));
    let listener = TcpListener::bind(listen).await?;
    info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await.map_err(Error::new)
}

fn log_init() {
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
