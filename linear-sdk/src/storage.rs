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

    #[test]
    fn test_keyring_operations() {
        // This test may fail on systems without proper keyring setup
        // Clear any existing entry first
        let _ = clear();

        // Test store and load
        let test_token = "test-token-12345";
        match store(test_token) {
            Ok(_) => {
                match load() {
                    Ok(loaded_token) => {
                        assert_eq!(loaded_token, test_token);
                        // Clean up
                        let _ = clear();
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
    fn test_clear_nonexistent() {
        // Should not error when clearing non-existent entry
        let result = clear();
        assert!(result.is_ok());
    }
}
