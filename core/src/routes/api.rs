//! HTTP surface: `/chat` + `/health` for Firedrill, `/api/*` for the dashboard and phone view.

use std::convert::Infallible;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Duration;
use futures::Stream;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::agent::{chat, planner};
use crate::db;
use crate::events;
use crate::plans;
use crate::seed;
use crate::state::{self, now, AppState};
use crate::{estimate, sourcing};

pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        ApiError(e.into())
    }
}

type Res = Result<Json<Value>, ApiError>;

#[derive(Deserialize, Default)]
pub struct Hh {
    household: Option<String>,
}

fn hh(st: &AppState, q: &Hh) -> String {
    q.household.clone().filter(|h| !h.is_empty()).unwrap_or_else(|| st.cfg.demo_household.clone())
}

pub fn router(st: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat_route))
        .route("/api/state", get(get_state))
        .route("/api/why/{subject_id}", get(why))
        .route("/api/plans/{id}/approve", post(approve))
        .route("/api/plans/{id}/skip", post(skip))
        .route("/api/items/{id}/correct", post(correct))
        .route("/api/invite/simulate", post(simulate_invite))
        .route("/api/request", post(request))
        .route("/api/receipts/ingest", post(ingest_receipt))
        .route("/api/airbyte/sync", post(airbyte_sync))
        .route("/api/verify/start", post(verify_start))
        .route("/api/verify/{id}/check", post(verify_check))
        .route("/api/verify/{id}/simulate", post(verify_simulate))
        .route("/api/verify/{id}/attest", post(verify_attest))
        .route("/api/outings", post(outing_create))
        .route("/api/outings/{id}/rsvp", post(outing_rsvp))
        .route("/api/outings/{id}/book", post(outing_book))
        .route("/api/outings/{id}/venue-closed", post(outing_venue_closed))
        .route("/api/outings/{id}/check-venue", post(outing_check_venue))
        .route("/api/outings/{id}/book-ride", post(outing_book_ride))
        .route("/api/outings/{id}/send-confirmation", post(outing_send_confirmation))
        .route("/api/world/venue", post(world_venue))
        .route("/api/sessions", get(sessions))
        .route("/api/scan", post(scan))
        .route("/api/reset", post(reset))
        .route("/api/stream", get(stream))
        .route("/api/firedrill/reports", get(firedrill_reports))
        .route("/api/integrations", get(integrations))
        .with_state(st)
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

#[derive(Deserialize)]
struct ChatIn {
    message: String,
    #[serde(default)]
    session: Option<String>,
}

async fn chat_route(State(st): State<AppState>, Json(body): Json<ChatIn>) -> Res {
    let session = body.session.unwrap_or_else(|| "default".into());
    let reply = chat::handle(&st, &session, &body.message).await?;
    Ok(Json(json!({ "reply": reply, "session": session, "household": chat::household_for_session(&session) })))
}

