// ABOUTME: HTTP client for downloading images with security and error handling
// ABOUTME: Implements timeouts, size limits, and content validation

use crate::image_protocols::{ImageUrlValidator, TerminalCapabilities};
use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log;
use reqwest::Client;
use std::time::Duration;

pub struct ImageDownloader {
    client: Client,
    validator: ImageUrlValidator,
    capabilities: TerminalCapabilities,
    show_progress: bool,
}

impl ImageDownloader {
    pub fn new(capabilities: TerminalCapabilities) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30)) // Increased timeout for larger images
            .user_agent("linear-cli/1.0")
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        let validator = ImageUrlValidator::new();

        // Enable progress for TTY and when not in quiet mode
        let show_progress = {
            use std::io::IsTerminal;
            std::io::stderr().is_terminal() && std::env::var("LINEAR_CLI_QUIET").is_err()
        };

        Ok(Self {
            client,
            validator,
            capabilities,
            show_progress,
        })
    }

    pub fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    pub async fn download_image(&self, url: &str) -> Result<Vec<u8>> {
        // Validate URL first
        let validated_url = self.validator.validate_image_url(url)?;

        if !self.validator.is_image_url(url) {
            return Err(anyhow!("URL does not appear to be an image: {}", url));
        }

        // Make HTTP request
        let response = self
            .client
            .get(validated_url.as_str())
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed for {}: {}", url, e))?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP request failed with status {}: {}",
                response.status(),
                url
            ));
        }

        // Validate content type
        self.validate_content_type(&response, url)?;

        // Check content length
        self.validate_content_length(&response, url)?;

        // Download body with size limit
        let bytes = self.download_body_with_limit(response, url).await?;

        // Validate image format
        self.validate_image_format(&bytes, url)?;

        Ok(bytes)
    }

    fn validate_content_type(&self, response: &reqwest::Response, url: &str) -> Result<()> {
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("unknown");

        if content_type.starts_with("image/") {
            return Ok(());
        }

        // Linear uploads might not have proper content-type
        if self.validator.is_linear_upload_url(url) {
            // Allow, but log warning in verbose mode
            log::debug!(
                "Warning: Linear upload URL missing image content-type: {}",
                url
            );
            return Ok(());
        }

        Err(anyhow!(
            "URL does not serve image content (content-type: {}): {}",
            content_type,
            url
        ))
    }

    fn validate_content_length(&self, response: &reqwest::Response, url: &str) -> Result<()> {
        if let Some(content_length) = response.content_length() {
            let max_size = self.validator.max_size_bytes();
            if content_length > max_size {
                return Err(anyhow!(
                    "Image too large: {} bytes (max: {} bytes): {}",
                    content_length,
                    max_size,
                    url
                ));
            }
        }
        Ok(())
    }

    async fn download_body_with_limit(
        &self,
        response: reqwest::Response,
        url: &str,
    ) -> Result<Vec<u8>> {
        let max_size = self.validator.max_size_bytes();
        let content_length = response.content_length();

        // Create progress bar if enabled and we know the content length
        let progress_bar = if self.show_progress && content_length.is_some() {
            let pb = ProgressBar::new(content_length.unwrap());
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} [{bar:25.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
                    .unwrap()
                    .progress_chars("=>-"),
            );

            // Extract filename from URL for display
            let filename = url.split('/').last().unwrap_or("image");
            pb.set_message(format!("Downloading {}", filename));
            Some(pb)
        } else if self.show_progress {
            // Spinner for unknown size
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg} {bytes}")
                    .unwrap(),
            );

            let filename = url.split('/').last().unwrap_or("image");
            pb.set_message(format!("Downloading {}", filename));
            Some(pb)
        } else {
            None
        };

        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow!("Failed to read response body: {}", e))?;

            bytes.extend_from_slice(&chunk);

            // Update progress bar
            if let Some(ref pb) = progress_bar {
                pb.set_position(bytes.len() as u64);
            }

            // Check size limit during download
            if bytes.len() as u64 > max_size {
                if let Some(pb) = progress_bar {
                    pb.finish_with_message("Download failed: size limit exceeded");
                }
                return Err(anyhow!(
                    "Image exceeded size limit during download: {} bytes (max: {}): {}",
                    bytes.len(),
                    max_size,
                    url
                ));
            }
        }

        // Finish progress bar
        if let Some(pb) = progress_bar {
            let filename = url.split('/').last().unwrap_or("image");
            pb.finish_with_message(format!(
                "Downloaded {} ({})",
                filename,
                format_bytes(bytes.len())
            ));
        }

        Ok(bytes)
    }

    fn validate_image_format(&self, bytes: &[u8], url: &str) -> Result<()> {
        if bytes.len() < 8 {
            return Err(anyhow!("File too small to be a valid image: {}", url));
        }

        // Check for common image signatures
        let is_valid_image =
            // PNG: 89 50 4E 47 0D 0A 1A 0A
            bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) ||
            // JPEG: FF D8 FF
            bytes.starts_with(&[0xFF, 0xD8, 0xFF]) ||
            // GIF: GIF87a or GIF89a
            bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") ||
            // WebP: RIFF....WEBP
            (bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP") ||
            // BMP: BM
            bytes.starts_with(b"BM") ||
            // TIFF: II or MM
            bytes.starts_with(b"II") || bytes.starts_with(b"MM");

        if !is_valid_image {
            return Err(anyhow!("File does not appear to be a valid image: {}", url));
        }

        Ok(())
    }

    #[allow(dead_code)] // May be used in future CLI enhancements
    pub fn can_handle_url(&self, url: &str) -> bool {
        self.capabilities.supports_inline_images() && self.validator.is_image_url(url)
    }
}

