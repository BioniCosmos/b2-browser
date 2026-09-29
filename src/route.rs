use std::env;

use askama::{DynTemplate, Template};
use axum::{
    Router,
    extract::{FromRef, Request, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse, Redirect, Response, Result},
    routing,
};
use tower_http::trace::TraceLayer;
use tracing::{error, instrument, warn};

use crate::{
    repo::{Dir, File},
    service::Svc,
    utils,
};

#[derive(Clone, FromRef)]
pub struct AppState {
    svc: Svc,
}

impl AppState {
    pub fn new(svc: Svc) -> Self {
        Self { svc }
    }
}

pub fn init(state: AppState) -> Router {
    Router::new()
        .route("/api/import", routing::post(import))
        .fallback(routing::get(index))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    children: Box<dyn DynTemplate>,
}

#[derive(Template)]
#[template(path = "file-browser.html")]
struct FileBrowser {
    path: String,
    directories: Vec<Dir>,
    files: Vec<File>,
}

struct Breadcrumb {
    name: String,
    path: String,
}

impl FileBrowser {
    fn breadcrumbs(path: &str) -> Vec<Breadcrumb> {
        let mut xs = vec![];

        let path = path.as_bytes();
        let mut i = 1;
        while i < path.len() {
            let start = i;
            while i < path.len() && path[i] != b'/' {
                i += 1;
            }
            xs.push(unsafe {
                Breadcrumb {
                    name: String::from_utf8_unchecked(path[start..i].to_vec()),
                    path: String::from_utf8_unchecked(path[..i].to_vec()),
                }
            });
            i += 1;
        }

        xs
    }

    fn format_size(size: i64) -> String {
        let units = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < units.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{size:.1} {}", units[unit_index])
    }
}

#[instrument(skip_all)]
#[allow(clippy::result_large_err)]
async fn index(State(svc): State<Svc>, req: Request) -> Result<Response> {
    let path = req.uri().path();

    if path == "/styles.css" {
        let res = (
            [(CONTENT_TYPE, "text/css")],
            include_str!(concat!(env!("OUT_DIR"), "/styles.css")),
        );
        return Ok(res.into_response());
    }

    let path = urlencoding::decode(path).map_err(|e| {
        warn!("failed to decode path in URL: {e}");
        (StatusCode::BAD_REQUEST, "invalid path in URL")
    })?;
    use crate::service::QueryResult::*;
    Ok(match svc.query(&path).await.map_err(Error::new)? {
        Dir {
            path,
            directories,
            files,
        } => Html(
            FileBrowser {
                path,
                directories,
                files,
            }
            .render()
            .map_err(Error::from)?,
        )
        .into_response(),
        File(url) => Redirect::to(&url).into_response(),
        NotFound => StatusCode::NOT_FOUND.into_response(),
    })
}

async fn import(State(svc): State<Svc>) -> impl IntoResponse {
    svc.import()
        .await
        .and(Ok(StatusCode::NO_CONTENT))
        .map_err(Error::new)
}

struct Error(anyhow::Error);

impl Error {
    fn new(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("unexpected error: {:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for Error {
    fn from(value: E) -> Self {
        Self(anyhow::Error::new(value))
    }
}
