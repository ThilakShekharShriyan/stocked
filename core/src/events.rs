//! The append-only `agent_events` table: every step the agent takes is recorded here.
//! It is the single source for the Why drawer, the ledger history, Amplitude forwarding and replay.

use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};

use crate::db::{self, Db};

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub kind: String,
    pub subject_type: String,
    pub subject_id: String,
    pub actor: String,
    pub summary: String,
    pub data: Value,
    pub model: Option<String>,
    pub latency_us: i64,
    pub cost_credits: f64,
    pub provenance: String,
}

impl Event {
    pub fn new(kind: &str, subject_type: &str, subject_id: &str, summary: impl Into<String>) -> Self {
        Event {
            kind: kind.into(),
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            actor: "agent".into(),
            summary: summary.into(),
            data: json!({}),
            model: None,
            latency_us: 0,
            cost_credits: 0.0,
            provenance: "computed".into(),
        }
    }
    pub fn actor(mut self, a: &str) -> Self {
        self.actor = a.into();
        self
    }
    pub fn data(mut self, d: Value) -> Self {
        self.data = d;
        self
    }
    pub fn model(mut self, m: &str, latency_us: i64, cost: f64) -> Self {
        self.model = Some(m.into());
        self.latency_us = latency_us;
        self.cost_credits = cost;
        self
    }
    pub fn latency(mut self, us: i64) -> Self {
        self.latency_us = us;
        self
    }
    pub fn provenance(mut self, p: &str) -> Self {
        self.provenance = p.into();
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StoredEvent {
    pub id: String,
    pub ts: Option<NaiveDateTime>,
    #[serde(flatten)]
    pub event: Event,
}

pub async fn record(db: &Db, household: &str, e: Event) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO agent_events (household_id, id, ts, kind, subject_type, subject_id, actor, summary, data, model, latency_us, cost_credits, provenance, forwarded)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(household)
    .bind(&id)
    .bind(Utc::now().naive_utc())
    .bind(&e.kind)
    .bind(&e.subject_type)
    .bind(&e.subject_id)
    .bind(&e.actor)
    .bind(&e.summary)
    .bind(e.data.to_string())
    .bind(&e.model)
    .bind(e.latency_us)
    .bind(e.cost_credits)
    .bind(&e.provenance)
    .execute(db)
    .await?;
    Ok(id)
}

fn from_row(r: &sqlx::mysql::MySqlRow) -> StoredEvent {
    StoredEvent {
        id: db::s(r, "id"),
        ts: db::dt(r, "ts"),
        event: Event {
            kind: db::s(r, "kind"),
            subject_type: db::s(r, "subject_type"),
            subject_id: db::s(r, "subject_id"),
            actor: db::s(r, "actor"),
            summary: db::s(r, "summary"),
            data: db::j(r, "data"),
            model: db::opt_s(r, "model"),
            latency_us: db::i(r, "latency_us"),
            cost_credits: db::f(r, "cost_credits"),
            provenance: db::s(r, "provenance"),
        },
    }
}

const COLS: &str = "id, ts, kind, subject_type, subject_id, actor, summary, data, model, latency_us, cost_credits, provenance";

pub async fn for_subject(db: &Db, household: &str, subject_id: &str) -> Result<Vec<StoredEvent>> {
    let rows = sqlx::query(&format!(
        "SELECT {COLS} FROM agent_events WHERE household_id = ? AND subject_id = ? ORDER BY ts"
    ))
    .bind(household)
    .bind(subject_id)
    .fetch_all(db)
    .await?;
    Ok(rows.iter().map(from_row).collect())
}

pub async fn recent(db: &Db, household: &str, limit: i64) -> Result<Vec<StoredEvent>> {
    let rows = sqlx::query(&format!(
        "SELECT {COLS} FROM agent_events WHERE household_id = ? ORDER BY ts DESC LIMIT ?"
    ))
    .bind(household)
    .bind(limit)
    .fetch_all(db)
    .await?;
    Ok(rows.iter().map(from_row).collect())
}

/// Events not yet sent to Amplitude (any household).
pub async fn unforwarded(db: &Db, limit: i64) -> Result<Vec<(String, StoredEvent)>> {
    let rows = sqlx::query(&format!(
        "SELECT household_id, {COLS} FROM agent_events WHERE forwarded = 0 ORDER BY ts LIMIT ?"
    ))
    .bind(limit)
    .fetch_all(db)
    .await?;
    Ok(rows.iter().map(|r| (db::s(r, "household_id"), from_row(r))).collect())
}

pub async fn mark_forwarded(db: &Db, ids: &[String]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!("UPDATE agent_events SET forwarded = 1 WHERE id IN ({placeholders})");
    let mut q = sqlx::query(&sql);
    for id in ids {
        q = q.bind(id);
    }
    q.execute(db).await?;
    Ok(())
}