async fn get_state(State(st): State<AppState>, Query(q): Query<Hh>) -> Res {
    let hh = hh(&st, &q);
    let t = now();
    let (cfg, members, orders, spent, items, plans, calendar, recent) = tokio::join!(
        st.sidecar.config(&hh),
        state::members(&st.db, &hh),
        state::orders(&st.db, &hh),
        state::spent_this_month(&st.db, &hh),
        estimate::item_statuses(&st.db, &hh, t),
        plans::list(&st, &hh),
        planner::calendar_events(&st, &hh),
        events::recent(&st.db, &hh, 160),
    );
    let (members, orders, spent, items, plans, calendar, recent) = (members?, orders?, spent?, items?, plans?, calendar?, recent?);
    let verifications = crate::verify::recent(&st, &hh).await?;
    let outings = crate::outings::list(&st, &hh).await?;
    // How much of the audit log exists and how much has been forwarded to analytics.
    let totals = sqlx::query("SELECT COUNT(*) AS n, CAST(COALESCE(SUM(forwarded), 0) AS SIGNED) AS sent, CAST(COALESCE(SUM(model IS NOT NULL), 0) AS SIGNED) AS model_steps, COALESCE(SUM(cost_credits), 0) AS credits FROM agent_events WHERE household_id = ?").bind(&hh).fetch_one(&st.db).await?;
    let receipts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM receipts WHERE household_id = ?").bind(&hh).fetch_one(&st.db).await.unwrap_or(0);
    // Row counts per table: this flat, and everything stored (simulated households included).
    let names = ["agent_events", "receipts", "purchases", "items", "calendar_events", "plans", "orders", "outings", "verifications", "members"];
    let mut sql: Vec<String> = names.iter().map(|t| format!("SELECT '{t}' AS t, CAST(COALESCE(SUM(household_id = ?), 0) AS SIGNED) AS here, COUNT(*) AS total FROM {t}")).collect();
    sql.push("SELECT '_products' AS t, 0 AS here, COUNT(*) AS total FROM products".into());
    sql.push("SELECT '_households' AS t, 0 AS here, COUNT(DISTINCT household_id) AS total FROM members".into());
    let joined = sql.join(" UNION ALL ");
    let mut q = sqlx::query(&joined);
    for _ in names { q = q.bind(&hh); }
    let counted = q.fetch_all(&st.db).await.unwrap_or_default();
    let total_of = |name: &str| counted.iter().find(|r| db::s(r, "t") == name).map(|r| db::i(r, "total")).unwrap_or(0);
    let tables: Vec<Value> = names.iter().filter_map(|t| counted.iter().find(|r| db::s(r, "t") == *t).map(|r| json!({ "table": t, "here": db::i(r, "here"), "all": db::i(r, "total") }))).collect();
    let (catalog, households) = (total_of("_products"), total_of("_households"));
    let kinds = sqlx::query("SELECT kind, COUNT(*) AS n FROM agent_events WHERE household_id = ? GROUP BY kind ORDER BY n DESC LIMIT 8").bind(&hh).fetch_all(&st.db).await.unwrap_or_default();
    let kinds: Vec<Value> = kinds.iter().map(|r| json!({ "kind": db::s(r, "kind"), "n": db::i(r, "n") })).collect();
    let synced: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM receipts WHERE household_id = ? AND source = 'airbyte'").bind(&hh).fetch_one(&st.db).await.unwrap_or(0);
    let (balances, settlements) = state::balances(&members, &orders);
    Ok(Json(json!({
        "household": { "id": hh, "name": "14 Alder St" },
        "now": now(),
        "budget": {
            "monthly_cents": st.cfg.monthly_budget_cents,
            "spent_cents": spent,
            "remaining_cents": st.cfg.monthly_budget_cents - spent,
            "auto_limit_cents": cfg.auto_limit_cents,
            "agent_version": cfg.agent_version,
            "config_source": cfg.source,
        },
        "payments": { "provider": st.payments.provider_name(), "test_mode": true },
        "world": { "mode": st.world.mode() },
        "members": members,
        "items": items,
        "calendar": calendar,
        "plans": plans,
        "orders": orders,
        "balances": balances,
        "settlements": settlements,
        "recent_events": recent,
        "verifications": verifications,
        "outings": outings,
        "database": { "tables": tables, "catalog_products": catalog, "households": households, "event_kinds": kinds },
        "totals": { "events": db::i(&totals, "n"), "events_forwarded": db::i(&totals, "sent"), "model_steps": db::i(&totals, "model_steps"), "model_credits": db::f(&totals, "credits"), "receipts": receipts, "receipts_synced": synced, "amplitude_configured": st.cfg.amplitude_key.is_some(), "statsig_connected": cfg.source == "statsig" },
        "identity": { "provider": if st.persona.configured() { "persona" } else if st.persona.relay() { "persona_relay" } else { "simulated" }, "sandbox": st.persona.sandbox(), "valid_minutes": crate::verify::VALID_MINUTES },
    })))
}

