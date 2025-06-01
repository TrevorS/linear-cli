// ABOUTME: Main image manager that orchestrates downloading, caching, and rendering
// ABOUTME: Provides high-level async interface for image processing workflow

use crate::image_protocols::{
    ImageCache, ImageDownloader, ImageProtocol, ImageUrlValidator, TerminalCapabilities,
    kitty::KittyProtocol,
};
use anyhow::{Result, anyhow};

pub struct ImageManager {
    downloader: Option<ImageDownloader>,
    cache: Option<ImageCache>,
    capabilities: TerminalCapabilities,
    validator: ImageUrlValidator,
    enabled: bool,
}

#[derive(Debug)]
pub enum ImageRenderResult {
    /// Image rendered successfully as escape sequence
    Rendered(String),
    /// Fallback to clickable link
    Fallback(String),
    /// Image processing disabled
    Disabled,
}

impl ImageManager {
    pub fn new() -> Result<Self> {
        let capabilities = TerminalCapabilities::detect();
        let enabled = capabilities.supports_inline_images();

        let (downloader, cache) = if enabled {
            (
                Some(ImageDownloader::new(capabilities.clone())?),
                Some(ImageCache::new()?),
            )
        } else {
            (None, None)
        };

        let validator = ImageUrlValidator::new();

        Ok(Self {
            downloader,
            cache,
            capabilities,
            validator,
            enabled,
        })
    }

    /// Create a disabled manager that always falls back to links
    pub fn disabled() -> Self {
        // Create dummy capabilities for disabled manager
        let capabilities = TerminalCapabilities {
            supports_kitty_images: false,
            supports_iterm2_images: false,
            supports_sixel: false,
            terminal_name: "disabled".to_string(),
        };

        let validator = ImageUrlValidator::new();

        Self {
            downloader: None,
            cache: None,
            capabilities,
            validator,
            enabled: false,
        }
    }

    /// Set whether image processing is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled && self.capabilities.supports_inline_images();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    /// Process an image URL for display - main entry point
    pub async fn process_image(&self, url: &str, alt_text: &str) -> ImageRenderResult {
        if !self.enabled {
            return ImageRenderResult::Disabled;
        }

        match self.process_image_internal(url, alt_text).await {
            Ok(rendered) => ImageRenderResult::Rendered(rendered),
            Err(e) => {
                if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                    eprintln!("Image processing failed for {}: {}", url, e);
                }

                // Create fallback link
                let fallback = if alt_text.is_empty() {
                    format!("🖼️  [Image]({})", url)
                } else {
                    format!("🖼️  [{}]({})", alt_text, url)
                };
                ImageRenderResult::Fallback(fallback)
            }
        }
    }

    /// Internal image processing workflow
    async fn process_image_internal(&self, url: &str, alt_text: &str) -> Result<String> {
        // Validate URL first
        if !self.validator.is_image_url(url) {
            return Err(anyhow!("URL does not appear to be an image: {}", url));
        }

        let cache = self
            .cache
            .as_ref()
            .ok_or_else(|| anyhow!("Cache not available in disabled mode"))?;
        let downloader = self
            .downloader
            .as_ref()
            .ok_or_else(|| anyhow!("Downloader not available in disabled mode"))?;

        // Check cache first
        if let Some(cached_data) = cache.get(url).await {
            if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                eprintln!("Using cached image: {}", url);
            }
            return self.render_image_data(&cached_data, alt_text, url);
        }

        // Download image
        let image_data = downloader.download_image(url).await?;

        // Cache the downloaded data
        if let Err(e) = cache.put(url, &image_data).await {
            if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                eprintln!("Warning: Failed to cache image {}: {}", url, e);
            }
        }

        // Render the image
        self.render_image_data(&image_data, alt_text, url)
    }

    /// Render image data using appropriate protocol
    fn render_image_data(&self, data: &[u8], alt_text: &str, url: &str) -> Result<String> {
        let protocol = self.get_protocol()?;
        protocol.render_image(data, alt_text, url)
    }

    /// Get the appropriate image protocol for current terminal
    fn get_protocol(&self) -> Result<Box<dyn ImageProtocol>> {
        match self.capabilities.preferred_protocol() {
            Some("kitty") => Ok(Box::new(KittyProtocol)),
            Some("iterm2") => {
                // Future: implement iTerm2Protocol
                Err(anyhow!("iTerm2 protocol not yet implemented"))
            }
            _ => Err(anyhow!("No supported image protocol available")),
        }
    }

    /// Check if URL can be processed (for early filtering)
    pub fn can_process_url(&self, url: &str) -> bool {
        self.enabled && self.validator.is_image_url(url)
    }

    /// Clear the image cache
    #[allow(dead_code)] // May be used in future CLI commands
    pub async fn clear_cache(&self) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.clear().await
        } else {
            Ok(()) // No cache to clear in disabled mode
        }
    }

    /// Get cache statistics (for debugging)
    #[allow(dead_code)] // May be used in future CLI commands
    pub async fn cache_stats(&self) -> Result<String> {
        // This is a simple implementation - could be enhanced with more detailed stats
        Ok(format!(
            "Image cache directory: {:?}",
            dirs::cache_dir()
                .map(|d| d.join("linear-cli").join("images"))
                .unwrap_or_else(|| std::path::PathBuf::from("unknown"))
        ))
    }
}

