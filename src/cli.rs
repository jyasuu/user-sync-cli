use clap::Parser;

/// Sync users from the remote user service into the local database.
///
/// All flags can also be supplied via environment variables (shown in brackets).
/// A `.env` file in the working directory is loaded automatically when present.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "user-sync",
    version,
    about = "One-shot user sync from user-service → database",
    long_about = None,
)]
pub struct Cli {
    /// User service base URL
    /// [env: USER_ENDPOINT]
    #[arg(long, env = "USER_ENDPOINT")]
    pub user_endpoint: String,

    /// Days before today for the earliest window boundary (e.g. 30)
    /// [env: START_INTERVAL]
    #[arg(long, env = "START_INTERVAL", default_value = "30")]
    pub start_interval: i64,

    /// Days before today for the latest window boundary (0 = today)
    /// [env: END_INTERVAL]
    #[arg(long, env = "END_INTERVAL", default_value = "0")]
    pub end_interval: i64,

    /// Maximum day-span sent per API request chunk
    /// [env: INTERVAL_LIMIT]
    #[arg(long, env = "INTERVAL_LIMIT", default_value = "7")]
    pub interval_limit: i64,

    /// Comma-separated realm types to include (omit for all)
    /// [env: INCLUDE_REALM_TYPES]
    #[arg(long, env = "INCLUDE_REALM_TYPES", default_value = "")]
    pub include_realm_types: String,

    /// Optional SQL statement to execute after the sync completes
    /// [env: SYNC_SQL]
    #[arg(long, env = "SYNC_SQL", default_value = "")]
    pub sync_sql: String,

    /// OAuth2 token endpoint URL
    /// [env: TOKEN_URL]
    #[arg(long, env = "TOKEN_URL")]
    pub token_url: String,

    /// OAuth2 client ID
    /// [env: CLIENT_ID]
    #[arg(long, env = "CLIENT_ID")]
    pub client_id: String,

    /// OAuth2 client secret
    /// [env: CLIENT_SECRET]
    #[arg(long, env = "CLIENT_SECRET")]
    pub client_secret: String,

    /// PostgreSQL connection string
    /// [env: DATABASE_URL]
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// HTTP request timeout in seconds
    /// [env: HTTP_TIMEOUT_SECS]
    #[arg(long, env = "HTTP_TIMEOUT_SECS", default_value = "600")]
    pub http_timeout_secs: u64,

    /// Sleep duration between chunks in seconds (default: 60)
    /// [env: CHUNK_SLEEP_SECS]
    #[arg(long, env = "CHUNK_SLEEP_SECS", default_value = "60")]
    pub chunk_sleep_secs: u64,

    /// Print what would be written without touching the database
    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    /// Suppress progress output (errors still printed to stderr)
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,
}

impl Cli {
    pub fn realm_type(&self) -> Option<&str> {
        if self.include_realm_types.is_empty() {
            None
        } else {
            Some(&self.include_realm_types)
        }
    }
}
