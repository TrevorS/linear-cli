// ABOUTME: URL validation and image detection for safe image processing
// ABOUTME: Implements security checks and content-type validation

use anyhow::{Result, anyhow};
use std::collections::HashSet;
use url::Url;

pub struct ImageUrlValidator {
    allowed_domains: HashSet<String>,
    max_size_hint: u64,
}

impl ImageUrlValidator {
    pub fn new() -> Self {
        let mut allowed_domains = HashSet::new();

        // Linear's official domains
        allowed_domains.insert("uploads.linear.app".to_string());
        allowed_domains.insert("linear-uploads.s3.amazonaws.com".to_string());

        // Allow additional domains via environment variable
        if let Ok(additional) = std::env::var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS") {
            for domain in additional.split(',') {
                allowed_domains.insert(domain.trim().to_string());
            }
        }

        let max_size_hint = parse_size_env("LINEAR_CLI_MAX_IMAGE_SIZE", 10 * 1024 * 1024);

        Self {
            allowed_domains,
            max_size_hint,
        }
    }

    pub fn is_image_url(&self, url: &str) -> bool {
        // Fast checks first
        if !self.is_http_url(url) {
            return false;
        }

        // Check file extension
        if self.has_image_extension(url) {
            return true;
        }

        // Check for Linear upload URLs (often images without extensions)
        if self.is_linear_upload_url(url) {
            return true;
        }

        false
    }

    pub fn validate_image_url(&self, url: &str) -> Result<Url> {
        let parsed_url = Url::parse(url).map_err(|e| anyhow!("Invalid URL '{}': {}", url, e))?;

        // Security checks
        self.check_scheme(&parsed_url)?;
        self.check_domain(&parsed_url)?;
        self.check_path(&parsed_url)?;

        Ok(parsed_url)
    }

    fn is_http_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    fn has_image_extension(&self, url: &str) -> bool {
        let image_extensions = [
            ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".tiff", ".svg",
        ];

        let url_lower = url.to_lowercase();

        // Check for extension at end of path (before query params)
        let path = url_lower.split('?').next().unwrap_or(&url_lower);
        image_extensions.iter().any(|ext| path.ends_with(ext))
    }

    pub fn is_linear_upload_url(&self, url: &str) -> bool {
        // Parse URL to check domain specifically
        if let Ok(parsed) = Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host == "uploads.linear.app" || host == "linear-uploads.s3.amazonaws.com";
            }
        }
        false
    }

    fn check_scheme(&self, url: &Url) -> Result<()> {
        match url.scheme() {
            "https" => Ok(()),
            "http" => {
                // Allow HTTP for localhost/development
                if let Some(host) = url.host_str() {
                    if host == "localhost"
                        || host.starts_with("127.")
                        || host.starts_with("192.168.")
                    {
                        return Ok(());
                    }
                }
                Err(anyhow!("HTTP URLs not allowed for security: {}", url))
            }
            scheme => Err(anyhow!("Unsupported URL scheme '{}': {}", scheme, url)),
        }
    }

    fn check_domain(&self, url: &Url) -> Result<()> {
        let host = url
            .host_str()
            .ok_or_else(|| anyhow!("URL missing host: {}", url))?;

        if self.allowed_domains.contains(host) {
            return Ok(());
        }

        // Check if host is subdomain of allowed domain
        for allowed in &self.allowed_domains {
            if host.ends_with(&format!(".{}", allowed)) {
                return Ok(());
            }
        }

        Err(anyhow!(
            "Domain '{}' not in allowed list: {:?}",
            host,
            self.allowed_domains
        ))
    }

    fn check_path(&self, url: &Url) -> Result<()> {
        let path = url.path();

        // Prevent directory traversal
        if path.contains("..") {
            return Err(anyhow!("Path traversal detected: {}", url));
        }

        // Prevent suspicious paths
        let suspicious_patterns = ["/etc/", "/var/", "/proc/", "/sys/"];
        for pattern in &suspicious_patterns {
            if path.contains(pattern) {
                return Err(anyhow!("Suspicious path detected: {}", url));
            }
        }

        Ok(())
    }

    pub fn max_size_bytes(&self) -> u64 {
        self.max_size_hint
    }
}

