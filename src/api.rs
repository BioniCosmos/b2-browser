use std::collections::HashMap;

use anyhow::{Error, Result, bail};
use base64::prelude::*;
use reqwest::{Client, header::AUTHORIZATION};
use serde::Deserialize;

use crate::arc;

arc!(B2API => B2APIInner {
    client: Client,
    base_url: String,
    token: String,
}, pub(self) new);

impl B2API {
    pub async fn init(id: &str, key: &str) -> Result<Self> {
        let client = Client::new();
        let res = Self::authorize_account(&client, id, key).await?;
        Ok(Self::new(
            client,
            res.api_info.storage_api.api_url,
            res.authorization_token,
        ))
    }

    async fn authorize_account(
        client: &Client,
        id: &str,
        key: &str,
    ) -> Result<AuthorizeAccountResponse> {
        let res = client
            .get("https://api.backblazeb2.com/b2api/v4/b2_authorize_account")
            .header(
                AUTHORIZATION,
                format!("Basic {}", BASE64_STANDARD.encode(format!("{id}:{key}"))),
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
        let mut query = HashMap::from([("bucketId", bucket_id), ("maxFileCount", "10000")]);
        if !start_file_name.is_empty() {
            query.insert("startFileName", start_file_name);
        }
        let res = self
            .client
            .get(format!("{}/b2api/v4/b2_list_file_names", self.base_url))
            .header(AUTHORIZATION, &self.token)
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
