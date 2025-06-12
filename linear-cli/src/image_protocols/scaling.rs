// ABOUTME: Terminal-aware image scaling and dimension management
// ABOUTME: Automatically resizes images to fit within terminal bounds while preserving aspect ratios

use anyhow::{anyhow, Result};
use crossterm::terminal::size as terminal_size;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use log;

#[derive(Debug, Clone)]
pub struct TerminalDimensions {
    pub width: u16,
    pub height: u16,
    pub char_width: u16,  // Approximate character width in pixels
    pub char_height: u16, // Approximate character height in pixels
}

#[derive(Debug, Clone)]
pub struct ScalingConfig {
    pub max_width_chars: Option<u16>,
    pub max_height_chars: Option<u16>,
    pub preserve_aspect_ratio: bool,
    pub quality: FilterType,
    pub margin_chars: u16, // Leave margin around image
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            max_width_chars: None,
            max_height_chars: None,
            preserve_aspect_ratio: true,
            quality: FilterType::Lanczos3, // High quality scaling
            margin_chars: 2,               // Leave 2 characters of margin
        }
    }
}

pub struct ImageScaler {
    terminal_dims: Option<TerminalDimensions>,
    config: ScalingConfig,
}

impl ImageScaler {
    pub fn new() -> Result<Self> {
        let terminal_dims = Self::detect_terminal_dimensions().ok();
        Ok(Self {
            terminal_dims,
            config: ScalingConfig::default(),
        })
    }

    pub fn with_config(config: ScalingConfig) -> Result<Self> {
        let terminal_dims = Self::detect_terminal_dimensions().ok();
        Ok(Self {
            terminal_dims,
            config,
        })
    }

    /// Detect current terminal dimensions
    pub fn detect_terminal_dimensions() -> Result<TerminalDimensions> {
        let (width, height) =
            terminal_size().map_err(|e| anyhow!("Failed to get terminal size: {}", e))?;

        // Estimate character pixel dimensions
        // Most terminals use approximately 8x16 pixel characters
        // These are rough estimates that work well for scaling calculations
        let char_width = 8;
        let char_height = 16;

        Ok(TerminalDimensions {
            width,
            height,
            char_width,
            char_height,
        })
    }

    /// Scale image data to fit terminal dimensions
    pub fn scale_image(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Load image from bytes
        let mut img = image::load_from_memory(data)
            .map_err(|e| anyhow!("Failed to load image for scaling: {}", e))?;

        // Determine if scaling is needed
        if let Some(target_dims) = self.calculate_target_dimensions(&img) {
            log::debug!(
                "Scaling image from {}x{} to {}x{}",
                img.width(),
                img.height(),
                target_dims.0,
                target_dims.1
            );

            // Resize the image
            img = img.resize(target_dims.0, target_dims.1, self.config.quality);
        } else {
            log::debug!("Image scaling skipped - no terminal dimensions or already optimal size");
        }

        // Convert back to bytes
        self.image_to_bytes(&img)
    }

    /// Calculate target dimensions for scaling
    fn calculate_target_dimensions(&self, img: &DynamicImage) -> Option<(u32, u32)> {
        let terminal_dims = self.terminal_dims.as_ref()?;

        // Calculate available space in characters (with margin)
        let available_width_chars = terminal_dims
            .width
            .saturating_sub(self.config.margin_chars * 2);
        let available_height_chars = terminal_dims
            .height
            .saturating_sub(self.config.margin_chars * 2);

        // Apply user-specified limits if any
        let max_width_chars = self
            .config
            .max_width_chars
            .map(|w| w.min(available_width_chars))
            .unwrap_or(available_width_chars);
        let max_height_chars = self
            .config
            .max_height_chars
            .map(|h| h.min(available_height_chars))
            .unwrap_or(available_height_chars);

        // Convert character dimensions to approximate pixels
        let max_width_pixels = max_width_chars as u32 * terminal_dims.char_width as u32;
        let max_height_pixels = max_height_chars as u32 * terminal_dims.char_height as u32;

        let img_width = img.width();
        let img_height = img.height();

        // Check if scaling is needed
        if img_width <= max_width_pixels && img_height <= max_height_pixels {
            return None; // No scaling needed
        }

        if !self.config.preserve_aspect_ratio {
            return Some((max_width_pixels, max_height_pixels));
        }

        // Calculate scaled dimensions preserving aspect ratio
        let width_ratio = max_width_pixels as f64 / img_width as f64;
        let height_ratio = max_height_pixels as f64 / img_height as f64;

        // Use the smaller ratio to ensure image fits in both dimensions
        let scale_ratio = width_ratio.min(height_ratio);

        let target_width = (img_width as f64 * scale_ratio) as u32;
        let target_height = (img_height as f64 * scale_ratio) as u32;

        Some((target_width, target_height))
    }

    /// Convert DynamicImage back to bytes
    fn image_to_bytes(&self, img: &DynamicImage) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Use PNG for lossless quality, JPEG for smaller size
        // PNG is generally better for terminal display due to no compression artifacts
        img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to encode scaled image: {}", e))?;

