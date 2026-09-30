#![allow(clippy::result_large_err)]

use askama::{DynTemplate, Template};
use axum::{
    Json, Router,
    extract::{FromRef, Request, State},
    http::{
        HeaderName, StatusCode,
        header::{CONTENT_TYPE, SET_COOKIE},
    },
    response::{Html, IntoResponse, Redirect, Response},
    routing,
};
use jsonwebtoken::{EncodingKey, Header};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{error, instrument, warn};

use crate::{
    repo::{Dir, File},
    service::{FileSvc, UserSvc},
    utils,
};

macro_rules! throw {
    ($res:expr) => {
        return Err(error($res))
    };
}

macro_rules! Result {
    () => {
      Result<impl IntoResponse>
    };
}

#[derive(Clone, FromRef)]
pub struct AppState {
    file_svc: FileSvc,
    user_svc: UserSvc,
}

impl AppState {
    pub fn new(file_svc: FileSvc, user_svc: UserSvc) -> Self {
        Self { file_svc, user_svc }
    }
}

pub fn init(state: AppState) -> Router {
    #[derive(Template)]
    #[template(path = "login.html")]
    struct Login;

    let login_page = Html(
        Index {
            children: Box::new(Login),
        }
        .render()
        .unwrap(),
    );

    Router::new()
        .route("/styles.css", routing::get(CSS))
        .route("/login", routing::get(login_page))
        .route("/api/login", routing::post(login))
        .route("/api/import", routing::post(import))
        .fallback(routing::get(index))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

const CSS: ([(HeaderName, &str); 1], &str) = (
    [(CONTENT_TYPE, "text/css")],
    include_str!(concat!(env!("OUT_DIR"), "/styles.css")),
);

#[derive(Deserialize)]
struct LoginParams {
    username: String,
    password: String,
}

async fn login(
    State(user_svc): State<UserSvc>,
    Json(LoginParams { username, password }): Json<LoginParams>,
) -> Result!() {
    if !user_svc.login(&username, &password).await? {
        throw!((StatusCode::UNAUTHORIZED, "wrong username or password"));
    }
    // TODO: use secret from env
    let token = jsonwebtoken::encode(&Header::default(), &(), &EncodingKey::from_secret(&[]))?;
    Ok((
        StatusCode::NO_CONTENT,
        [(SET_COOKIE, format!("token={token}; HttpOnly; Path=/"))],
    ))
}

async fn import(State(svc): State<FileSvc>) -> impl IntoResponse {
    svc.import()
        .await
        .and(Ok(StatusCode::NO_CONTENT))
        .map_err(Error::from)
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
async fn index(State(svc): State<FileSvc>, req: Request) -> Result {
    let path = urlencoding::decode(req.uri().path()).map_err(|e| {
        warn!("failed to decode path in URL: {e}");
        error((StatusCode::BAD_REQUEST, "invalid path in URL"))
    })?;
    use crate::service::QueryResult::*;
    Ok(match svc.query(&path).await? {
        Dir {
            path,
            directories,
            files,
        } => Html(
            Index {
                children: Box::new(FileBrowser {
                    path,
                    directories,
                    files,
                }),
            }
            .render()?,
        )
        .into_response(),
        File(url) => Redirect::to(&url).into_response(),
        NotFound => StatusCode::NOT_FOUND.into_response(),
    })
}

struct Error(Response);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.0
    }
}

impl<E: Into<anyhow::Error>> From<E> for Error {
    fn from(value: E) -> Self {
        error!("unexpected error: {:?}", value.into());
        Self(StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}

fn error(res: impl IntoResponse) -> Error {
    Error(res.into_response())
}

type Result<T = Response, E = Error> = std::result::Result<T, E>;