/// The audit trail for one subject (item, plan, event): its events plus the receipts behind an item.
async fn why(State(st): State<AppState>, Path(subject_id): Path<String>, Query(q): Query<Hh>) -> Res {
    let hh = hh(&st, &q);
    let mut evs = events::for_subject(&st.db, &hh, &subject_id).await?;
    let mut plan = Value::Null;
    if let Ok(p) = plans::get(&st, &hh, &subject_id).await {
        if let Some(t) = &p.trigger_ref {
            let mut trigger = events::for_subject(&st.db, &hh, t).await?;
            trigger.append(&mut evs);
            evs = trigger;
        }
        plan = serde_json::to_value(&p)?;
    }
    let rows = sqlx::query(
        "SELECT p.qty, p.unit_price_cents, p.purchased_at, p.evidence_span, p.verified, p.extractor,
                r.id AS receipt_id, r.source, r.source_ref, r.store, r.subject, r.raw_text, r.member_id
         FROM purchases p JOIN receipts r ON r.household_id = p.household_id AND r.id = p.receipt_id
         WHERE p.household_id = ? AND p.item_id = ? ORDER BY p.purchased_at DESC LIMIT 6",
    )
    .bind(&hh)
    .bind(&subject_id)
    .fetch_all(&st.db)
    .await?;
    let receipts: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "receipt_id": db::s(r, "receipt_id"),
                "source": db::s(r, "source"),
                "source_ref": db::opt_s(r, "source_ref"),
                "store": db::s(r, "store"),
                "subject": db::s(r, "subject"),
                "raw_text": db::s(r, "raw_text"),
                "member_id": db::s(r, "member_id"),
                "qty": db::i(r, "qty"),
                "unit_price_cents": db::i(r, "unit_price_cents"),
                "purchased_at": db::dt(r, "purchased_at"),
                "evidence_span": db::s(r, "evidence_span"),
                "verified": db::b(r, "verified"),
                "extractor": db::s(r, "extractor"),
            })
        })
        .collect();
    let item = estimate::item_statuses(&st.db, &hh, now()).await?.into_iter().find(|i| i.id == subject_id);
    let options = match &item {
        Some(i) => {
            let catalog = sourcing::products(&st.db).await?;
            catalog.iter().filter(|p| p.item_id == i.id).map(|p| serde_json::to_value(sourcing::option_for(p, None, now())).unwrap_or(Value::Null)).collect()
        }
        None => vec![],
    };
    Ok(Json(json!({ "subject_id": subject_id, "item": item, "plan": plan, "receipts": receipts, "options": options, "events": evs })))
}

#[derive(Deserialize)]
struct Decide {
    member: String,
    #[serde(default)]
    reason: String,
}

async fn approve(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Decide>) -> Res {
    let p = plans::approve(&st, &hh(&st, &q), &id, &b.member).await?;
    Ok(Json(json!({ "plan": p })))
}

async fn skip(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Decide>) -> Res {
    let p = plans::skip(&st, &hh(&st, &q), &id, &b.member, &b.reason).await?;
    Ok(Json(json!({ "plan": p })))
}

#[derive(Deserialize)]
struct Correct {
    member: String,
    level: String,
}

async fn correct(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Correct>) -> Res {
    let summary = planner::correct_stock(&st, &hh(&st, &q), &b.member, &id, &b.level).await?;
    Ok(Json(json!({ "ok": true, "summary": summary })))
}

#[derive(Deserialize)]
struct Invite {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    guests: Option<i64>,
    #[serde(default)]
    days_from_now: Option<i64>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    nights: Option<i64>,
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    source_ref: Option<String>,
}

/// An invite arrives (from Airbyte ingest via the sidecar, or the labelled "Simulate invite" button).
async fn simulate_invite(State(st): State<AppState>, Query(q): Query<Hh>, Json(b): Json<Invite>) -> Res {
    let hh = hh(&st, &q);
    let starts = (now() + Duration::days(b.days_from_now.unwrap_or(4))).date().and_hms_opt(19, 0, 0).unwrap();
    let id = planner::add_calendar_event(
        &st,
        &hh,
        b.title.as_deref().unwrap_or("Priya's birthday"),
        starts,
        b.guests.unwrap_or(12),
        b.kind.as_deref().unwrap_or("party"),
        b.nights.unwrap_or(0),
        b.host.as_deref().unwrap_or("priya"),
        b.source.as_deref().unwrap_or("simulated"),
        b.source_ref.as_deref(),
    )
    .await?;
    let plan = planner::plan_for_event(&st, &hh, &id, true).await?;
    Ok(Json(json!({ "event_id": id, "plan": plan })))
}

