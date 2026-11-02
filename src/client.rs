use crate::model::{Event, Snapshot};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Cannot read the local service connection. Start factoryd first: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid local service connection")]
    Invalid,
    #[error("Cannot reach the local service: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Local service rejected the request: {0}")]
    Rejected(String),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionInfo {
    pub schema_version: u32,
    pub base_url: String,
    pub token: String,
}

pub fn default_data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("FACTORY_DATA_DIR") {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    #[cfg(target_os = "macos")]
    return home.join("Library/Application Support/SoftwareFactory");
    #[cfg(not(target_os = "macos"))]
    return home.join(".local/share/software-factory");
}

pub fn read_connection(data_dir: &Path) -> Result<ConnectionInfo, ClientError> {
    let path = data_dir.join("connection.json");
    let meta = std::fs::symlink_metadata(&path)?;
    if !meta.is_file() || meta.len() > 4096 {
        return Err(ClientError::Invalid);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o077 != 0 || meta.uid() != unsafe { libc::geteuid() } {
            return Err(ClientError::Invalid);
        }
    }
    let value: ConnectionInfo =
        serde_json::from_slice(&std::fs::read(path)?).map_err(|_| ClientError::Invalid)?;
    let url = reqwest::Url::parse(&value.base_url).map_err(|_| ClientError::Invalid)?;
    if value.schema_version != 1
        || url.scheme() != "http"
        || url.host_str() != Some("127.0.0.1")
        || url.port().is_none()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || value.token.len() != 64
        || !value.token.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(ClientError::Invalid);
    }
    Ok(value)
}

#[derive(Clone)]
pub struct DaemonClient {
    http: reqwest::Client,
    connection: ConnectionInfo,
}

impl DaemonClient {
    pub fn connect(data_dir: PathBuf) -> Result<Self, ClientError> {
        let connection = read_connection(&data_dir)?;
        let http = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self { http, connection })
    }
    async fn response<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, ClientError> {
        let response = request.bearer_auth(&self.connection.token).send().await?;
        let status = response.status();
        if response.content_length().unwrap_or(0) > 16 * 1024 * 1024 {
            return Err(ClientError::Invalid);
        }
        let bytes = response.bytes().await?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(ClientError::Invalid);
        }
        if !status.is_success() {
            let reason = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| v["error"].as_str().map(str::to_owned))
                .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
            return Err(ClientError::Rejected(reason));
        }
        serde_json::from_slice(&bytes).map_err(|_| ClientError::Invalid)
    }
    pub async fn snapshot(&self) -> Result<Snapshot, ClientError> {
        self.response(
            self.http
                .get(format!("{}v1/snapshot", self.connection.base_url)),
        )
        .await
    }
    pub async fn events(&self, after: u64, limit: u32) -> Result<Vec<Event>, ClientError> {
        self.response(self.http.get(format!(
            "{}v1/events?after={after}&limit={}",
            self.connection.base_url,
            limit.clamp(1, 500)
        )))
        .await
    }
    pub async fn capabilities(&self) -> Result<Value, ClientError> {
        self.response(
            self.http
                .get(format!("{}v1/capabilities", self.connection.base_url)),
        )
        .await
    }
    pub async fn command(&self, command: Value) -> Result<Value, ClientError> {
        self.response(
            self.http
                .post(format!("{}v1/commands", self.connection.base_url))
                .json(&command),
        )
        .await
    }
}
