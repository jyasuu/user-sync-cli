use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug)]
struct CachedToken {
    token: String,
    /// Epoch seconds at which the token expires (with safety margin)
    expires_at: u64,
}

/// Thread-safe OAuth2 token manager using the client-credentials flow.
/// Automatically re-fetches the token when it is about to expire.
#[derive(Clone)]
pub struct TokenManager {
    client: Client,
    token_url: String,
    client_id: String,
    client_secret: String,
    cache: Arc<RwLock<Option<CachedToken>>>,
}

impl TokenManager {
    pub fn new(
        client: Client,
        token_url: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self {
            client,
            token_url,
            client_id,
            client_secret,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns a valid bearer token, re-fetching from the token endpoint if expired.
    pub async fn token(&self) -> Result<String> {
        // Fast path: valid cached token
        {
            let guard = self.cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if !self.is_expired(cached) {
                    debug!("Returning cached OAuth2 token");
                    return Ok(cached.token.clone());
                }
            }
        }

        // Slow path: fetch a new token
        let mut guard = self.cache.write().await;

        // Double-check after acquiring write lock (another task may have refreshed)
        if let Some(cached) = guard.as_ref() {
            if !self.is_expired(cached) {
                return Ok(cached.token.clone());
            }
        }

        info!("Fetching new OAuth2 token from {}", self.token_url);

        let resp = self
            .client
            .post(&self.token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .context("Token request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Token endpoint returned {status}: {body}");
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .context("Failed to deserialize token response")?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Subtract 60-second safety margin before the real expiry
        let expires_at = now + token_resp.expires_in.saturating_sub(60);

        info!(
            "OAuth2 token obtained, expires in {}s",
            token_resp.expires_in
        );

        *guard = Some(CachedToken {
            token: token_resp.access_token.clone(),
            expires_at,
        });

        Ok(token_resp.access_token)
    }

    fn is_expired(&self, cached: &CachedToken) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= cached.expires_at
    }
}
