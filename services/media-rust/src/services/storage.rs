use async_trait::async_trait;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::errors::AppError;

/// Trait for storage backends that can store, retrieve, delete, and
/// generate presigned URLs for files.
///
/// # Expected Behavior
///
/// Implementations must support putting objects (writing file bytes with
/// a content type), getting objects (reading file bytes), deleting objects,
/// generating presigned GET URLs, checking health, and reporting the
/// provider name. All methods are async and return `Result` with `AppError`.
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Write `data` bytes to `path` with the given `content_type`.
    ///
    /// # Expected Behavior
    ///
    /// Stores the file content at the specified path. Overwrites any
    /// existing file at the same path. For local storage, also writes
    /// a sidecar `.meta` JSON file with `content_type` and size. For S3,
    /// uploads the object with the specified content type metadata.
    ///
    /// # Errors
    ///
    /// Returns `AppError::StorageError` if the write fails (filesystem
    /// error, S3 API error, etc.).
    ///
    /// # Side Effects
    ///
    /// - Writes bytes to the filesystem or S3.
    /// - Creates parent directories if they do not exist (local storage).
    /// - Writes a sidecar `.meta` file (local storage).
    async fn put_object(&self, path: &str, data: &[u8], content_type: &str)
        -> Result<(), AppError>;

    /// Read the file bytes at `path`.
    ///
    /// # Expected Behavior
    ///
    /// Retrieves the full content of the file at the specified path.
    /// For local storage, reads from the filesystem. For S3, downloads
    /// the object. Returns the raw bytes.
    ///
    /// # Errors
    ///
    /// Returns `AppError::StorageError` if the file does not exist or
    /// the read fails.
    ///
    /// # Side Effects
    ///
    /// - Reads bytes from the filesystem or S3.
    async fn get_object(&self, path: &str) -> Result<Vec<u8>, AppError>;

    /// Delete the file at `path`.
    ///
    /// # Expected Behavior
    ///
    /// Removes the file at the specified path. For local storage, also
    /// removes the sidecar `.meta` file. For S3, deletes the object.
    /// Returns Ok(()) even if the file did not exist (idempotent).
    ///
    /// # Errors
    ///
    /// Returns `AppError::StorageError` if the delete fails for a reason
    /// other than the file not existing.
    ///
    /// # Side Effects
    ///
    /// - Deletes bytes from the filesystem or S3.
    /// - Deletes the sidecar `.meta` file (local storage).
    async fn delete_object(&self, path: &str) -> Result<(), AppError>;

    /// Generate a presigned GET URL for the file at `path`.
    ///
    /// # Expected Behavior
    ///
    /// For S3, generates a presigned URL that grants temporary read access
    /// for `expiration` seconds. For local storage, returns a relative URL
    /// path like `/api/media/<file_id>` since presigned URLs are not
    /// applicable to local storage.
    ///
    /// # Errors
    ///
    /// Returns `AppError::StorageError` if URL generation fails.
    ///
    /// # Side Effects
    ///
    /// - May perform S3 API call (network) for presigning.
    async fn presign_get(&self, path: &str, expiration: u64) -> Result<String, AppError>;

    /// Check if the storage backend is healthy.
    ///
    /// # Expected Behavior
    ///
    /// For local storage, checks that the storage directory exists and is
    /// writable. For S3, performs a lightweight check (e.g., list objects
    /// with max 1 or head bucket). Returns `Ok(())` if healthy.
    ///
    /// # Errors
    ///
    /// Returns `AppError::StorageError` if the storage is not healthy.
    ///
    /// # Side Effects
    ///
    /// - May perform filesystem or network I/O to verify health.
    #[allow(dead_code)]
    async fn health_check(&self) -> Result<(), AppError>;

    /// Return the name of this storage provider (e.g., "local" or "s3").
    ///
    /// # Expected Behavior
    ///
    /// Returns a static string identifier for the storage backend.
    ///
    /// # Errors
    ///
    /// None. Always returns a valid string.
    ///
    /// # Side Effects
    ///
    /// None.
    fn provider_name(&self) -> &str;
}

