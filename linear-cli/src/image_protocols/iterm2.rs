// ABOUTME: iTerm2 terminal inline image protocol implementation
// ABOUTME: Handles base64 encoding with iTerm2-specific escape sequences

use super::ImageProtocol;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};

pub struct ITerm2Protocol;

impl ImageProtocol for ITerm2Protocol {
    fn render_image(&self, data: &[u8], _alt_text: &str, url: &str) -> Result<String> {
        // Encode image data to base64
        let base64_data = STANDARD.encode(data);

        // Extract filename from URL for the name parameter
        let filename = extract_filename_from_url(url);
        let filename_b64 = STANDARD.encode(filename.as_bytes());

        // ITerm2 inline image format:
        // \x1b]1337;File=name=filename;size=filesize;inline=1:base64data\x07
        let escape_sequence = format!(
            "\x1b]1337;File=name={};size={};inline=1:{}\x07",
            filename_b64,
            data.len(),
            base64_data
        );

        // Add a newline after the image for better formatting
        Ok(format!("{}\n", escape_sequence))
    }

    fn max_size_bytes(&self) -> u64 {
        10 * 1024 * 1024 // 10MB default, same as Kitty
    }

    fn supported_formats(&self) -> &[&str] {
        // iTerm2 supports these formats natively
        &[
            "image/png",
            "image/jpeg",
            "image/gif",
            "image/tiff",
            "image/bmp",
        ]
    }
}

/// Extract a reasonable filename from a URL for the iTerm2 name parameter
fn extract_filename_from_url(url: &str) -> String {
    // Try to get the last path component
    if let Some(path_part) = url.split('/').last() {
        // Remove query parameters and fragments
        let clean_name = path_part
            .split('?')
            .next()
            .unwrap_or(path_part)
            .split('#')
            .next()
            .unwrap_or(path_part);

        if !clean_name.is_empty() && clean_name != "/" {
            return clean_name.to_string();
        }
    }

    // Fallback to generic name
    "image".to_string()
}

/// Detect image format from data (reuse logic from kitty.rs if needed)
#[allow(dead_code)] // May be used for format-specific optimizations
fn detect_image_format(data: &[u8]) -> &'static str {
    if data.len() < 8 {
        return "unknown";
    }

    // PNG signature: 89 50 4E 47 0D 0A 1A 0A
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return "png";
    }

    // JPEG signature: FF D8 FF
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "jpeg";
    }

    // GIF signature: GIF87a or GIF89a
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return "gif";
    }

    // TIFF signatures: II*\0 (little-endian) or MM\0* (big-endian)
    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return "tiff";
    }

    // BMP signature: BM
    if data.starts_with(b"BM") {
        return "bmp";
    }

    "unknown"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_extraction() {
        assert_eq!(
            extract_filename_from_url("https://uploads.linear.app/abc123/screenshot.png"),
            "screenshot.png"
        );

        assert_eq!(
            extract_filename_from_url("https://example.com/path/to/image.jpg?param=value"),
            "image.jpg"
        );

        assert_eq!(
            extract_filename_from_url("https://example.com/path/to/image.gif#fragment"),
            "image.gif"
        );

        assert_eq!(extract_filename_from_url("https://example.com/"), "image");

        assert_eq!(
            extract_filename_from_url("https://example.com/no-extension"),
            "no-extension"
        );
    }

    #[test]
    fn test_image_format_detection() {
        // PNG test data
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_image_format(&png_data), "png");

        // JPEG test data
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(detect_image_format(&jpeg_data), "jpeg");

        // GIF test data
        let gif_data = b"GIF89a\x00\x00".to_vec();
        assert_eq!(detect_image_format(&gif_data), "gif");

        // TIFF test data (little-endian)
        let tiff_data = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
        assert_eq!(detect_image_format(&tiff_data), "tiff");

        // BMP test data
        let bmp_data = b"BM\x36\x00\x00\x00\x00\x00".to_vec();
        assert_eq!(detect_image_format(&bmp_data), "bmp");

        // Unknown format
        let unknown_data = vec![0x00, 0x01, 0x02];
        assert_eq!(detect_image_format(&unknown_data), "unknown");
    }

    #[test]
    fn test_iterm2_protocol_rendering() {
        let protocol = ITerm2Protocol;
        let test_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature

        let result =
            protocol.render_image(&test_data, "test image", "https://example.com/test.png");
        assert!(result.is_ok());

        let output = result.unwrap();

        // Should contain iTerm2 escape sequence markers
        assert!(output.contains("\x1b]1337;File="));
        assert!(output.contains("name="));
        assert!(output.contains("size=8")); // Length of test_data
        assert!(output.contains("inline=1"));
        assert!(output.contains("\x07")); // iTerm2 terminator

        // Should end with newline
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_iterm2_protocol_base64_encoding() {
        let protocol = ITerm2Protocol;
        let test_data = b"test data".to_vec();
        let expected_b64 = STANDARD.encode(&test_data);

        let result = protocol.render_image(&test_data, "test", "https://example.com/test.png");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains(&expected_b64));
    }

    #[test]
    fn test_protocol_max_size() {
        let protocol = ITerm2Protocol;
        assert_eq!(protocol.max_size_bytes(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_supported_formats() {
        let protocol = ITerm2Protocol;
        let formats = protocol.supported_formats();

        assert!(formats.contains(&"image/png"));
        assert!(formats.contains(&"image/jpeg"));
        assert!(formats.contains(&"image/gif"));
        assert!(formats.contains(&"image/tiff"));
        assert!(formats.contains(&"image/bmp"));
    }
}
