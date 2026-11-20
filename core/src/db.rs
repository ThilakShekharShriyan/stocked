//! SingleStore access: connection pool, schema bootstrap, and tolerant row decoding.
//!
//! SingleStore speaks the MySQL wire protocol. Column types (JSON, BOOL as TINYINT) are decoded
//! "unchecked" so we don't depend on exact type metadata.

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlRow, MySqlSslMode};
use sqlx::{Executor, Row};

use crate::config::Config;

pub type Db = MySqlPool;

fn options(cfg: &Config, with_db: bool) -> MySqlConnectOptions {
    let mut o = MySqlConnectOptions::new()
        .host(&cfg.db_host)
        .port(cfg.db_port)
        .username(&cfg.db_user)
        .password(&cfg.db_password)
        .ssl_mode(MySqlSslMode::Required)
        // Keep the connection handshake to statements SingleStore accepts.
        .pipes_as_concat(false)
        .no_engine_substitution(false)
        .set_names(false)
        .timezone(None);
    if with_db {
        o = o.database(&cfg.db_name);
    }
    o
}

pub async fn connect(cfg: &Config) -> Result<Db> {
    anyhow::ensure!(!cfg.db_host.is_empty(), "SINGLESTORE_HOST is not set");
    // Create the database first (dedicated workspaces start empty).
    let bootstrap = MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(options(cfg, false))
        .await
        .context("connecting to SingleStore")?;
    bootstrap
        .execute(format!("CREATE DATABASE IF NOT EXISTS {}", cfg.db_name).as_str())
        .await
        .context("creating database")?;
    bootstrap.close().await;

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect_with(options(cfg, true))
        .await
        .context("connecting to SingleStore database")?;
    Ok(pool)
}

/// Apply db/schema.sql (idempotent: CREATE TABLE IF NOT EXISTS).
pub async fn apply_schema(db: &Db) -> Result<()> {
    let sql = include_str!("../../db/schema.sql");
    let cleaned: String = sql
        .lines()
        .filter(|l| !l.trim_start().starts_with("--"))
        .map(|l| match l.find(" -- ") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    for stmt in cleaned.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        db.execute(stmt).await.with_context(|| format!("schema statement failed: {stmt}"))?;
    }
    // Columns added after the first deploy; errors mean the column already exists.
    for alter in [
        "ALTER TABLE calendar_events ADD COLUMN kind VARCHAR(24) NOT NULL DEFAULT 'party'",
        "ALTER TABLE calendar_events ADD COLUMN nights INT NOT NULL DEFAULT 0",
    ] {
        let _ = db.execute(alter).await;
    }
    Ok(())
}

// ---- tolerant decoders -------------------------------------------------------------------

pub fn s(row: &MySqlRow, col: &str) -> String {
    row.try_get_unchecked::<Option<String>, _>(col).ok().flatten().unwrap_or_default()
}

pub fn opt_s(row: &MySqlRow, col: &str) -> Option<String> {
    row.try_get_unchecked::<Option<String>, _>(col).ok().flatten()
}

pub fn i(row: &MySqlRow, col: &str) -> i64 {
    if let Ok(Some(v)) = row.try_get_unchecked::<Option<i64>, _>(col) {
        return v;
    }
    if let Ok(Some(v)) = row.try_get_unchecked::<Option<i32>, _>(col) {
        return v as i64;
    }
    if let Ok(Some(v)) = row.try_get_unchecked::<Option<i8>, _>(col) {
        return v as i64;
    }
    if let Ok(Some(v)) = row.try_get_unchecked::<Option<f64>, _>(col) {
        return v as i64;
    }
    s(row, col).parse().unwrap_or(0)
}

pub fn f(row: &MySqlRow, col: &str) -> f64 {
    if let Ok(Some(v)) = row.try_get_unchecked::<Option<f64>, _>(col) {
        return v;
    }
    i(row, col) as f64
}

pub fn b(row: &MySqlRow, col: &str) -> bool {
    i(row, col) != 0
}

pub fn dt(row: &MySqlRow, col: &str) -> Option<NaiveDateTime> {
    row.try_get_unchecked::<Option<NaiveDateTime>, _>(col).ok().flatten()
}

pub fn j(row: &MySqlRow, col: &str) -> Value {
    serde_json::from_str(&s(row, col)).unwrap_or(Value::Null)
}