/// Local filesystem storage provider.
///
/// # Expected Behavior
///
/// Stores files in a configurable directory (env `STORAGE_PATH`, default
/// `./uploads`). Each file is stored at `<base_path>/<path>` and
/// accompanied by a sidecar `<path>.meta` JSON file containing the
/// content type and size. Parent directories are created as needed.
///
/// # Errors
///
/// Returns `AppError::IoError` for filesystem operations.
///
/// # Side Effects
///
/// - Creates directories on the filesystem.
/// - Reads and writes files on the filesystem.
pub struct LocalStorageProvider {
    base_path: PathBuf,
}

impl LocalStorageProvider {
    /// Create a new local storage provider rooted at `base_path`.
    ///
    /// # Expected Behavior
    ///
    /// Creates the base directory and any parent directories if they do
    /// not exist. Returns the provider instance.
    ///
    /// # Errors
    ///
    /// Panics if the directory cannot be created (e.g., permission denied).
    ///
    /// # Side Effects
    ///
    /// - Creates the base directory on the filesystem.
    pub fn new(base_path: &str) -> Self {
        let base = PathBuf::from(base_path);
        Self { base_path: base }
    }

    /// Resolve a storage path to an absolute filesystem path.
    ///
    /// # Expected Behavior
    ///
    /// Joins the base path with the relative storage path. No path
    /// traversal validation is performed (assumes trusted input).
    ///
    /// # Errors
    ///
    /// None. Always returns a valid `PathBuf`.
    ///
    /// # Side Effects
    ///
    /// None.
    fn resolve_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }

    /// Resolve the sidecar metadata file path for a given storage path.
    ///
    /// # Expected Behavior
    ///
    /// Appends `.meta` to the resolved file path.
    ///
    /// # Errors
    ///
    /// None. Always returns a valid `PathBuf`.
    ///
    /// # Side Effects
    ///
    /// None.
    fn resolve_meta_path(&self, path: &str) -> PathBuf {
        self.resolve_path(path).with_extension("meta")
    }
}

