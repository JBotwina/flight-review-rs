use bytes::Bytes;
use object_store::local::LocalFileSystem;
use object_store::path::Path as ObjectPath;
use object_store::ObjectStore;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("storage error: {0}")]
    ObjectStore(#[from] object_store::Error),
    #[error("unsupported storage URL: {0}")]
    UnsupportedUrl(String),
    #[error("invalid storage configuration: {0}")]
    InvalidConfig(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct FileStorage {
    store: Arc<dyn ObjectStore>,
}

impl FileStorage {
    /// Create storage from a URL:
    /// - `file:///path/to/dir` -> LocalFileSystem
    /// - `s3://bucket/prefix`  -> AmazonS3 (requires `s3` feature)
    pub fn from_url(url: &str) -> Result<Self, StorageError> {
        if url.starts_with("file://") {
            let path = url.strip_prefix("file://").unwrap();
            std::fs::create_dir_all(path)?;
            let store =
                LocalFileSystem::new_with_prefix(path)?;
            Ok(Self {
                store: Arc::new(store),
            })
        } else if url.starts_with("s3://") {
            #[cfg(feature = "s3")]
            {
                let without_scheme = url.strip_prefix("s3://").unwrap();
                let (bucket, prefix) =
                    without_scheme.split_once('/').unwrap_or((without_scheme, ""));
                if bucket.trim().is_empty() {
                    return Err(StorageError::InvalidConfig(
                        "S3 URL must include a bucket name".to_string(),
                    ));
                }
                let s3 = build_s3_store(bucket)?;
                if prefix.is_empty() {
                    Ok(Self {
                        store: Arc::new(s3),
                    })
                } else {
                    let store = object_store::prefix::PrefixStore::new(s3, prefix);
                    Ok(Self {
                        store: Arc::new(store),
                    })
                }
            }
            #[cfg(not(feature = "s3"))]
            {
                Err(StorageError::UnsupportedUrl(
                    "S3 support not compiled in. Enable the 's3' feature.".to_string(),
                ))
            }
        } else {
            Err(StorageError::UnsupportedUrl(url.to_string()))
        }
    }

    /// Get the underlying ObjectStore for direct access.
    pub fn inner(&self) -> &Arc<dyn ObjectStore> {
        &self.store
    }

    // ── Path helpers ──────────────────────────────────────────────────

    fn log_prefix(log_id: Uuid) -> ObjectPath {
        ObjectPath::from(log_id.to_string())
    }

    fn log_file_path(log_id: Uuid, filename: &str) -> ObjectPath {
        ObjectPath::from(format!("{}/{}", log_id, filename))
    }

    // ── Convenience methods ───────────────────────────────────────────

    /// Store a file for a log.
    pub async fn put_file(
        &self,
        log_id: Uuid,
        filename: &str,
        data: Bytes,
    ) -> Result<(), StorageError> {
        let path = Self::log_file_path(log_id, filename);
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    /// Get a file for a log.
    pub async fn get_file(
        &self,
        log_id: Uuid,
        filename: &str,
    ) -> Result<Bytes, StorageError> {
        let path = Self::log_file_path(log_id, filename);
        let result = self.store.get(&path).await?;
        Ok(result.bytes().await?)
    }

    /// Get a byte range of a file (for HTTP Range requests / DuckDB-WASM).
    pub async fn get_range(
        &self,
        log_id: Uuid,
        filename: &str,
        range: std::ops::Range<u64>,
    ) -> Result<Bytes, StorageError> {
        let path = Self::log_file_path(log_id, filename);
        Ok(self.store.get_range(&path, range).await?)
    }

    /// List files for a log.
    pub async fn list_files(&self, log_id: Uuid) -> Result<Vec<String>, StorageError> {
        use futures::TryStreamExt;
        let prefix = Self::log_prefix(log_id);
        let mut files = Vec::new();
        let mut stream = self.store.list(Some(&prefix));
        while let Some(meta) = stream.try_next().await? {
            if let Some(name) = meta.location.filename() {
                files.push(name.to_string());
            }
        }
        Ok(files)
    }

    /// Get a file at an arbitrary path (relative to storage root).
    /// Used for reading v1 ULG files that are not under the standard UUID prefix.
    pub async fn get_raw(&self, path: &str) -> Result<Bytes, StorageError> {
        let obj_path = ObjectPath::from(path.to_string());
        let result = self.store.get(&obj_path).await?;
        Ok(result.bytes().await?)
    }

    /// Delete all files for a log.
    pub async fn delete_log_files(&self, log_id: Uuid) -> Result<(), StorageError> {
        use futures::TryStreamExt;
        let prefix = Self::log_prefix(log_id);
        let mut stream = self.store.list(Some(&prefix));
        while let Some(meta) = stream.try_next().await? {
            self.store.delete(&meta.location).await?;
        }
        Ok(())
    }
}

#[cfg(feature = "s3")]
fn build_s3_store(bucket: &str) -> Result<object_store::aws::AmazonS3, StorageError> {
    use object_store::aws::AmazonS3Builder;

    let mut builder = AmazonS3Builder::from_env().with_bucket_name(bucket);

    if let Some(value) = first_env(&["S3_ACCESS_KEY_ID", "ACCESS_KEY_ID"]) {
        builder = builder.with_access_key_id(value);
    }
    if let Some(value) = first_env(&["S3_SECRET_ACCESS_KEY", "SECRET_ACCESS_KEY"]) {
        builder = builder.with_secret_access_key(value);
    }
    if let Some(value) = first_env(&["S3_REGION", "REGION"]) {
        builder = builder.with_region(value);
    }

    if let Some(endpoint) = first_env(&[
        "S3_ENDPOINT",
        "AWS_ENDPOINT_URL",
        "AWS_ENDPOINT",
        "ENDPOINT",
    ]) {
        let url_style = first_env(&["S3_URL_STYLE", "AWS_S3_URL_STYLE"])
            .unwrap_or_else(|| default_url_style(&endpoint).to_string());
        let virtual_hosted = match url_style.to_ascii_lowercase().as_str() {
            "virtual" | "virtual-hosted" | "virtual_hosted" => true,
            "path" | "path-style" | "path_style" => false,
            other => {
                return Err(StorageError::InvalidConfig(format!(
                    "unsupported S3 URL style '{other}'; expected 'virtual' or 'path'"
                )))
            }
        };
        let endpoint = if virtual_hosted {
            virtual_hosted_endpoint(&endpoint, bucket)?
        } else {
            endpoint
        };
        builder = builder
            .with_endpoint(endpoint)
            .with_virtual_hosted_style_request(virtual_hosted);
    }

    Ok(builder.build()?)
}

#[cfg(feature = "s3")]
fn first_env(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

#[cfg(feature = "s3")]
fn default_url_style(endpoint: &str) -> &'static str {
    if endpoint.contains("storage.railway.app") {
        "virtual"
    } else {
        "path"
    }
}

#[cfg(feature = "s3")]
fn virtual_hosted_endpoint(endpoint: &str, bucket: &str) -> Result<String, StorageError> {
    let mut url = reqwest::Url::parse(endpoint).map_err(|error| {
        StorageError::InvalidConfig(format!("invalid S3 endpoint '{endpoint}': {error}"))
    })?;
    let host = url
        .host_str()
        .ok_or_else(|| StorageError::InvalidConfig("S3 endpoint has no host".to_string()))?;
    let bucket_prefix = format!("{bucket}.");
    if !host.starts_with(&bucket_prefix) {
        let virtual_host = format!("{bucket}.{host}");
        url.set_host(Some(&virtual_host)).map_err(|_| {
            StorageError::InvalidConfig("could not construct virtual-hosted S3 URL".to_string())
        })?;
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

#[cfg(all(test, feature = "s3"))]
mod tests {
    use super::*;

    #[test]
    fn converts_railway_base_endpoint_to_virtual_hosted_style() {
        assert_eq!(
            virtual_hosted_endpoint("https://storage.railway.app", "flight-data-abc123").unwrap(),
            "https://flight-data-abc123.storage.railway.app"
        );
    }

    #[test]
    fn preserves_an_existing_virtual_hosted_endpoint() {
        assert_eq!(
            virtual_hosted_endpoint(
                "https://flight-data-abc123.storage.railway.app",
                "flight-data-abc123"
            )
            .unwrap(),
            "https://flight-data-abc123.storage.railway.app"
        );
    }

    #[test]
    fn defaults_railway_to_virtual_and_other_providers_to_path_style() {
        assert_eq!(default_url_style("https://storage.railway.app"), "virtual");
        assert_eq!(default_url_style("https://minio.example.com"), "path");
    }
}
