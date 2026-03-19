use std::sync::Arc;

use tokio::time::{sleep, Duration};
use tracing::info;

use crate::{api::UserApiClient, cli::Cli, db::UserRepository};

pub struct SyncOrchestrator {
    args: Arc<Cli>,
    api: Arc<UserApiClient>,
    repo: Arc<UserRepository>,
}

impl SyncOrchestrator {
    pub fn new(args: Arc<Cli>, api: Arc<UserApiClient>, repo: Arc<UserRepository>) -> Self {
        Self { args, api, repo }
    }

    /// Runs one complete sync and returns a summary.
    pub async fn run(&self) -> SyncSummary {
        let mut summary = SyncSummary::default();

        self.sync_users(&mut summary).await;

        if !self.args.dry_run && !self.args.sync_sql.is_empty() {
            if let Err(e) = self.repo.run_sync_sql(&self.args.sync_sql).await {
                eprintln!("syncSQL failed: {e:#}");
                summary.errors += 1;
            } else {
                info!("syncSQL executed");
            }
        } else if self.args.dry_run && !self.args.sync_sql.is_empty() {
            self.progress("[dry-run] would execute syncSQL");
        }

        summary
    }

    async fn sync_users(&self, summary: &mut SyncSummary) {
        let realm = self.args.realm_type().map(str::to_owned);
        let mut chunk_start = self.args.start_interval;

        loop {
            let chunk_end = (chunk_start - self.args.interval_limit).max(self.args.end_interval);

            self.progress(&format!(
                "Chunk: -{chunk_start}d .. -{chunk_end}d from today{}",
                if self.args.dry_run { " [dry-run]" } else { "" }
            ));

            let users = self
                .api
                .fetch_users(chunk_start, chunk_end, realm.as_deref())
                .await;

            summary.fetched += users.len();

            if self.args.dry_run {
                self.progress(&format!(
                    "  [dry-run] would upsert {} users -- skipping DB write",
                    users.len()
                ));
            } else {
                let errs = self.repo.upsert_all(&users).await;
                summary.upserted += users.len().saturating_sub(errs);
                summary.errors += errs;
            }

            chunk_start = chunk_end;
            if chunk_start <= self.args.end_interval {
                break;
            }

            self.progress(&format!(
                "Sleeping {}s before next chunk...",
                self.args.chunk_sleep_secs
            ));
            sleep(Duration::from_secs(self.args.chunk_sleep_secs)).await;
        }
    }

    fn progress(&self, msg: &str) {
        if !self.args.quiet {
            println!("{msg}");
        }
        info!("{msg}");
    }
}

#[derive(Debug, Default)]
pub struct SyncSummary {
    pub fetched: usize,
    pub upserted: usize,
    pub errors: usize,
}

impl std::fmt::Display for SyncSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fetched={} upserted={} errors={}",
            self.fetched, self.upserted, self.errors
        )
    }
}
