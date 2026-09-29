use anyhow::{Error, Result};
use sqlx::PgPool;

use crate::{
    api::{self, B2API, ListFileNamesResponse},
    arc,
    repo::{Dir, Entry, EntryRepo, File},
};

arc!(Svc => SvcInner {
    db: PgPool,
    entry_repo: EntryRepo,
    b2_api: B2API,
    bucket_id: String,
    file_base_url: String,
});

pub enum QueryResult {
    Dir {
        path: String,
        directories: Vec<Dir>,
        files: Vec<File>,
    },
    File(String),
    NotFound,
}

impl Svc {
    pub async fn query(&self, path: &str) -> Result<QueryResult> {
        Ok(match self.entry_repo.query(path).await? {
            Some(Entry::Dir(Dir {
                name: _,
                path,
                children,
            })) => {
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
                QueryResult::Dir {
                    path,
                    directories,
                    files,
                }
            }
            Some(Entry::File(file)) => {
                let path = file
                    .path
                    .trim_start_matches('/')
                    .split('/')
                    .map(urlencoding::encode)
                    .fold(String::new(), |acc, x| acc + "/" + &x);
                QueryResult::File(self.file_base_url.clone() + &path)
            }
            None => QueryResult::NotFound,
        })
    }

    pub async fn import(&self) -> Result<()> {
        let mut tx = self.db.begin().await?;

        let mut start = Some(String::new());
        while let Some(start_file_name) = start {
            let ListFileNamesResponse {
                files,
                next_file_name,
            } = self
                .b2_api
                .list_file_names(&self.bucket_id, &start_file_name)
                .await?;

            let mut paths: Vec<String> = Vec::with_capacity(files.len());
            let mut sizes: Vec<i64> = Vec::with_capacity(files.len());
            let mut content_types: Vec<String> = Vec::with_capacity(files.len());
            let mut last_modified_items: Vec<i64> = Vec::with_capacity(files.len());
            for api::File {
                action,
                content_length,
                content_type,
                file_name,
                upload_timestamp,
            } in files
            {
                if action != "upload" {
                    continue;
                }
                paths.push(format!("/{file_name}"));
                sizes.push(content_length);
                content_types.push(content_type);
                last_modified_items.push(upload_timestamp);
            }

            EntryRepo::push_tmp(&mut tx, paths, sizes, content_types, last_modified_items).await?;
            start = next_file_name;
        }

        EntryRepo::merge(&mut tx).await?;
        EntryRepo::reset_tmp(&mut tx).await?;

        tx.commit().await.map_err(Error::new)
    }
}
