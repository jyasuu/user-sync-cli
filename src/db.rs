use anyhow::Result;
use bigdecimal::ToPrimitive;
use sqlx::PgPool;
use tracing::{debug, error, info};

use crate::models::{GlobalUserRow, User};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upserts a batch of users. Returns the number of individual failures.
    pub async fn upsert_all(&self, users: &[User]) -> usize {
        info!("Upserting {} users", users.len());
        let mut errors = 0usize;
        for user in users {
            if let Err(e) = self.upsert_one(user).await {
                error!(pccuid = user.pccuid, "Upsert failed: {e:#}");
                errors += 1;
            }
        }
        errors
    }

    async fn upsert_one(&self, user: &User) -> Result<()> {
        let row = GlobalUserRow::from(user);
        debug!(pccuid = %row.pccuid, "Upserting user");

        sqlx::query(
            r#"
            INSERT INTO global_users (
                pccuid, sso_acct, fact_no, local_fact_no,
                chinese_nm, local_pnl_nm, english_nm, contact_mail,
                sex, lo_posi_nm, disabled, disabled_date,
                update_date, lo_dept_nm, tel, leave_mk, acct_type, id
            )
            VALUES (
                $1,  $2,  $3,  $4,
                $5,  $6,  $7,  $8,
                $9,  $10, $11, $12,
                $13, $14, $15, $16, $17, $1
            )
            ON CONFLICT (pccuid) DO UPDATE SET
                sso_acct      = EXCLUDED.sso_acct,
                fact_no       = EXCLUDED.fact_no,
                local_fact_no = EXCLUDED.local_fact_no,
                chinese_nm    = EXCLUDED.chinese_nm,
                local_pnl_nm  = EXCLUDED.local_pnl_nm,
                english_nm    = EXCLUDED.english_nm,
                contact_mail  = EXCLUDED.contact_mail,
                sex           = EXCLUDED.sex,
                lo_posi_nm    = EXCLUDED.lo_posi_nm,
                disabled      = EXCLUDED.disabled,
                disabled_date = EXCLUDED.disabled_date,
                update_date   = EXCLUDED.update_date,
                lo_dept_nm    = EXCLUDED.lo_dept_nm,
                tel           = EXCLUDED.tel,
                leave_mk      = EXCLUDED.leave_mk,
                acct_type     = EXCLUDED.acct_type
            "#,
        )
        .bind(row.pccuid.to_i64())
        .bind(row.sso_acct)
        .bind(row.fact_no)
        .bind(row.local_fact_no)
        .bind(row.chinese_nm)
        .bind(row.local_pnl_nm)
        .bind(row.english_nm)
        .bind(row.contact_mail)
        .bind(row.sex)
        .bind(row.lo_posi_nm)
        .bind(row.disabled)
        .bind(row.disabled_date)
        .bind(row.update_date)
        .bind(row.lo_dept_nm)
        .bind(row.tel)
        .bind(row.leave_mk)
        .bind(row.acct_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn run_sync_sql(&self, sql: &str) -> Result<()> {
        if sql.is_empty() {
            return Ok(());
        }
        info!("Running syncSQL");
        sqlx::raw_sql(sql).execute(&self.pool).await?;
        Ok(())
    }
}