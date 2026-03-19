mod api;
mod cli;
mod db;
mod models;
mod sync;
mod token;

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use api::UserApiClient;
use cli::Cli;
use db::UserRepository;
use sync::SyncOrchestrator;
use token::TokenManager;

#[tokio::main]
async fn main() {
    // Load .env before clap so env vars set there are visible to --env flags
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    match run().await {
        Ok(summary) => {
            if summary.errors > 0 {
                eprintln!("Sync finished with errors -- {summary}");
                std::process::exit(1);
            } else {
                println!("Sync complete -- {summary}");
                std::process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("Fatal: {e:#}");
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<sync::SyncSummary> {
    let args = Arc::new(Cli::parse());

    if args.dry_run {
        info!("Running in dry-run mode -- no DB writes will occur");
    }

    let http = Client::builder()
        .timeout(Duration::from_secs(args.http_timeout_secs))
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .context("Failed to build HTTP client")?;

    let token_mgr = TokenManager::new(
        http.clone(),
        args.token_url.clone(),
        args.client_id.clone(),
        args.client_secret.clone(),
    );

    let api = Arc::new(UserApiClient::new(
        http,
        token_mgr,
        args.user_endpoint.clone(),
    ));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&args.database_url)
        .await
        .context("Failed to connect to database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Migration failed")?;

    let repo = Arc::new(UserRepository::new(pool));

    let orchestrator = SyncOrchestrator::new(Arc::clone(&args), api, repo);
    Ok(orchestrator.run().await)
}