/// Format bytes in a human-readable way
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_successful_image_download() {
        let mut server = Server::new_async().await;

        // Mock a PNG image response
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];

        let mock = server
            .mock("GET", "/test.png")
            .with_status(200)
            .with_header("content-type", "image/png")
            .with_header("content-length", &png_data.len().to_string())
            .with_body(&png_data)
            .create_async()
            .await;

        // Create downloader that allows localhost
        unsafe {
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "127.0.0.1");
        }
        let capabilities = TerminalCapabilities::detect(); // This will be mocked in real tests
        let downloader = ImageDownloader::new(capabilities).unwrap();

        let url = &format!("{}/test.png", server.url());
        let result = downloader.download_image(url).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), png_data);

        unsafe {
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_oversized_image_rejection() {
        let mut server = Server::new_async().await;

        // Mock an oversized response with correct content-length
        let large_data = vec![0xFF; 999999]; // Actually large file

        let mock = server
            .mock("GET", "/large.png")
            .with_status(200)
            .with_header("content-type", "image/png")
            .with_header("content-length", &large_data.len().to_string())
            .with_body(&large_data)
            .create_async()
            .await;

        unsafe {
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "127.0.0.1");
            std::env::set_var("LINEAR_CLI_MAX_IMAGE_SIZE", "1KB"); // Very small limit
        }

        let capabilities = TerminalCapabilities::detect();
        let downloader = ImageDownloader::new(capabilities).unwrap();

        let url = &format!("{}/large.png", server.url());
        let result = downloader.download_image(url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));

        unsafe {
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
            std::env::remove_var("LINEAR_CLI_MAX_IMAGE_SIZE");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_invalid_content_type_rejection() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/fake.png")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html>Not an image</html>")
            .create_async()
            .await;

        unsafe {
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "127.0.0.1");
        }
        let capabilities = TerminalCapabilities::detect();
        let downloader = ImageDownloader::new(capabilities).unwrap();

        let url = &format!("{}/fake.png", server.url());
        let result = downloader.download_image(url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not serve image content"));

        unsafe {
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_http_error_handling() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/missing.png")
            .with_status(404)
            .create_async()
            .await;

        unsafe {
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "127.0.0.1");
        }
        let capabilities = TerminalCapabilities::detect();
        let downloader = ImageDownloader::new(capabilities).unwrap();

        let url = &format!("{}/missing.png", server.url());
        let result = downloader.download_image(url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));

        unsafe {
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
        }
    }
}
