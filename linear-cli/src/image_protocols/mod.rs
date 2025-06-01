// ABOUTME: Image protocol implementations for terminal inline image display
// ABOUTME: Handles encoding and rendering for different terminal protocols

use anyhow::Result;

pub trait ImageProtocol {
    /// Render image data as terminal escape sequence
    fn render_image(&self, data: &[u8], alt_text: &str, url: &str) -> Result<String>;

    /// Maximum supported image size in bytes
    #[allow(dead_code)]
    fn max_size_bytes(&self) -> u64;

    /// Supported image MIME types
    #[allow(dead_code)]
    fn supported_formats(&self) -> &[&str];
}

pub mod cache;
pub mod detection;
pub mod downloader;
pub mod kitty;
pub mod manager;
pub mod url_validator;

pub use cache::ImageCache;
pub use detection::TerminalCapabilities;
pub use downloader::ImageDownloader;
pub use manager::{ImageManager, ImageRenderResult};
pub use url_validator::ImageUrlValidator;