#[derive(Deserialize)]
struct Request {
    member: String,
    text: String,
    #[serde(default)]
    qty: Option<i64>,
}

async fn request(State(st): State<AppState>, Query(q): Query<Hh>, Json(b): Json<Request>) -> Res {
    let p = planner::handle_request(&st, &hh(&st, &q), &b.member, &b.text, b.qty.unwrap_or(1)).await?;
    Ok(Json(json!({ "plan": p })))
}

async fn ingest_receipt(State(st): State<AppState>, Query(q): Query<Hh>, Json(b): Json<crate::ingest::ReceiptIn>) -> Res {
    Ok(Json(crate::ingest::ingest_receipt(&st, &hh(&st, &q), b).await?))
}

async fn airbyte_sync(State(st): State<AppState>, Query(q): Query<Hh>) -> Res {
    Ok(Json(crate::ingest::airbyte_sync(&st, &hh(&st, &q)).await?))
}

#[derive(Deserialize)]
struct VerifyStart {
    member: String,
    #[serde(default)]
    plan_id: Option<String>,
}

async fn verify_start(State(st): State<AppState>, Query(q): Query<Hh>, Json(b): Json<VerifyStart>) -> Res {
    Ok(Json(json!({ "verification": crate::verify::start(&st, &hh(&st, &q), &b.member, b.plan_id.as_deref()).await? })))
}

async fn verify_check(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>) -> Res {
    Ok(Json(json!({ "verification": crate::verify::check(&st, &hh(&st, &q), &id).await? })))
}

#[derive(Deserialize)]
struct Attest {
    passed: bool,
    #[serde(default)]
    claim_type: String,
}

async fn verify_attest(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, headers: axum::http::HeaderMap, Json(b): Json<Attest>) -> Res {
    let key = headers.get("x-internal-key").and_then(|v| v.to_str().ok()).unwrap_or_default();
    if !st.persona.key_matches(key) {
        return Err(anyhow::anyhow!("not authorised to attest verifications").into());
    }
    Ok(Json(json!({ "verification": crate::verify::attest(&st, &hh(&st, &q), &id, b.passed, &b.claim_type).await? })))
}

async fn verify_simulate(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>) -> Res {
    Ok(Json(json!({ "verification": crate::verify::simulate(&st, &hh(&st, &q), &id).await? })))
}

async fn outing_create(State(st): State<AppState>, Query(q): Query<Hh>, Json(b): Json<crate::outings::NewOuting>) -> Res {
    Ok(Json(json!({ "outing": crate::outings::create(&st, &hh(&st, &q), b).await? })))
}

#[derive(Deserialize)]
struct Rsvp {
    member: String,
    status: String,
}

async fn outing_rsvp(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Rsvp>) -> Res {
    Ok(Json(json!({ "outing": crate::outings::rsvp(&st, &hh(&st, &q), &id, &b.member, &b.status).await? })))
}

async fn outing_venue_closed(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Decide>) -> Res {
    Ok(Json(json!({ "outing": crate::outings::venue_closed(&st, &hh(&st, &q), &id, &b.member).await? })))
}

async fn outing_check_venue(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Decide>) -> Res {
    Ok(Json(crate::agent::world_tools::check_venue(&st, &hh(&st, &q), &id, &b.member).await?))
}

async fn outing_book_ride(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>) -> Res {
    Ok(Json(crate::agent::world_tools::book_ride(&st, &hh(&st, &q), &id).await?))
}

async fn outing_send_confirmation(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>) -> Res {
    Ok(Json(crate::agent::world_tools::send_confirmation(&st, &hh(&st, &q), &id).await?))
}

#[derive(Deserialize)]
struct VenueState {
    venue: String,
    closed: bool,
}

/// Demo control for the built-in simulated web: put up or take down a venue's closure notice.
async fn world_venue(State(st): State<AppState>, Json(b): Json<VenueState>) -> Res {
    st.world.set_closed(&b.venue, b.closed).await;
    Ok(Json(json!({ "ok": true, "venue": b.venue, "closed": b.closed, "mode": st.world.mode() })))
}

