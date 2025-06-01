// ABOUTME: File-based caching system for downloaded images
// ABOUTME: Implements URL-based hashing and cache size management

use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ImageCache {
    cache_dir: PathBuf,
    max_age_seconds: u64,
    max_size_bytes: u64,
}

impl ImageCache {
    pub fn new() -> Result<Self> {
        let cache_dir = get_cache_directory()?;

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .map_err(|e| anyhow!("Failed to create cache directory {:?}: {}", cache_dir, e))?;

        let max_age_seconds = parse_duration_env("LINEAR_CLI_CACHE_TTL", 24 * 60 * 60); // 24 hours default
        let max_size_bytes = crate::image_protocols::url_validator::parse_size_env(
            "LINEAR_CLI_CACHE_SIZE",
            100 * 1024 * 1024,
        ); // 100MB default

        Ok(Self {
            cache_dir,
            max_age_seconds,
            max_size_bytes,
        })
    }

    pub async fn get(&self, url: &str) -> Option<Vec<u8>> {
        let cache_path = self.get_cache_path(url);

        // Check if cache file exists and is not expired
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let age_seconds = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        - duration.as_secs();

                    if age_seconds < self.max_age_seconds {
                        // Cache hit - read and return
                        if let Ok(data) = fs::read(&cache_path) {
                            if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                                eprintln!("Cache hit: {}", url);
                            }
                            return Some(data);
                        }
                    }
                }
            }
        }

        if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
            eprintln!("Cache miss: {}", url);
        }
        None
    }

    pub async fn put(&self, url: &str, data: &[u8]) -> Result<()> {
        let cache_path = self.get_cache_path(url);

        // Ensure parent directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create cache subdirectory {:?}: {}", parent, e))?;
        }

        // Write to temporary file first (atomic operation)
        let temp_path = cache_path.with_extension("tmp");
        fs::write(&temp_path, data)
            .map_err(|e| anyhow!("Failed to write cache file {:?}: {}", temp_path, e))?;

        // Atomically move to final location
        fs::rename(&temp_path, &cache_path).map_err(|e| {
            anyhow!(
                "Failed to move cache file {:?} -> {:?}: {}",
                temp_path,
                cache_path,
                e
            )
        })?;

        if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
            eprintln!("Cached: {} -> {:?}", url, cache_path);
        }

        // Trigger cleanup in background (non-blocking)
        let self_clone = Self {
            cache_dir: self.cache_dir.clone(),
            max_age_seconds: self.max_age_seconds,
            max_size_bytes: self.max_size_bytes,
        };
        tokio::spawn(async move {
            let _ = self_clone.cleanup_if_needed().await;
        });

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).map_err(|e| {
                anyhow!(
                    "Failed to clear cache directory {:?}: {}",
                    self.cache_dir,
                    e
                )
            })?;
        }
        Ok(())
    }

    fn get_cache_path(&self, url: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        // Use first 2 chars for subdirectory (reduces directory size)
        let subdir = &hash[..2];
        let filename = &hash[2..];

        self.cache_dir.join(subdir).join(filename)
    }

    async fn cleanup_if_needed(&self) -> Result<()> {
        if let Err(e) = self.cleanup().await {
            if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                eprintln!("Cache cleanup error: {}", e);
            }
        }
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        let total_size = self.calculate_cache_size().await?;

        if total_size <= self.max_size_bytes {
            return Ok(());
        }

        if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
            eprintln!(
                "Cache size {} bytes exceeds limit {} bytes, cleaning up...",
                total_size, self.max_size_bytes
            );
        }

        // Get all cache files with their access times
        let mut files = self.collect_cache_files().await?;

        // Sort by access time (oldest first)
        files.sort_by_key(|(_, _, access_time)| *access_time);

        let mut current_size = total_size;
        let target_size = self.max_size_bytes * 80 / 100; // Clean down to 80% of limit

        for (path, size, _) in files {
            if current_size <= target_size {
                break;
            }

            if let Err(e) = fs::remove_file(&path) {
                if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                    eprintln!("Failed to remove cache file {:?}: {}", path, e);
                }
            } else {
                current_size = current_size.saturating_sub(size);
                if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                    eprintln!("Removed cache file: {:?}", path);
                }
            }
        }

        Ok(())
    }

    async fn calculate_cache_size(&self) -> Result<u64> {
        let mut total_size = 0;
        self.walk_cache_dir(&self.cache_dir, &mut |metadata| {
            total_size += metadata.len();
        })?;
        Ok(total_size)
    }

    async fn collect_cache_files(&self) -> Result<Vec<(PathBuf, u64, SystemTime)>> {
        let mut files = Vec::new();
        self.walk_cache_dir_with_paths(&self.cache_dir, &mut |path, metadata| {
            if let Ok(access_time) = metadata.accessed().or_else(|_| metadata.modified()) {
                files.push((path.to_path_buf(), metadata.len(), access_time));
            }
        })?;
        Ok(files)
    }

    fn walk_cache_dir<F>(&self, dir: &Path, visitor: &mut F) -> Result<()>
    where
        F: FnMut(&fs::Metadata),
    {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_cache_dir(&path, visitor)?;
            } else if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    visitor(&metadata);
                }
            }
        }

        Ok(())
    }

    fn walk_cache_dir_with_paths<F>(&self, dir: &Path, visitor: &mut F) -> Result<()>
    where
        F: FnMut(&Path, &fs::Metadata),
    {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_cache_dir_with_paths(&path, visitor)?;
            } else if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    visitor(&path, &metadata);
                }
            }
        }

        Ok(())
    }
}

