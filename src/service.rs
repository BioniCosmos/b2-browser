use anyhow::{Error, Result};
use sqlx::PgPool;

use crate::{
    api::{B2API, File, ListFileNamesResponse},
    arc,
    repo::EntryRepo,
};

arc!(Svc => SvcInner {
    db: PgPool,
    b2_api: B2API,
    bucket_id: String,
});

impl Svc {
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
            for File {
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
