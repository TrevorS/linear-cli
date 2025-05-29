## Description

Add OAuth authentication as an optional feature, providing a more secure alternative to API keys.

## Context

From the implementation plan (Prompt 11), we need to:
- Implement OAuth2 flow behind a feature flag
- Store tokens securely in system keychain
- Provide login/logout commands

## Acceptance Criteria

- [ ] Add OAuth dependencies behind feature flag:
  ```toml
  [features]
  default = []
  oauth = ["oauth2", "keyring", "open", "tiny_http"]
  ```
- [ ] Create OAuth module (conditional compilation):
  ```rust
  #[cfg(feature = "oauth")]
  mod oauth {
      pub async fn login() -> Result<String> {
          // OAuth2 flow implementation
      }
  }
  ```
- [ ] Add login command (only with oauth feature):
  ```rust
  #[cfg(feature = "oauth")]
  Login {
      /// Force new login even if token exists
      #[arg(long)]
      force: bool,
  }
  ```
- [ ] Implement OAuth2 flow:
  - [ ] Start local server on port 8089
  - [ ] Open browser to Linear's OAuth URL
  - [ ] Handle callback with authorization code
  - [ ] Exchange for access token
  - [ ] Store in system keychain (macOS Keychain, Linux Secret Service)
- [ ] Update authentication priority:
  1. Command line `--api-key`
  2. `LINEAR_API_KEY` env var
  3. OAuth token from keychain (if feature enabled)
- [ ] Add logout command to clear keychain
- [ ] Security requirements:
  - [ ] Use PKCE for OAuth flow
  - [ ] Set restrictive keychain access
  - [ ] Clear sensitive data after use
- [ ] Add tests:
  - [ ] Mock OAuth flow
  - [ ] Keychain storage (with mock)
  - [ ] Feature flag compilation

## Technical Details

- Use oauth2 crate for OAuth implementation
- Use keyring crate for cross-platform credential storage
- Implement proper PKCE flow for security

## Dependencies

- Depends on: #10 (Error Handling Polish)

## Definition of Done

- [ ] `linear login` opens browser and completes OAuth flow
- [ ] Token stored securely in system keychain
- [ ] `linear logout` clears stored credentials
- [ ] Feature compiles conditionally with oauth flag
- [ ] Authentication falls back appropriately
- [ ] PKCE implemented for security