fn get_cache_directory() -> Result<PathBuf> {
    if let Ok(custom_dir) = std::env::var("LINEAR_CLI_IMAGE_CACHE") {
        return Ok(PathBuf::from(custom_dir));
    }

    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow!("Cannot determine cache directory"))?
        .join("linear-cli")
        .join("images");

    Ok(cache_dir)
}

fn parse_duration_env(env_var: &str, default_seconds: u64) -> u64 {
    let Ok(duration_str) = std::env::var(env_var) else {
        return default_seconds;
    };

    let duration_str = duration_str.to_lowercase();

    let (number_part, multiplier) = if duration_str.ends_with("h") {
        (duration_str.trim_end_matches("h"), 3600)
    } else if duration_str.ends_with("m") {
        (duration_str.trim_end_matches("m"), 60)
    } else if duration_str.ends_with("d") {
        (duration_str.trim_end_matches("d"), 24 * 3600)
    } else {
        (duration_str.as_str(), 1)
    };

    // Only apply multiplier if parsing succeeds
    if let Ok(num) = number_part.parse::<u64>() {
        num * multiplier
    } else {
        default_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_put_get() {
        let temp_dir = TempDir::new().unwrap();
        unsafe {
            std::env::set_var("LINEAR_CLI_IMAGE_CACHE", temp_dir.path());
        }

        let cache = ImageCache::new().unwrap();
        let test_url = "https://example.com/test.png";
        let test_data = b"fake image data";

        // Cache miss initially
        assert!(cache.get(test_url).await.is_none());

        // Put data in cache
        cache.put(test_url, test_data).await.unwrap();

        // Cache hit now
        let cached_data = cache.get(test_url).await.unwrap();
        assert_eq!(cached_data, test_data);

        unsafe {
            std::env::remove_var("LINEAR_CLI_IMAGE_CACHE");
        }
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        unsafe {
            std::env::set_var("LINEAR_CLI_IMAGE_CACHE", temp_dir.path());
            std::env::set_var("LINEAR_CLI_CACHE_TTL", "1"); // 1 second TTL
        }

        let cache = ImageCache::new().unwrap();
        let test_url = "https://example.com/expire.png";
        let test_data = b"expiring data";

        // Put data in cache
        cache.put(test_url, test_data).await.unwrap();

        // Should be available immediately
        assert!(cache.get(test_url).await.is_some());

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be expired now
        assert!(cache.get(test_url).await.is_none());

        unsafe {
            std::env::remove_var("LINEAR_CLI_IMAGE_CACHE");
            std::env::remove_var("LINEAR_CLI_CACHE_TTL");
        }
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        unsafe {
            std::env::set_var("LINEAR_CLI_IMAGE_CACHE", temp_dir.path());
        }

        let cache = ImageCache::new().unwrap();
        let test_url = "https://example.com/clear.png";
        let test_data = b"data to clear";

        // Put data in cache
        cache.put(test_url, test_data).await.unwrap();
        assert!(cache.get(test_url).await.is_some());

        // Clear cache
        cache.clear().await.unwrap();

        // Should be gone
        assert!(cache.get(test_url).await.is_none());

        unsafe {
            std::env::remove_var("LINEAR_CLI_IMAGE_CACHE");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_duration_parsing() {
        // Ensure clean state
        unsafe {
            std::env::remove_var("TEST_DURATION");
        }

        assert_eq!(parse_duration_env("NONEXISTENT", 3600), 3600);

        unsafe {
            std::env::set_var("TEST_DURATION", "2h");
        }
        assert_eq!(parse_duration_env("TEST_DURATION", 3600), 2 * 3600);

        unsafe {
            std::env::set_var("TEST_DURATION", "30m");
        }
        assert_eq!(parse_duration_env("TEST_DURATION", 3600), 30 * 60);

        unsafe {
            std::env::set_var("TEST_DURATION", "7d");
        }
        assert_eq!(parse_duration_env("TEST_DURATION", 3600), 7 * 24 * 3600);

        unsafe {
            std::env::set_var("TEST_DURATION", "invalid");
        }
        let result = parse_duration_env("TEST_DURATION", 3600);
        if result != 3600 {
            eprintln!(
                "Expected 3600, got {}. ENV var value: {:?}",
                result,
                std::env::var("TEST_DURATION")
            );
        }
        assert_eq!(result, 3600);

        unsafe {
            std::env::remove_var("TEST_DURATION");
        }
    }
}
