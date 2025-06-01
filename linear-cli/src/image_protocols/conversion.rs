// ABOUTME: Image format conversion pipeline for terminal compatibility
// ABOUTME: Converts unsupported formats to terminal-friendly ones with optimization

use anyhow::{Result, anyhow};
use image::{DynamicImage, ImageEncoder, ImageFormat};
use log;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct ConversionConfig {
    pub target_format: TargetFormat,
    pub jpeg_quality: u8,
    pub png_compression: PngCompression,
    pub max_file_size_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetFormat {
    /// Always convert to PNG (lossless, good for graphics)
    Png,
    /// Always convert to JPEG (smaller size, good for photos)
    Jpeg,
    /// Smart choice based on image characteristics
    Auto,
}

#[derive(Debug, Clone, Copy)]
pub enum PngCompression {
    Fast,
    Default,
    Best,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            target_format: TargetFormat::Auto,
            jpeg_quality: 85, // Good balance of quality vs size
            png_compression: PngCompression::Default,
            max_file_size_bytes: 5 * 1024 * 1024, // 5MB limit
        }
    }
}

pub struct ImageConverter {
    config: ConversionConfig,
}

impl ImageConverter {
    pub fn new() -> Self {
        Self {
            config: ConversionConfig::default(),
        }
    }

    pub fn with_config(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// Convert image data to terminal-compatible format
    pub fn convert_image(
        &self,
        data: &[u8],
        original_format: Option<ImageFormat>,
    ) -> Result<Vec<u8>> {
        // Try to detect format if not provided
        let detected_format = original_format.or_else(|| self.detect_format(data));

        // Check if conversion is needed
        if let Some(format) = detected_format {
            if self.is_terminal_compatible(format) && data.len() <= self.config.max_file_size_bytes
            {
                log::debug!(
                    "Image format {} is already terminal-compatible, no conversion needed",
                    format_name(format)
                );
                return Ok(data.to_vec());
            }
        }

        // Load the image
        let img = image::load_from_memory(data)
            .map_err(|e| anyhow!("Failed to load image for conversion: {}", e))?;

        let format_str = detected_format.map(format_name).unwrap_or("unknown");
        log::debug!(
            "Converting image from {} to terminal-compatible format",
            format_str
        );

        // Determine target format
        let target_format = self.determine_target_format(&img, detected_format);

        // Convert the image
        self.encode_image(&img, target_format)
    }

    /// Detect image format from data
    fn detect_format(&self, data: &[u8]) -> Option<ImageFormat> {
        image::guess_format(data).ok()
    }

    /// Check if format is directly supported by terminals
    fn is_terminal_compatible(&self, format: ImageFormat) -> bool {
        matches!(
            format,
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif
        )
    }

    /// Determine the best target format for the image
    fn determine_target_format(
        &self,
        img: &DynamicImage,
        original_format: Option<ImageFormat>,
    ) -> ImageFormat {
        match self.config.target_format {
            TargetFormat::Png => ImageFormat::Png,
            TargetFormat::Jpeg => ImageFormat::Jpeg,
            TargetFormat::Auto => {
                // Smart format selection based on image characteristics
                self.auto_select_format(img, original_format)
            }
        }
    }

    /// Automatically select the best format for an image
    fn auto_select_format(
        &self,
        img: &DynamicImage,
        original_format: Option<ImageFormat>,
    ) -> ImageFormat {
        // If it's already a good format, keep it
        if let Some(format) = original_format {
            if self.is_terminal_compatible(format) {
                return format;
            }
        }

        // Analyze image characteristics
        let has_transparency = self.has_transparency(img);
        let is_photographic = self.is_photographic(img);

        // Decision logic:
        // - Images with transparency -> PNG
        // - Photographic images without transparency -> JPEG
        // - Graphics/screenshots -> PNG
        if has_transparency {
            ImageFormat::Png
        } else if is_photographic {
            ImageFormat::Jpeg
        } else {
            ImageFormat::Png
        }
    }

    /// Check if image has transparency
    fn has_transparency(&self, img: &DynamicImage) -> bool {
        match img {
            DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageRgba16(_)
            | DynamicImage::ImageRgba32F(_) => true,
            DynamicImage::ImageLumaA8(_) | DynamicImage::ImageLumaA16(_) => true,
            _ => false,
        }
    }

    /// Heuristic to determine if image is photographic
    fn is_photographic(&self, img: &DynamicImage) -> bool {
        // Simple heuristic: larger images with many colors are likely photographic
        let (width, height) = (img.width(), img.height());
        let pixel_count = width * height;

        // Images larger than 500x500 pixels are likely photos
        // This is a simple heuristic - could be made more sophisticated
        pixel_count > 500 * 500
    }

    /// Encode image to target format with optimization
    fn encode_image(&self, img: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        match format {
            ImageFormat::Png => {
                // Use the simpler write_to approach for PNG
                img.write_to(&mut cursor, ImageFormat::Png)
                    .map_err(|e| anyhow!("Failed to encode PNG: {}", e))?;
            }

            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut cursor,
                    self.config.jpeg_quality,
                );
                encoder
                    .write_image(
                        img.as_bytes(),
                        img.width(),
                        img.height(),
                        img.color().into(),
                    )
                    .map_err(|e| anyhow!("Failed to encode JPEG: {}", e))?;
            }

            ImageFormat::Gif => {
                // For GIF, we'll use the default encoder
                img.write_to(&mut cursor, format)
                    .map_err(|e| anyhow!("Failed to encode GIF: {}", e))?;
            }

            _ => {
                return Err(anyhow!("Unsupported target format: {:?}", format));
            }
        }

