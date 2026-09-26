use std::collections::VecDeque;

use anyhow::{Result, ensure};
use sqlx::{PgPool, query_as};

use crate::utils;

#[derive(Clone)]
pub struct EntryRepo {
    db: PgPool,
}

struct RawFile {
    path: String,
    size: i64,
    content_type: String,
    last_modified: i64,
}

#[derive(Debug)]
pub enum Entry {
    File(File),
    Dir(Dir),
}

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub path: String,
    pub size: i64,
    pub content_type: String,
    pub last_modified: i64,
}

#[derive(Debug)]
pub struct Dir {
    pub name: String,
    pub path: String,
    pub children: Vec<Entry>,
}

impl Entry {
    fn path(&self) -> &str {
        match self {
            Entry::File(File { path, .. }) => path,
            Entry::Dir(Dir { path, .. }) => path,
        }
    }
}

impl EntryRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn query(&self, path: &str) -> Result<Entry> {
        assert!(!path.ends_with('/') || path == "/");

        let mut raw_files: Vec<RawFile> = query_as!(
            RawFile,
            "
            SELECT * FROM files
            WHERE CASE WHEN $1 != '/' THEN path = $1 OR path LIKE $1 || '/%' ELSE TRUE END
            ORDER BY path
            ",
            path
        )
        .fetch_all(&self.db)
        .await?;
        ensure!(raw_files.len() != 0, "not found");

        fn raw_to_entry(
            RawFile {
                path,
                size,
                content_type,
                last_modified,
            }: RawFile,
        ) -> Entry {
            fn name(path: &str) -> String {
                path[path.rfind('/').unwrap() + 1..].to_owned()
            }
            Entry::File(File {
                name: name(&path),
                path,
                size,
                content_type,
                last_modified,
            })
        }

        if raw_files.len() == 1 && raw_files[0].path == path {
            return Ok(raw_to_entry(raw_files.pop().unwrap()));
        }

        #[derive(PartialEq)]
        enum Segment {
            Root,
            Normal(String),
        }

        fn to_dir_segments(path: &str) -> VecDeque<Segment> {
            let mut segments = VecDeque::new();
            if path.starts_with('/') {
                segments.push_back(Segment::Root);
            }
            segments.extend(
                path.trim_start_matches('/')
                    .split('/')
                    .map(|segment| Segment::Normal(segment.to_owned())),
            );
            segments.pop_back();
            segments
        }

        struct Frame {
            segment: Segment,
            children: Vec<Entry>,
        }

        impl Frame {
            fn to_dir_entry(self) -> Entry {
                Entry::Dir(Dir {
                    name: if let Segment::Normal(segment) = self.segment {
                        segment
                    } else {
                        "/".to_owned()
                    },
                    path: utils::dir(self.children[0].path()),
                    children: self.children,
                })
            }
        }

        let mut stack = vec![Frame {
            segment: Segment::Root,
            children: vec![],
        }];

        for file in raw_files {
            let mut segments = to_dir_segments(&file.path);

            let mut common_prefix_len = 0;
            let mut i = 0;
            while i < stack.len() && i < segments.len() && segments[i] == stack[i].segment {
                common_prefix_len += 1;
                i += 1;
            }

            if common_prefix_len < stack.len() {
                for _ in 0..stack.len() - common_prefix_len {
                    let top = stack.pop().unwrap();
                    stack.last_mut().unwrap().children.push(top.to_dir_entry());
                }
            }

            if segments.len() > stack.len() {
                for _ in 0..stack.len() {
                    segments.pop_front();
                }
                stack.extend(segments.into_iter().map(|segment| Frame {
                    segment,
                    children: vec![],
                }));
            }

            stack.last_mut().unwrap().children.push(raw_to_entry(file));
        }

        let level = if path == "/" {
            1
        } else {
            path.split('/').count()
        };
        while stack.len() > level {
            let top = stack.pop().unwrap();
            stack.last_mut().unwrap().children.push(top.to_dir_entry());
        }

        Ok(stack.pop().unwrap().to_dir_entry())
    }
}