async fn outing_book(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<Hh>, Json(b): Json<Decide>) -> Res {
    Ok(Json(json!({ "outing": crate::outings::book(&st, &hh(&st, &q), &id, &b.member).await? })))
}

/// Recent agent conversations with every tool call, across all session households.
async fn sessions(State(st): State<AppState>) -> Res {
    let rows = sqlx::query(
        "SELECT household_id, id, ts, kind, subject_id, summary, data, model, latency_us, cost_credits FROM agent_events
         WHERE subject_type = 'session' ORDER BY ts DESC LIMIT 300",
    )
    .fetch_all(&st.db)
    .await?;
    let mut by_session: Vec<(String, Vec<Value>)> = vec![];
    for r in rows.iter().rev() {
        let session = db::s(r, "subject_id");
        let ev = json!({ "id": db::s(r, "id"), "ts": db::dt(r, "ts"), "kind": db::s(r, "kind"), "summary": db::s(r, "summary"), "data": db::j(r, "data"), "model": db::opt_s(r, "model"), "latency_us": db::i(r, "latency_us"), "cost_credits": db::f(r, "cost_credits") });
        match by_session.iter_mut().find(|(s, _)| *s == session) {
            Some((_, v)) => v.push(ev),
            None => by_session.push((session, vec![ev])),
        }
    }
    by_session.reverse();
    Ok(Json(json!({ "sessions": by_session.into_iter().take(12).map(|(s, events)| json!({ "session": s, "events": events })).collect::<Vec<_>>() })))
}

async fn scan(State(st): State<AppState>, Query(q): Query<Hh>) -> Res {
    let p = plans::restock_scan(&st, &hh(&st, &q)).await?;
    Ok(Json(json!({ "plan": p })))
}

async fn reset(State(st): State<AppState>, Query(q): Query<Hh>) -> Res {
    let hh = hh(&st, &q);
    seed::seed_household(&st.db, &hh).await?;
    st.notify(&hh);
    Ok(Json(json!({ "ok": true, "household": hh })))
}

async fn stream(State(st): State<AppState>) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = st.changes.subscribe();
    let s = BroadcastStream::new(rx).filter_map(|m| m.ok()).map(|hh| Ok(SseEvent::default().event("change").data(hh)));
    Sse::new(s).keep_alive(KeepAlive::default())
}

/// Firedrill campaign reports saved under firedrill/reports/*.json (rendered by the Tests tab).
async fn firedrill_reports() -> Res {
    let mut out = vec![];
    for dir in ["../firedrill/reports", "firedrill/reports"] {
        let Ok(mut rd) = tokio::fs::read_dir(dir).await else { continue };
        while let Ok(Some(e)) = rd.next_entry().await {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            if let Ok(text) = tokio::fs::read_to_string(&path).await {
                let name = path.file_stem().and_then(|x| x.to_str()).unwrap_or_default().to_string();
                out.push(json!({ "name": name, "report": serde_json::from_str::<Value>(&text).unwrap_or(json!({ "raw": text })) }));
            }
        }
        break;
    }
    out.sort_by_key(|v| v.get("name").and_then(Value::as_str).unwrap_or_default().to_string());
    Ok(Json(json!({ "reports": out })))
}

async fn integrations(State(st): State<AppState>) -> Res {
    Ok(Json(json!({
        "singlestore": { "connected": true, "host": st.cfg.db_host },
        "sie": { "configured": st.sie.enabled(), "planner_model": st.sie.planner_model, "decide_model": st.sie.decide_model },
        "amplitude": { "configured": st.cfg.amplitude_key.is_some() },
        "senso": { "configured": st.cfg.senso_key.is_some() },
        "statsig": { "configured": st.cfg.statsig_key.is_some(), "contingency_gate": crate::integrations::statsig::GATE_CONTINGENCY, "gate": crate::integrations::statsig::GATE_MODEL_PLANNER, "config": crate::integrations::statsig::CONFIG_HOUSE_RULES },
        "persona": { "configured": st.persona.configured() || st.persona.relay(), "mode": if st.persona.relay() { "relay" } else { "inquiry" }, "sandbox": st.persona.sandbox() },
        "payments": { "provider": st.payments.provider_name() },
        "sidecar": st.sidecar.health().await,
    })))
}
