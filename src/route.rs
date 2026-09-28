use askama::{DynTemplate, Template};
use axum::{
    Router,
    extract::{Request, State},
    http::header::CONTENT_TYPE,
    response::{Html, IntoResponse, Response},
    routing,
};
use tower_http::trace::TraceLayer;

use crate::{
    repo::{Dir, Entry, EntryRepo, File},
    utils,
};

#[derive(Clone)]
pub struct AppState {
    pub entry_repo: EntryRepo,
}

pub fn init(state: AppState) -> Router {
    Router::new()
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
        let mut xs = vec![Breadcrumb {
            name: "root".to_owned(),
            path: "/".to_owned(),
        }];

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

        return format!("{size:.1} {}", units[unit_index]);
    }
}

async fn index(State(AppState { entry_repo }): State<AppState>, req: Request) -> Response {
    let path = req.uri().path();

    if path == "/styles.css" {
        return (
            [(CONTENT_TYPE, "text/css")],
            include_str!(concat!(env!("OUT_DIR"), "/styles.css")),
        )
            .into_response();
    }

    match entry_repo
        .query(&urlencoding::decode(path).unwrap())
        .await
        .unwrap()
    {
        Entry::Dir(Dir {
            name: _,
            path,
            children,
        }) => {
            let (directories, files) = children.into_iter().fold(
                (vec![], vec![]),
                |(mut directories, mut files), entry| {
                    match entry {
                        Entry::File(file) => files.push(file),
                        Entry::Dir(dir) => directories.push(dir),
                    }
                    (directories, files)
                },
            );
            Html(
                Index {
                    children: Box::new(FileBrowser {
                        path,
                        directories,
                        files,
                    }),
                }
                .render()
                .unwrap(),
            )
            .into_response()
        }
        Entry::File(file) => todo!(),
    }
}