        // Check if the result is within size limits
        if buffer.len() > self.config.max_file_size_bytes {
            // Try to reduce quality/compression if possible
            if format == ImageFormat::Jpeg && self.config.jpeg_quality > 50 {
                log::debug!(
                    "Image too large ({}), retrying with lower JPEG quality",
                    format_size(buffer.len())
                );
                let mut reduced_config = self.config.clone();
                reduced_config.jpeg_quality = 50;
                let reduced_converter = ImageConverter::with_config(reduced_config);
                return reduced_converter.encode_image(img, format);
            } else {
                return Err(anyhow!(
                    "Converted image too large: {} exceeds limit of {}",
                    format_size(buffer.len()),
                    format_size(self.config.max_file_size_bytes)
                ));
            }
        }

        log::debug!(
            "Successfully converted image to {} ({})",
            format_name(format),
            format_size(buffer.len())
        );

        Ok(buffer)
    }

    /// Get conversion statistics for an image
    pub fn get_conversion_info(&self, data: &[u8]) -> Result<ConversionInfo> {
        let original_format = self.detect_format(data);
        let original_size = data.len();

        let needs_conversion = if let Some(format) = original_format {
            !self.is_terminal_compatible(format) || original_size > self.config.max_file_size_bytes
        } else {
            true // Unknown format needs conversion
        };

        if !needs_conversion {
            return Ok(ConversionInfo {
                original_format,
                original_size,
                target_format: original_format,
                needs_conversion: false,
                estimated_target_size: original_size,
            });
        }

        // Load image to determine target format
        let img = image::load_from_memory(data)
            .map_err(|e| anyhow!("Failed to load image for analysis: {}", e))?;

        let target_format = self.determine_target_format(&img, original_format);

        // Estimate target size (rough approximation)
        let estimated_target_size = self.estimate_converted_size(&img, target_format);

        Ok(ConversionInfo {
            original_format,
            original_size,
            target_format: Some(target_format),
            needs_conversion: true,
            estimated_target_size,
        })
    }

    /// Estimate the size of converted image
    fn estimate_converted_size(&self, img: &DynamicImage, target_format: ImageFormat) -> usize {
        let pixel_count = img.width() * img.height();

        match target_format {
            ImageFormat::Png => {
                // PNG: roughly 3-4 bytes per pixel for RGB, more for RGBA
                let bytes_per_pixel = if self.has_transparency(img) { 4 } else { 3 };
                (pixel_count * bytes_per_pixel) as usize
            }
            ImageFormat::Jpeg => {
                // JPEG: roughly 0.5-2 bytes per pixel depending on quality
                let quality_factor = self.config.jpeg_quality as f64 / 100.0;
                let bytes_per_pixel = 0.5 + (1.5 * quality_factor);
                (pixel_count as f64 * bytes_per_pixel) as usize
            }
            _ => {
                // Fallback estimate
                (pixel_count * 3) as usize
            }
        }
    }

    pub fn update_config(&mut self, config: ConversionConfig) {
        self.config = config;
    }
}

#[derive(Debug)]
pub struct ConversionInfo {
    pub original_format: Option<ImageFormat>,
    pub original_size: usize,
    pub target_format: Option<ImageFormat>,
    pub needs_conversion: bool,
    pub estimated_target_size: usize,
}

impl ConversionInfo {
    pub fn original_format_name(&self) -> &str {
        self.original_format.map(format_name).unwrap_or("Unknown")
    }

    pub fn target_format_name(&self) -> &str {
        self.target_format.map(format_name).unwrap_or("Unknown")
    }

