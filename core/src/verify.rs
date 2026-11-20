//! Step-up identity checks. Approving a purchase over the house limit requires a recent
//! verification of the human doing it, so "it's Maya" typed into a chat is never enough.
//! Provider is Persona when configured; otherwise a simulation that is labelled as such everywhere.

use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDateTime};
use serde::Serialize;
use serde_json::json;

use crate::db;
use crate::events::{self, Event};
use crate::state::{self, now, AppState};

pub const VALID_MINUTES: i64 = 30;

#[derive(Clone, Debug, Serialize)]
pub struct Verification {
    pub id: String,
    pub member_id: String,
    pub plan_id: Option<String>,
    pub provider: String,
    pub inquiry_id: Option<String>,
    pub status: String,
    pub url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub verified_at: Option<NaiveDateTime>,
    pub valid_until: Option<NaiveDateTime>,
}

fn from_row(r: &sqlx::mysql::MySqlRow) -> Verification {
    let verified_at = db::dt(r, "verified_at");
    Verification {
        id: db::s(r, "id"),
        member_id: db::s(r, "member_id"),
        plan_id: db::opt_s(r, "plan_id"),
        provider: db::s(r, "provider"),
        inquiry_id: db::opt_s(r, "inquiry_id"),
        status: db::s(r, "status"),
        url: db::opt_s(r, "url"),
        created_at: db::dt(r, "created_at"),
        verified_at,
        valid_until: verified_at.map(|t| t + Duration::minutes(VALID_MINUTES)),
    }
}

const COLS: &str = "id, member_id, plan_id, provider, inquiry_id, status, url, created_at, verified_at";

pub async fn recent(st: &AppState, hh: &str) -> Result<Vec<Verification>> {
    let rows = sqlx::query(&format!("SELECT {COLS} FROM verifications WHERE household_id = ? ORDER BY created_at DESC LIMIT 20")).bind(hh).fetch_all(&st.db).await?;
    Ok(rows.iter().map(from_row).collect())
}

/// The member's current, unexpired verification, if any.
pub async fn current(st: &AppState, hh: &str, member_id: &str) -> Result<Option<Verification>> {
    Ok(recent(st, hh).await?.into_iter().find(|v| v.member_id == member_id && v.status == "approved" && v.valid_until.map_or(false, |t| t > now())))
}

pub async fn start(st: &AppState, hh: &str, member_id: &str, plan_id: Option<&str>) -> Result<Verification> {
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let id = format!("ver_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
    let (provider, inquiry_id, url) = if st.persona.configured() {
        let redirect = format!("{}/phone/{}", st.cfg.web_url, member.id);
        let (inq, url) = st.persona.create_inquiry(&format!("{hh}:{}", member.id), &redirect).await?;
        ("persona", Some(inq), Some(url))
    } else if st.persona.relay() {
        ("persona_relay", None, None)
    } else {
        ("simulated", None, None)
    };
    sqlx::query("INSERT INTO verifications (household_id, id, member_id, plan_id, provider, inquiry_id, status, url, created_at, verified_at) VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?, NULL)")
        .bind(hh).bind(&id).bind(&member.id).bind(plan_id).bind(provider).bind(&inquiry_id).bind(&url).bind(now())
        .execute(&st.db)
        .await?;
    events::record(
        &st.db,
        hh,
        Event::new("verification_started", "plan", plan_id.unwrap_or(&member.id), format!("Asked {} to verify their identity ({})", member.name, match provider { "persona" => "Persona", "persona_relay" => "Persona Relay", _ => "simulated: Persona not connected" }))
            .actor(&member.id)
            .data(json!({ "provider": provider, "inquiry_id": inquiry_id, "verification_id": id })),
    )
    .await?;
    st.notify(hh);
    get(st, hh, &id).await
}

pub async fn get(st: &AppState, hh: &str, id: &str) -> Result<Verification> {
    let row = sqlx::query(&format!("SELECT {COLS} FROM verifications WHERE household_id = ? AND id = ?")).bind(hh).bind(id).fetch_optional(&st.db).await?.ok_or_else(|| anyhow!("no verification {id}"))?;
    Ok(from_row(&row))
}

async fn settle(st: &AppState, hh: &str, v: &Verification, approved: bool, detail: &str) -> Result<()> {
    sqlx::query("UPDATE verifications SET status = ?, verified_at = ? WHERE household_id = ? AND id = ?")
        .bind(if approved { "approved" } else { "declined" })
        .bind(if approved { Some(now()) } else { None })
        .bind(hh)
        .bind(&v.id)
        .execute(&st.db)
        .await?;
    let name = state::find_member(&st.db, hh, &v.member_id).await?.map(|m| m.name).unwrap_or_else(|| v.member_id.clone());
    events::record(
        &st.db,
        hh,
        Event::new(
            if approved { "identity_verified" } else { "identity_declined" },
            "plan",
            v.plan_id.as_deref().unwrap_or(&v.member_id),
            if approved { format!("{name}: a live human confirmed behind this approval ({detail}); valid {VALID_MINUTES} min") } else { format!("{name} was not verified ({detail})") },
        )
        .actor(&v.member_id)
        .provenance("observed")
        .data(json!({ "provider": v.provider, "inquiry_id": v.inquiry_id, "verification_id": v.id })),
    )
    .await?;
    st.notify(hh);
    Ok(())
}

/// Poll Persona for a pending inquiry. Simulated verifications are settled by `simulate`.
pub async fn check(st: &AppState, hh: &str, id: &str) -> Result<Verification> {
    let v = get(st, hh, id).await?;
    if v.status == "pending" && v.provider == "persona" {
        if let Some(inq) = &v.inquiry_id {
            match st.persona.inquiry_status(inq).await?.as_str() {
                "approved" | "completed" => settle(st, hh, &v, true, &format!("Persona inquiry {inq}{}", if st.persona.sandbox() { ", sandbox" } else { "" })).await?,
                "declined" | "failed" | "expired" => settle(st, hh, &v, false, &format!("Persona inquiry {inq}")).await?,
                _ => {}
            }
        }
    }
    get(st, hh, id).await
}

/// Result of a Persona Relay claim, reported by the dashboard's server after it redeemed the claim.
/// A Relay claim proves a live human is present; it does not say which human.
pub async fn attest(st: &AppState, hh: &str, id: &str, passed: bool, claim_type: &str) -> Result<Verification> {
    let v = get(st, hh, id).await?;
    anyhow::ensure!(v.provider == "persona_relay", "this verification does not use Persona Relay");
    if v.status == "pending" {
        settle(st, hh, &v, passed, &format!("Persona Relay claim {claim_type}: {}{}", if passed { "passed" } else { "failed" }, if st.persona.sandbox() { ", sandbox" } else { "" })).await?;
    }
    get(st, hh, id).await
}

/// Only available when Persona is not connected. Always labelled as simulated.
pub async fn simulate(st: &AppState, hh: &str, id: &str) -> Result<Verification> {
    let v = get(st, hh, id).await?;
    anyhow::ensure!(v.provider == "simulated", "this verification runs through Persona; it cannot be simulated");
    if v.status == "pending" {
        settle(st, hh, &v, true, "SIMULATED: Persona not connected").await?;
    }
    get(st, hh, id).await
}
