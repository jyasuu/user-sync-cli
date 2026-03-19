-- Migration: create global_users table (matches the Java GlobalUsers entity)

CREATE TABLE IF NOT EXISTS global_users (
    pccuid        NUMERIC(20, 0) PRIMARY KEY,
    id            NUMERIC(20, 0) NOT NULL,
    sso_acct      VARCHAR(50)    NOT NULL,
    fact_no       VARCHAR(10),
    local_fact_no VARCHAR(10),
    chinese_nm    VARCHAR(300),
    local_pnl_nm  VARCHAR(300),
    english_nm    VARCHAR(200),
    contact_mail  VARCHAR(200),
    sex           VARCHAR(1),
    lo_posi_nm    VARCHAR(60),
    disabled      VARCHAR(1)     NOT NULL DEFAULT 'N',
    disabled_date TIMESTAMPTZ,
    update_date   TIMESTAMPTZ    NOT NULL,
    lo_dept_nm    VARCHAR(60),
    tel           VARCHAR(100)   NOT NULL DEFAULT '',
    leave_mk      VARCHAR(20),
    acct_type     VARCHAR(1)
);