    pub fn size_reduction_percent(&self) -> Option<f64> {
        if self.needs_conversion && self.original_size > 0 {
            let reduction = (self.original_size as f64 - self.estimated_target_size as f64)
                / self.original_size as f64;
            Some(reduction * 100.0)
        } else {
            None
        }
    }
}

fn format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Gif => "GIF",
        ImageFormat::WebP => "WebP",
        ImageFormat::Tiff => "TIFF",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Tga => "TGA",
        ImageFormat::Ico => "ICO",
        ImageFormat::Hdr => "HDR",
        ImageFormat::OpenExr => "OpenEXR",
        ImageFormat::Farbfeld => "Farbfeld",
        ImageFormat::Avif => "AVIF",
        _ => "Unknown",
    }
}

fn format_size(bytes: usize) -> String {
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

    #[test]
    fn test_conversion_config_default() {
        let config = ConversionConfig::default();
        assert_eq!(config.target_format, TargetFormat::Auto);
        assert_eq!(config.jpeg_quality, 85);
        assert_eq!(config.max_file_size_bytes, 5 * 1024 * 1024);
    }

    #[test]
    fn test_format_compatibility_check() {
        let converter = ImageConverter::new();

        assert!(converter.is_terminal_compatible(ImageFormat::Png));
        assert!(converter.is_terminal_compatible(ImageFormat::Jpeg));
        assert!(converter.is_terminal_compatible(ImageFormat::Gif));

        assert!(!converter.is_terminal_compatible(ImageFormat::Tiff));
        assert!(!converter.is_terminal_compatible(ImageFormat::Bmp));
        assert!(!converter.is_terminal_compatible(ImageFormat::WebP));
    }

    #[test]
    fn test_format_detection() {
        let converter = ImageConverter::new();

        // PNG signature
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(converter.detect_format(&png_data), Some(ImageFormat::Png));

        // JPEG signature
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(converter.detect_format(&jpeg_data), Some(ImageFormat::Jpeg));

        // Invalid data
        let invalid_data = vec![0x00, 0x01, 0x02];
        assert_eq!(converter.detect_format(&invalid_data), None);
    }

    #[test]
    fn test_target_format_selection() {
        let converter = ImageConverter::new();

        // Test PNG preference
        let mut config = ConversionConfig::default();
        config.target_format = TargetFormat::Png;
        let png_converter = ImageConverter::with_config(config);

        // Create a dummy image for testing
        let img = DynamicImage::new_rgb8(100, 100);
        assert_eq!(
            png_converter.determine_target_format(&img, None),
            ImageFormat::Png
        );

        // Test JPEG preference
        let mut config = ConversionConfig::default();
        config.target_format = TargetFormat::Jpeg;
        let jpeg_converter = ImageConverter::with_config(config);
        assert_eq!(
            jpeg_converter.determine_target_format(&img, None),
            ImageFormat::Jpeg
        );
    }

    #[test]
    fn test_transparency_detection() {
        let converter = ImageConverter::new();

        // RGBA image has transparency
        let rgba_img = DynamicImage::new_rgba8(100, 100);
        assert!(converter.has_transparency(&rgba_img));

        // RGB image doesn't have transparency
        let rgb_img = DynamicImage::new_rgb8(100, 100);
        assert!(!converter.has_transparency(&rgb_img));
    }

    #[test]
    fn test_photographic_heuristic() {
        let converter = ImageConverter::new();

        // Large image should be considered photographic
        let large_img = DynamicImage::new_rgb8(1000, 1000);
        assert!(converter.is_photographic(&large_img));

        // Small image should not be considered photographic
        let small_img = DynamicImage::new_rgb8(100, 100);
        assert!(!converter.is_photographic(&small_img));
    }

    #[test]
    fn test_size_formatting() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(2 * 1024 * 1024), "2.0 MB");
    }

    #[test]
    fn test_format_name_mapping() {
        assert_eq!(format_name(ImageFormat::Png), "PNG");
        assert_eq!(format_name(ImageFormat::Jpeg), "JPEG");
        assert_eq!(format_name(ImageFormat::WebP), "WebP");
        assert_eq!(format_name(ImageFormat::Tiff), "TIFF");
    }

    #[test]
    fn test_conversion_info_calculations() {
        let info = ConversionInfo {
            original_format: Some(ImageFormat::Tiff),
            original_size: 1000,
            target_format: Some(ImageFormat::Png),
            needs_conversion: true,
            estimated_target_size: 500,
        };

        assert_eq!(info.original_format_name(), "TIFF");
        assert_eq!(info.target_format_name(), "PNG");
        assert_eq!(info.size_reduction_percent(), Some(50.0));
    }
}