#[async_trait]
impl StorageProvider for LocalStorageProvider {
    async fn put_object(
        &self,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<(), AppError> {
        let file_path = self.resolve_path(path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::StorageError(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        fs::write(&file_path, data).await.map_err(|e| {
            AppError::StorageError(format!(
                "Failed to write file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let meta_path = self.resolve_meta_path(path);
        let meta = json!({
            "content_type": content_type,
            "size": data.len(),
        });
        let meta_str = serde_json::to_string_pretty(&meta).unwrap_or_default();
        fs::write(&meta_path, meta_str).await.map_err(|e| {
            AppError::StorageError(format!(
                "Failed to write meta file {}: {}",
                meta_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    async fn get_object(&self, path: &str) -> Result<Vec<u8>, AppError> {
        let file_path = self.resolve_path(path);
        fs::read(&file_path).await.map_err(|e| {
            AppError::StorageError(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })
    }

    async fn delete_object(&self, path: &str) -> Result<(), AppError> {
        let file_path = self.resolve_path(path);
        let meta_path = self.resolve_meta_path(path);

        if file_path.exists() {
            fs::remove_file(&file_path).await.map_err(|e| {
                AppError::StorageError(format!(
                    "Failed to delete file {}: {}",
                    file_path.display(),
                    e
                ))
            })?;
        }

        if meta_path.exists() {
            let _ = fs::remove_file(&meta_path).await;
        }

        Ok(())
    }

    async fn presign_get(&self, path: &str, _expiration: u64) -> Result<String, AppError> {
        Ok(format!(
            "/api/media/{}",
            path.split('/').next_back().unwrap_or(path)
        ))
    }

    async fn health_check(&self) -> Result<(), AppError> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).await.map_err(|e| {
                AppError::StorageError(format!("Failed to create storage directory: {e}"))
            })?;
        }

        let test_file = self.base_path.join(".health_check");
        fs::write(&test_file, b"health")
            .await
            .map_err(|e| AppError::StorageError(format!("Storage directory not writable: {e}")))?;
        let _ = fs::remove_file(&test_file).await;

        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "local"
    }
}

/// S3-compatible storage provider using the `rust-s3` crate.
///
/// # Expected Behavior
///
/// Stores files in an S3-compatible bucket (AWS S3, `MinIO`, etc.).
/// Configuration is read from environment variables: `S3_BUCKET_NAME`,
/// `S3_REGION`, `S3_ENDPOINT`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`. Supports
/// presigned URL generation with configurable expiration.
///
/// # Errors
///
/// Returns `AppError::S3Error` for any S3 API failures.
///
/// # Side Effects
///
/// - Makes network calls to the S3-compatible endpoint.
pub struct S3StorageProvider {
    bucket: Arc<Bucket>,
}

impl S3StorageProvider {
    /// Create a new S3 storage provider from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads `S3_BUCKET_NAME`, `S3_REGION`, `S3_ENDPOINT`, `S3_ACCESS_KEY`, and
    /// `S3_SECRET_KEY` from environment variables. Creates an S3 `Bucket`
    /// instance with the given configuration. If `S3_ENDPOINT` is set, uses
    /// a custom endpoint (for `MinIO` or other S3-compatible services).
    /// The bucket is configured with path-style addressing for `MinIO`
    /// compatibility.
    ///
    /// # Errors
    ///
    /// Panics if the bucket name is empty or if S3 credentials/region
    /// configuration is invalid.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables.
    /// - No network calls during construction (lazy client).
    pub fn from_env() -> Self {
        let bucket_name = std::env::var("S3_BUCKET_NAME")
            .expect("S3_BUCKET_NAME must be set when STORAGE_PROVIDER=s3");
        let region_str = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_default();
        let access_key = std::env::var("S3_ACCESS_KEY")
            .expect("S3_ACCESS_KEY must be set when STORAGE_PROVIDER=s3");
        let secret_key = std::env::var("S3_SECRET_KEY")
            .expect("S3_SECRET_KEY must be set when STORAGE_PROVIDER=s3");

        let region = if endpoint.is_empty() {
            region_str.parse().expect("Failed to parse S3 region")
        } else {
            Region::Custom {
                region: region_str,
                endpoint,
            }
        };

        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
            .expect("Failed to create S3 credentials");

        let mut bucket =
            Bucket::new(&bucket_name, region, credentials).expect("Failed to create S3 bucket");

        bucket.set_path_style();

        Self {
            bucket: Arc::new(bucket),
        }
    }
}

#[async_trait]
impl StorageProvider for S3StorageProvider {
    async fn put_object(
        &self,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<(), AppError> {
        self.bucket
            .put_object_with_content_type(path, data, content_type)
            .await
            .map_err(|e| AppError::S3Error(format!("S3 put_object failed: {e}")))?;
        Ok(())
    }

    async fn get_object(&self, path: &str) -> Result<Vec<u8>, AppError> {
        let data = self
            .bucket
            .get_object(path)
            .await
            .map_err(|e| AppError::S3Error(format!("S3 get_object failed: {e}")))?;
        Ok(data.to_vec())
    }

    async fn delete_object(&self, path: &str) -> Result<(), AppError> {
        self.bucket
            .delete_object(path)
            .await
            .map_err(|e| AppError::S3Error(format!("S3 delete_object failed: {e}")))?;
        Ok(())
    }

    async fn presign_get(&self, path: &str, expiration: u64) -> Result<String, AppError> {
        let expiry_secs: u32 = expiration.try_into().unwrap_or(3600);
        let url = self
            .bucket
            .presign_get(path, expiry_secs, None)
            .map_err(|e| AppError::S3Error(format!("S3 presign_get failed: {e}")))?;
        Ok(url)
    }

    async fn health_check(&self) -> Result<(), AppError> {
        self.bucket
            .list("/".to_string(), None)
            .await
            .map_err(|e| AppError::S3Error(format!("S3 health check failed: {e}")))?;
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "s3"
    }
}
