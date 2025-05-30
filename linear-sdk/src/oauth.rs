// ABOUTME: OAuth2 authentication flow implementation for Linear API
// ABOUTME: Handles browser-based OAuth with PKCE, token storage in system keychain

#[cfg(feature = "oauth")]
use crate::{Result, storage};
#[cfg(feature = "oauth")]
use std::borrow::Cow;

#[cfg(feature = "oauth")]
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient,
};

#[cfg(feature = "oauth")]
use tiny_http::{Header, Response, Server};

#[cfg(feature = "oauth")]
use url::Url;

#[cfg(feature = "oauth")]
const REDIRECT_PORT: u16 = 8089;
#[cfg(feature = "oauth")]
const REDIRECT_PATH: &str = "/callback";

#[cfg(feature = "oauth")]
fn auth_error(reason: &'static str) -> crate::LinearError {
    crate::LinearError::Auth {
        reason: Cow::Borrowed(reason),
        source: None,
    }
}

// Type alias for the OAuth client with all its type state parameters
#[cfg(feature = "oauth")]
type ConfiguredClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

#[cfg(feature = "oauth")]
pub struct OAuthManager {
    client: ConfiguredClient,
    http_client: reqwest::blocking::Client,
}

#[cfg(feature = "oauth")]
impl OAuthManager {
    pub fn new(client_id: String) -> Result<Self> {
        if client_id.is_empty() {
            return Err(crate::LinearError::OAuthConfig);
        }

        // These URLs come from Linear's OAuth config
        let auth_url = AuthUrl::new("https://linear.app/oauth/authorize".to_string())
            .map_err(|_| crate::LinearError::OAuthConfig)?;
        let token_url = TokenUrl::new("https://api.linear.app/oauth/token".to_string())
            .map_err(|_| crate::LinearError::OAuthConfig)?;

        let client = BasicClient::new(ClientId::new(client_id))
            // No client secret for public PKCE clients
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(
                RedirectUrl::new(format!("http://localhost:{REDIRECT_PORT}{REDIRECT_PATH}"))
                    .map_err(|_| crate::LinearError::OAuthConfig)?,
            );

        // Create HTTP client without redirect following (security best practice)
        let http_client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| auth_error("HTTP client build failed"))?;

        Ok(Self {
            client,
            http_client,
        })
    }

    pub fn from_env() -> Result<Self> {
        let client_id =
            std::env::var("LINEAR_OAUTH_CLIENT_ID").map_err(|_| crate::LinearError::OAuthConfig)?;

        if client_id.is_empty() {
            return Err(crate::LinearError::OAuthConfig);
        }

        Self::new(client_id)
    }

    pub fn login(&self) -> Result<()> {
        // Step 1: generate CSRF token + PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("read".into()))
            .add_scope(Scope::new("write".into()))
            .url();

        // Step 2: open browser
        open::that(auth_url.as_str()).map_err(|_| auth_error("Failed to open browser"))?;

        println!("Opening browser for authentication...");
        println!(
            "Waiting for callback on http://localhost:{}{}",
            REDIRECT_PORT, REDIRECT_PATH
        );

        // Step 3: listen for callback
        let server = Server::http(("127.0.0.1", REDIRECT_PORT))
            .map_err(|_| auth_error("Failed to start callback server"))?;

        for request in server.incoming_requests() {
            let request_url = request.url();

            // Skip favicon requests
            if request_url.contains("favicon.ico") {
                let _ = request.respond(Response::empty(404));
                continue;
            }

            if request_url.starts_with(REDIRECT_PATH) {
                // Parse query ?code=…&state=…
                let url = format!("http://localhost:{}{}", REDIRECT_PORT, request_url);
                let params: Url = url.parse().map_err(|_| auth_error("Invalid URL format"))?;

                let code = params
                    .query_pairs()
                    .find(|p| p.0 == "code")
                    .map(|p| p.1.to_string())
                    .ok_or(auth_error("Missing OAuth parameter"))?;

                let state = params
                    .query_pairs()
                    .find(|p| p.0 == "state")
                    .map(|p| p.1.to_string())
                    .ok_or(auth_error("Missing OAuth parameter"))?;

                // CSRF check
                if state != *csrf_token.secret() {
                    return Err(auth_error("OAuth authentication failed"));
                }

                // Respond immediately so the browser tab can close
                let response =
                    Response::from_string("Authentication successful! You can close this window.")
                        .with_header(
                            Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..])
                                .map_err(|_| auth_error("Failed to create HTTP headers"))?,
                        );

                request
                    .respond(response)
                    .map_err(|_| auth_error("Failed to send HTTP response"))?;

                // Step 4: trade code+verifier for access token
                let token = self
                    .client
                    .exchange_code(AuthorizationCode::new(code))
                    .set_pkce_verifier(pkce_verifier)
                    .request(&self.http_client)
                    .map_err(|_| auth_error("Token exchange failed"))?;

                // Step 5: persist
                storage::store(token.access_token().secret())
                    .map_err(|_| auth_error("Failed to store token"))?;
                println!("✓ Logged in successfully!");
                break;
            }
        }
        Ok(())
    }

    pub fn logout(&self) -> Result<()> {
        storage::clear().map_err(|_| auth_error("Failed to clear stored credentials"))?;
        Ok(())
    }

    pub fn get_token(&self) -> Result<String> {
        storage::load().map_err(|_| auth_error("Failed to load stored credentials"))
    }
}

#[cfg(test)]
#[cfg(feature = "oauth")]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_manager_creation() {
        // Test with explicit client ID should always succeed
        let manager = OAuthManager::new("test-client-id".to_string());
        assert!(manager.is_ok());

        // Test with empty client ID should fail
        let manager = OAuthManager::new("".to_string());
        assert!(manager.is_err());

        // Note: We can't easily test from_env() without modifying environment variables,
        // which is unsafe in Rust 1.87+. The from_env() method is simple enough that
        // testing new() with various inputs provides sufficient coverage.
    }
}