pub fn parse_size_env(env_var: &str, default: u64) -> u64 {
    let Ok(size_str) = std::env::var(env_var) else {
        return default;
    };

    let size_str = size_str.to_uppercase();

    let (number_part, unit) = if size_str.ends_with("MB") {
        (size_str.trim_end_matches("MB"), 1024 * 1024)
    } else if size_str.ends_with("KB") {
        (size_str.trim_end_matches("KB"), 1024)
    } else if size_str.ends_with("GB") {
        (size_str.trim_end_matches("GB"), 1024 * 1024 * 1024)
    } else {
        (size_str.as_str(), 1)
    };

    number_part.parse::<u64>().unwrap_or(default) * unit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_extension_detection() {
        let validator = ImageUrlValidator::new();

        assert!(validator.has_image_extension("https://example.com/image.png"));
        assert!(validator.has_image_extension("https://example.com/image.JPG"));
        assert!(validator.has_image_extension("https://example.com/path/image.jpeg?query=1"));
        assert!(!validator.has_image_extension("https://example.com/document.pdf"));
        assert!(!validator.has_image_extension("https://example.com/no-extension"));
    }

    #[test]
    fn test_linear_upload_detection() {
        let validator = ImageUrlValidator::new();

        assert!(validator.is_linear_upload_url("https://uploads.linear.app/abc123/image"));
        assert!(
            validator
                .is_linear_upload_url("https://linear-uploads.s3.amazonaws.com/def456/image.png")
        );
        assert!(!validator.is_linear_upload_url("https://evil.com/uploads.linear.app/fake"));
    }

    #[test]
    fn test_url_validation_security() {
        let validator = ImageUrlValidator::new();

        // Valid Linear URLs should pass
        assert!(
            validator
                .validate_image_url("https://uploads.linear.app/image.png")
                .is_ok()
        );

        // Invalid schemes should fail
        assert!(
            validator
                .validate_image_url("ftp://uploads.linear.app/image.png")
                .is_err()
        );
        assert!(validator.validate_image_url("file:///etc/passwd").is_err());

        // Directory traversal should fail
        assert!(
            validator
                .validate_image_url("https://uploads.linear.app/../../../etc/passwd")
                .is_err()
        );

        // Unauthorized domains should fail
        assert!(
            validator
                .validate_image_url("https://evil.com/image.png")
                .is_err()
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_custom_allowed_domains() {
        // Test environment variable configuration
        unsafe {
            std::env::set_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS", "example.com,test.org");
        }

        let validator = ImageUrlValidator::new();

        assert!(
            validator
                .validate_image_url("https://example.com/image.png")
                .is_ok()
        );
        assert!(
            validator
                .validate_image_url("https://test.org/image.png")
                .is_ok()
        );
        assert!(
            validator
                .validate_image_url("https://unauthorized.com/image.png")
                .is_err()
        );

        unsafe {
            std::env::remove_var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_size_parsing() {
        assert_eq!(parse_size_env("NONEXISTENT", 1000), 1000);

        unsafe {
            std::env::set_var("TEST_SIZE", "5MB");
        }
        assert_eq!(parse_size_env("TEST_SIZE", 1000), 5 * 1024 * 1024);

        unsafe {
            std::env::set_var("TEST_SIZE", "2GB");
        }
        assert_eq!(parse_size_env("TEST_SIZE", 1000), 2 * 1024 * 1024 * 1024);

        unsafe {
            std::env::set_var("TEST_SIZE", "500KB");
        }
        assert_eq!(parse_size_env("TEST_SIZE", 1000), 500 * 1024);

        unsafe {
            std::env::set_var("TEST_SIZE", "invalid");
        }
        assert_eq!(parse_size_env("TEST_SIZE", 1000), 1000);

        unsafe {
            std::env::remove_var("TEST_SIZE");
        }
    }

    #[test]
    fn test_is_image_url_comprehensive() {
        let validator = ImageUrlValidator::new();

        // Should detect as images
        assert!(validator.is_image_url("https://uploads.linear.app/abc123/screenshot.png"));
        assert!(validator.is_image_url("https://uploads.linear.app/def456/diagram")); // Linear upload without extension
        assert!(validator.is_image_url("https://example.com/image.jpeg"));

        // Should not detect as images
        assert!(!validator.is_image_url("https://example.com/document.pdf"));
        assert!(!validator.is_image_url("https://example.com/api/endpoint"));
        assert!(!validator.is_image_url("ftp://example.com/image.png")); // Wrong protocol
        assert!(!validator.is_image_url("not-a-url"));
    }
}
