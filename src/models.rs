use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::Deserialize;

// ── API response types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UsersResponse {
    pub timestamp: i64,
    pub status_code: i64,
    pub data: Vec<User>,
    pub statistic: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub pccuid: i64,
    pub sex: Option<String>,
    pub disabled: bool,
    pub sso_acct: String,
    pub fact_no: Option<String>,
    pub local_fact_no: Option<String>,
    pub chinese_nm: Option<String>,
    pub local_pnl_nm: Option<String>,
    pub english_nm: Option<String>,
    pub contact_mail: Option<String>,
    pub lo_posi_nm: Option<String>,
    /// Epoch milliseconds; None when not disabled
    pub disabled_date: Option<i64>,
    /// Epoch milliseconds
    pub update_date: i64,
    pub lo_dept_nm: Option<String>,
    pub tel: Option<String>,
    pub leave_mk: Option<String>,
    pub enable_time: Option<i64>,
    pub acct_type: Option<i64>,
}

// ── DB row ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct GlobalUserRow {
    pub pccuid: BigDecimal,
    pub sso_acct: String,
    pub fact_no: Option<String>,
    pub local_fact_no: Option<String>,
    pub chinese_nm: Option<String>,
    pub local_pnl_nm: Option<String>,
    pub english_nm: Option<String>,
    pub contact_mail: String,
    pub sex: Option<String>,
    pub lo_posi_nm: Option<String>,
    pub disabled: String,
    pub disabled_date: Option<DateTime<Utc>>,
    pub update_date: DateTime<Utc>,
    pub lo_dept_nm: Option<String>,
    pub tel: String,
    pub leave_mk: Option<String>,
    pub acct_type: Option<String>,
}

impl From<&User> for GlobalUserRow {
    fn from(u: &User) -> Self {
        let ms_to_dt = |ms: i64| -> DateTime<Utc> {
            DateTime::from_timestamp_millis(ms).unwrap_or(Utc::now())
        };

        GlobalUserRow {
            pccuid: BigDecimal::from(u.pccuid),
            sso_acct: u.sso_acct.clone(),
            fact_no: u.fact_no.clone(),
            local_fact_no: u.local_fact_no.clone(),
            chinese_nm: u.chinese_nm.clone(),
            local_pnl_nm: u.local_pnl_nm.clone(),
            english_nm: u.english_nm.clone(),
            contact_mail: u.contact_mail.clone().unwrap_or_default(),
            sex: u.sex.clone(),
            lo_posi_nm: u.lo_posi_nm.clone(),
            disabled: if u.disabled { "Y".into() } else { "N".into() },
            disabled_date: u.disabled_date.map(ms_to_dt),
            update_date: ms_to_dt(u.update_date),
            lo_dept_nm: u.lo_dept_nm.clone(),
            tel: u.tel.clone().unwrap_or_default(),
            leave_mk: u.leave_mk.clone(),
            acct_type: u.acct_type.map(|v| v.to_string()),
        }
    }
}
