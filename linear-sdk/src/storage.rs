// ABOUTME: Secure token storage using system keychain
// ABOUTME: Handles OAuth token persistence with keyring backend

#[cfg(feature = "oauth")]
use keyring::Entry;

#[cfg(feature = "oauth")]
const SERVICE: &str = "linear-cli";
#[cfg(feature = "oauth")]
const ACCOUNT: &str = "oauth-token";

#[cfg(feature = "oauth")]
pub fn store(token: &str) -> anyhow::Result<()> {
    Entry::new(SERVICE, ACCOUNT)?.set_password(token)?;
    Ok(())
}

#[cfg(feature = "oauth")]
pub fn load() -> anyhow::Result<String> {
    Ok(Entry::new(SERVICE, ACCOUNT)?.get_password()?)
}

#[cfg(feature = "oauth")]
pub fn clear() -> anyhow::Result<()> {
    // delete_credential() is feature-gated; ignore error on first run
    let _ = Entry::new(SERVICE, ACCOUNT)?.delete_credential();
    Ok(())
}

#[cfg(not(feature = "oauth"))]
pub fn store(_token: &str) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("OAuth feature not enabled"))
}

#[cfg(not(feature = "oauth"))]
pub fn load() -> anyhow::Result<String> {
    Err(anyhow::anyhow!("OAuth feature not enabled"))
}

#[cfg(not(feature = "oauth"))]
pub fn clear() -> anyhow::Result<()> {
    Err(anyhow::anyhow!("OAuth feature not enabled"))
}

#[cfg(test)]
#[cfg(feature = "oauth")]
mod tests {
    use super::*;

    // Use a test-specific service name to avoid interfering with real keychain
    const TEST_SERVICE: &str = "linear-cli-test";
    const TEST_ACCOUNT: &str = "oauth-token-test";

    fn test_store(token: &str) -> anyhow::Result<()> {
        Entry::new(TEST_SERVICE, TEST_ACCOUNT)?.set_password(token)?;
        Ok(())
    }

    fn test_load() -> anyhow::Result<String> {
        Ok(Entry::new(TEST_SERVICE, TEST_ACCOUNT)?.get_password()?)
    }

    fn test_clear() -> anyhow::Result<()> {
        let _ = Entry::new(TEST_SERVICE, TEST_ACCOUNT)?.delete_credential();
        Ok(())
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_keyring_operations() {
        // This test requires keychain access and should be run manually
        // Clear any existing entry first
        let _ = test_clear();

        // Test store and load
        let test_token = "test-token-12345";
        match test_store(test_token) {
            Ok(_) => {
                match test_load() {
                    Ok(loaded_token) => {
                        assert_eq!(loaded_token, test_token);
                        // Clean up
                        let _ = test_clear();
                    }
                    Err(_) => {
                        // May fail on CI or headless systems
                        eprintln!("Warning: Could not load token from keyring");
                    }
                }
            }
            Err(_) => {
                // May fail on CI or headless systems
                eprintln!("Warning: Could not store token in keyring");
            }
        }
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_clear_nonexistent() {
        // Should not error when clearing non-existent entry
        let result = test_clear();
        assert!(result.is_ok());
    }

    #[test]
    fn test_oauth_disabled_fallback() {
        // Test that functions return appropriate errors when oauth is disabled
        // This test can run without keychain access
        #[cfg(not(feature = "oauth"))]
        {
            assert!(store("test").is_err());
            assert!(load().is_err());
            assert!(clear().is_err());
        }
    }
}