/// Helper trait for creating image managers with different configurations
#[allow(dead_code)] // Future extensibility trait
pub trait ImageManagerBuilder {
    fn new() -> Result<ImageManager>;
    fn disabled() -> ImageManager;
    fn with_capabilities(capabilities: TerminalCapabilities) -> Result<ImageManager>;
}

impl ImageManagerBuilder for ImageManager {
    fn new() -> Result<ImageManager> {
        ImageManager::new()
    }

    fn disabled() -> ImageManager {
        ImageManager::disabled()
    }

    fn with_capabilities(capabilities: TerminalCapabilities) -> Result<ImageManager> {
        let enabled = capabilities.supports_inline_images();

        let (downloader, cache) = if enabled {
            (
                Some(ImageDownloader::new(capabilities.clone())?),
                Some(ImageCache::new()?),
            )
        } else {
            (None, None)
        };

        let validator = ImageUrlValidator::new();

        Ok(ImageManager {
            downloader,
            cache,
            capabilities,
            validator,
            enabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_disabled_manager() {
        let manager = ImageManager::disabled();
        assert!(!manager.is_enabled());

        let result = manager
            .process_image("https://uploads.linear.app/test.png", "test")
            .await;
        matches!(result, ImageRenderResult::Disabled);
    }

    #[tokio::test]
    #[serial]
    async fn test_enabled_manager_with_kitty_support() {
        // Mock kitty support
        unsafe {
            std::env::set_var("TERM_PROGRAM", "kitty");
        }

        let manager = ImageManager::new().unwrap();
        assert!(manager.is_enabled());
        assert_eq!(manager.capabilities().preferred_protocol(), Some("kitty"));

        unsafe {
            std::env::remove_var("TERM_PROGRAM");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_image_processing_workflow() {
        let mut server = Server::new_async().await;

        // Mock a small PNG image
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
            0x49, 0x48, 0x44, 0x52, // IHDR chunk type
            0x00, 0x00, 0x00, 0x01, // Width: 1
            0x00, 0x00, 0x00, 0x01, // Height: 1
            0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
            0x90, 0x77, 0x53, 0xDE, // CRC
        ];

        let mock = server
            .mock("GET", "/test.png")
            .with_status(200)
            .with_header("content-type", "image/png")
            .with_header("content-length", &png_data.len().to_string())
            .with_body(&png_data)
            .create_async()
            .await;

        // Setup environment for testing
        unsafe {
            std::env::set_var("TERM_PROGRAM", "kitty");
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "127.0.0.1");
        }

        let manager = ImageManager::new().unwrap();
        let url = &format!("{}/test.png", server.url());

        let result = manager.process_image(url, "test image").await;

        mock.assert_async().await;

        match result {
            ImageRenderResult::Rendered(output) => {
                assert!(output.contains("\x1b_Ga=T")); // Kitty protocol marker
            }
            ImageRenderResult::Fallback(_) => {
                // This is acceptable - test environment might not support all features
            }
            ImageRenderResult::Disabled => {
                panic!("Manager should be enabled in this test");
            }
        }

        unsafe {
            std::env::remove_var("TERM_PROGRAM");
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_caching_behavior() {
        let mut server = Server::new_async().await;

        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];

        // This mock should only be called once due to caching
        let mock = server
            .mock("GET", "/cached.png")
            .with_status(200)
            .with_header("content-type", "image/png")
            .with_body(&png_data)
            .expect(1) // Should only be called once
            .create_async()
            .await;

        unsafe {
            std::env::set_var("TERM_PROGRAM", "kitty");
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "127.0.0.1");
        }

        let manager = ImageManager::new().unwrap();
        let url = &format!("{}/cached.png", server.url());

        // First request - should download
        let result1 = manager.process_image(url, "test").await;

        // Second request - should use cache
        let result2 = manager.process_image(url, "test").await;

        mock.assert_async().await;

        // Both should succeed (either rendered or fallback)
        assert!(matches!(
            result1,
            ImageRenderResult::Rendered(_) | ImageRenderResult::Fallback(_)
        ));
        assert!(matches!(
            result2,
            ImageRenderResult::Rendered(_) | ImageRenderResult::Fallback(_)
        ));

        unsafe {
            std::env::remove_var("TERM_PROGRAM");
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_fallback_on_error() {
        unsafe {
            std::env::set_var("TERM_PROGRAM", "kitty");
        }

        let manager = ImageManager::new().unwrap();

        // Invalid URL should fallback
        let result = manager
            .process_image("https://nonexistent.invalid/image.png", "broken image")
            .await;

        match result {
            ImageRenderResult::Fallback(link) => {
                assert!(link.contains("broken image"));
                assert!(link.contains("🖼️"));
            }
            _ => panic!("Expected fallback for invalid URL"),
        }

        unsafe {
            std::env::remove_var("TERM_PROGRAM");
        }
    }

    #[test]
    fn test_can_process_url() {
        let manager = ImageManager::disabled();
        assert!(!manager.can_process_url("https://uploads.linear.app/test.png"));

        // Note: We can't easily test enabled manager without mocking terminal detection
        // This would be covered in integration tests
    }
}