        Ok(buffer)
    }

    /// Get image metadata without fully loading the image
    pub fn get_image_metadata(&self, data: &[u8]) -> Result<ImageMetadata> {
        // Use image crate's ability to read just the headers
        let reader = image::ImageReader::new(std::io::Cursor::new(data))
            .with_guessed_format()
            .map_err(|e| anyhow!("Failed to create image reader: {}", e))?;

        let format = reader
            .format()
            .ok_or_else(|| anyhow!("Could not determine image format"))?;

        let (width, height) = reader
            .into_dimensions()
            .map_err(|e| anyhow!("Failed to read image dimensions: {}", e))?;

        Ok(ImageMetadata {
            width,
            height,
            format,
            size_bytes: data.len(),
        })
    }

    /// Check if an image needs scaling based on terminal dimensions
    pub fn needs_scaling(&self, metadata: &ImageMetadata) -> bool {
        if let Some(target_dims) = self.calculate_target_dimensions_from_metadata(metadata) {
            target_dims.0 != metadata.width || target_dims.1 != metadata.height
        } else {
            false
        }
    }

    fn calculate_target_dimensions_from_metadata(
        &self,
        metadata: &ImageMetadata,
    ) -> Option<(u32, u32)> {
        let terminal_dims = self.terminal_dims.as_ref()?;

        let available_width_chars = terminal_dims
            .width
            .saturating_sub(self.config.margin_chars * 2);
        let available_height_chars = terminal_dims
            .height
            .saturating_sub(self.config.margin_chars * 2);

        let max_width_chars = self
            .config
            .max_width_chars
            .map(|w| w.min(available_width_chars))
            .unwrap_or(available_width_chars);
        let max_height_chars = self
            .config
            .max_height_chars
            .map(|h| h.min(available_height_chars))
            .unwrap_or(available_height_chars);

        let max_width_pixels = max_width_chars as u32 * terminal_dims.char_width as u32;
        let max_height_pixels = max_height_chars as u32 * terminal_dims.char_height as u32;

        if metadata.width <= max_width_pixels && metadata.height <= max_height_pixels {
            return None;
        }

        if !self.config.preserve_aspect_ratio {
            return Some((max_width_pixels, max_height_pixels));
        }

        let width_ratio = max_width_pixels as f64 / metadata.width as f64;
        let height_ratio = max_height_pixels as f64 / metadata.height as f64;
        let scale_ratio = width_ratio.min(height_ratio);

        let target_width = (metadata.width as f64 * scale_ratio) as u32;
        let target_height = (metadata.height as f64 * scale_ratio) as u32;

        Some((target_width, target_height))
    }

    pub fn update_terminal_dimensions(&mut self) -> Result<()> {
        self.terminal_dims = Some(Self::detect_terminal_dimensions()?);
        Ok(())
    }

    pub fn get_terminal_dimensions(&self) -> Option<&TerminalDimensions> {
        self.terminal_dims.as_ref()
    }

    pub fn set_config(&mut self, config: ScalingConfig) {
        self.config = config;
    }
}

#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub size_bytes: usize,
}

impl ImageMetadata {
    pub fn format_name(&self) -> &'static str {
        match self.format {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::Gif => "GIF",
            ImageFormat::WebP => "WebP",
            ImageFormat::Tiff => "TIFF",
            ImageFormat::Bmp => "BMP",
            _ => "Unknown",
        }
    }

    pub fn dimensions_str(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    pub fn size_str(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaling_config_default() {
        let config = ScalingConfig::default();
        assert!(config.preserve_aspect_ratio);
        assert_eq!(config.margin_chars, 2);
        assert!(matches!(config.quality, FilterType::Lanczos3));
    }

    #[test]
    fn test_terminal_dimensions_creation() {
        let dims = TerminalDimensions {
            width: 80,
            height: 24,
            char_width: 8,
            char_height: 16,
        };

        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }

    #[test]
    fn test_image_metadata_display() {
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            format: ImageFormat::Png,
            size_bytes: 1024 * 1024, // 1MB
        };

        assert_eq!(metadata.format_name(), "PNG");
        assert_eq!(metadata.dimensions_str(), "1920x1080");
        assert_eq!(metadata.size_str(), "1.0 MB");
    }

    #[test]
    fn test_size_formatting() {
        let small = ImageMetadata {
            width: 100,
            height: 100,
            format: ImageFormat::Png,
            size_bytes: 512,
        };
        assert_eq!(small.size_str(), "512 B");

        let medium = ImageMetadata {
            width: 100,
            height: 100,
            format: ImageFormat::Png,
            size_bytes: 1536,
        };
        assert_eq!(medium.size_str(), "1.5 KB");

        let large = ImageMetadata {
            width: 100,
            height: 100,
            format: ImageFormat::Png,
            size_bytes: 2 * 1024 * 1024,
        };
        assert_eq!(large.size_str(), "2.0 MB");
    }

    #[test]
    fn test_aspect_ratio_calculation() {
        let mut config = ScalingConfig::default();
        config.margin_chars = 0; // No margins for easier testing

        let terminal_dims = TerminalDimensions {
            width: 80,  // 80 chars * 8 pixels = 640 pixels
            height: 24, // 24 chars * 16 pixels = 384 pixels
            char_width: 8,
            char_height: 16,
        };

        let scaler = ImageScaler {
            terminal_dims: Some(terminal_dims),
            config,
        };

        // Test image that's too wide
        let metadata = ImageMetadata {
            width: 1280, // Would need 160 chars width
            height: 800, // Would need 50 chars height
            format: ImageFormat::Png,
            size_bytes: 1000,
        };

        let target = scaler.calculate_target_dimensions_from_metadata(&metadata);
        assert!(target.is_some());

        if let Some((target_width, target_height)) = target {
            // Should scale down proportionally
            let original_ratio = metadata.width as f64 / metadata.height as f64;
            let target_ratio = target_width as f64 / target_height as f64;

            // Ratios should be approximately equal (within floating point precision)
            assert!((original_ratio - target_ratio).abs() < 0.01);

            // Should fit within terminal bounds
            assert!(target_width <= 640); // 80 chars * 8 pixels
            assert!(target_height <= 384); // 24 chars * 16 pixels
        }
    }
}
