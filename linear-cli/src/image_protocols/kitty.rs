// ABOUTME: Kitty terminal graphics protocol implementation
// ABOUTME: Handles base64 encoding and chunking according to Kitty spec

use super::ImageProtocol;
use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};

pub struct KittyProtocol;

impl ImageProtocol for KittyProtocol {
    fn render_image(&self, data: &[u8], _alt_text: &str, _url: &str) -> Result<String> {
        // Encode image data to base64
        let base64_data = STANDARD.encode(data);

        // Chunk into 4096-byte pieces (multiples of 4)
        let chunk_size = 4096;
        let chunks: Result<Vec<&str>, _> = base64_data
            .as_bytes()
            .chunks(chunk_size)
            .map(|chunk| std::str::from_utf8(chunk))
            .collect();
        let chunks =
            chunks.map_err(|e| anyhow!("Failed to convert base64 chunk to UTF-8: {}", e))?;

        let mut output = String::new();

        // Format depends on image format detection
        let format = detect_image_format(data);
        let format_code = match format {
            "png" => 100,
            "jpeg" => 24,
            "gif" => 100, // PNG for GIF (let kitty convert)
            _ => 100,     // Default to PNG
        };

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let m_value = if is_last { 0 } else { 1 };

            if i == 0 {
                // First chunk includes format and transmission action
                output.push_str(&format!(
                    "\x1b_Ga=T,f={},m={};{}\x1b\\",
                    format_code, m_value, chunk
                ));
            } else {
                // Continuation chunks
                output.push_str(&format!("\x1b_Gm={};{}\x1b\\", m_value, chunk));
            }
        }

        // Display the transmitted image
        output.push_str("\x1b_Ga=p,q=2\x1b\\");

        Ok(output)
    }

    fn max_size_bytes(&self) -> u64 {
        10 * 1024 * 1024 // 10MB default
    }

    fn supported_formats(&self) -> &[&str] {
        &["image/png", "image/jpeg", "image/gif", "image/webp"]
    }
}

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

    // WebP signature: RIFF....WEBP
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return "webp";
    }

    "unknown"
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // Unknown format
        let unknown_data = vec![0x00, 0x01, 0x02];
        assert_eq!(detect_image_format(&unknown_data), "unknown");
    }

    #[test]
    fn test_kitty_protocol_chunking() {
        let protocol = KittyProtocol;
        let test_data = vec![0x89, 0x50, 0x4E, 0x47]; // Minimal PNG signature

        let result = protocol.render_image(&test_data, "test", "http://example.com/test.png");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("\x1b_Ga=T")); // Transmission action
        assert!(output.contains("f=100")); // PNG format
        assert!(output.contains("m=0")); // Final chunk marker
        assert!(output.contains("\x1b_Ga=p")); // Display action
    }
}
