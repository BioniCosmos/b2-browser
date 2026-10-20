use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anyhow::{Error, Result, bail};
use base64::prelude::*;
use reqwest::{Client, header::AUTHORIZATION};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct B2API(Arc<B2APIInner>);

#[derive(Default)]
pub struct B2APIInner {
    client: Client,
    id: Arc<str>,
    key: Arc<str>,
    state: RwLock<State>,
}

struct State {
    created_at: SystemTime,
    base_url: String,
    token: String,
}

impl Deref for B2API {
    type Target = B2APIInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            created_at: SystemTime::UNIX_EPOCH,
            base_url: Default::default(),
            token: Default::default(),
        }
    }
}

impl B2API {
    pub fn new(id: Arc<str>, key: Arc<str>) -> Self {
        Self(Arc::new(B2APIInner {
            id,
            key,
            ..Default::default()
        }))
    }

    async fn ensure_state(&self) -> Result<()> {
        if self.state.read().await.created_at + Duration::from_hours(24) > SystemTime::now() {
            return Ok(());
        }
        let res = self.authorize_account().await?;
        *self.state.write().await = State {
            created_at: SystemTime::now(),
            base_url: res.api_info.storage_api.api_url,
            token: res.authorization_token,
        };
        Ok(())
    }

    async fn authorize_account(&self) -> Result<AuthorizeAccountResponse> {
        let res = self
            .client
            .get("https://api.backblazeb2.com/b2api/v4/b2_authorize_account")
            .header(
                AUTHORIZATION,
                format!(
                    "Basic {}",
                    BASE64_STANDARD.encode(format!("{}:{}", self.id, self.key))
                ),
            )
            .send()
            .await?;
        if !res.status().is_success() {
            let ErrorDetails {
                status,
                code,
                message,
            } = res.json::<ErrorDetails>().await?;
            bail!("[B2API::authorize_account] {status} {code}: {message}");
        }
        res.json().await.map_err(Error::new)
    }

    pub async fn list_file_names(
        &self,
        bucket_id: &str,
        start_file_name: &str,
    ) -> Result<ListFileNamesResponse> {
        self.ensure_state().await?;

        let mut query = HashMap::from([("bucketId", bucket_id), ("maxFileCount", "10000")]);
        if !start_file_name.is_empty() {
            query.insert("startFileName", start_file_name);
        }
        let State {
            base_url, token, ..
        } = &*self.state.read().await;
        let res = self
            .client
            .get(format!("{base_url}/b2api/v4/b2_list_file_names",))
            .header(AUTHORIZATION, token)
            .query(&query)
            .send()
            .await?;
        if !res.status().is_success() {
            let ErrorDetails {
                status,
                code,
                message,
            } = res.json::<ErrorDetails>().await?;
            bail!("[B2API::list_file_names] {status} {code}: {message}");
        }
        res.json().await.map_err(Error::new)
    }
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetails {
    status: i32,
    code: String,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeAccountResponse {
    api_info: APIInfo,
    authorization_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIInfo {
    storage_api: StorageAPI,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageAPI {
    api_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFileNamesResponse {
    pub files: Vec<File>,
    pub next_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub action: String,
    pub content_length: i64,
    pub content_type: String,
    pub file_name: String,
    pub upload_timestamp: i64,
}
