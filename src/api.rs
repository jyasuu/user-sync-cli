use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use reqwest::Client;
use std::time::Duration as StdDuration;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};
use tracing::{error, info, warn};
use url::Url;

use crate::{models::User, token::TokenManager};

pub struct UserApiClient {
    http: Client,
    token_manager: TokenManager,
    base_url: String,
}

impl UserApiClient {
    pub fn new(http: Client, token_manager: TokenManager, base_url: String) -> Self {
        Self {
            http,
            token_manager,
            base_url,
        }
    }

    /// Fetches users for a time window with automatic retry + exponential backoff.
    ///
    /// - start_offset: days before today for the window start (larger = further back)
    /// - end_offset:   days before today for the window end   (smaller = closer to today)
    /// - realm_type:   optional filter value
    pub async fn fetch_users(
        &self,
        start_offset: i64,
        end_offset: i64,
        realm_type: Option<&str>,
    ) -> Vec<User> {
        // Retry strategy: 5 attempts, 2 s → 4 s → 8 s → 16 s → 30 s (capped), with jitter
        let strategy = ExponentialBackoff::from_millis(2_000)
            .factor(2)
            .max_delay(StdDuration::from_secs(30))
            .map(jitter)
            .take(5);

        let realm = realm_type.map(str::to_owned);

        let result = Retry::spawn(strategy, || async {
            self.try_fetch(start_offset, end_offset, realm.as_deref())
                .await
                .map_err(|e| {
                    warn!("User fetch attempt failed: {e:#}");
                    e
                })
        })
        .await;

        match result {
            Ok(users) => users,
            Err(e) => {
                error!(
                    start_offset,
                    end_offset, "All retry attempts exhausted: {e:#}"
                );
                Vec::new() // graceful fallback – matches @Recover behaviour
            }
        }
    }

    async fn try_fetch(
        &self,
        start_offset: i64,
        end_offset: i64,
        realm_type: Option<&str>,
    ) -> Result<Vec<User>> {
        let today = Utc::now().date_naive();
        let start = (today - Duration::days(start_offset))
            .format("%Y%m%d")
            .to_string();
        let end = (today - Duration::days(end_offset))
            .format("%Y%m%d")
            .to_string();

        let token = self
            .token_manager
            .token()
            .await
            .context("Could not obtain OAuth2 token")?;

        let mut url = Url::parse(&self.base_url).context("Invalid user_endpoint URL")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("start_time", &start);
            q.append_pair("end_time", &end);
            if let Some(rt) = realm_type {
                q.append_pair("realm_type", rt);
            }
        }

        info!("Fetching users: {url}");

        let resp = self
            .http
            .get(url.as_str())
            .bearer_auth(&token)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("User endpoint returned {status}: {body}");
        }

        let payload: crate::models::UsersResponse = resp
            .json()
            .await
            .context("Failed to deserialize users response")?;

        info!(
            "Fetched {} users (start_offset={start_offset}, end_offset={end_offset})",
            payload.data.len()
        );
        Ok(payload.data)
    }
}
