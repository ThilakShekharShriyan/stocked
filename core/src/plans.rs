//! Plans: propose → verify → (auto-approve | hold | refuse) → approve/skip → place order.
//! Phone taps and the `/chat` agent both call these functions, so Firedrill tests exactly
//! the code path that is demoed.

use std::time::Instant;

use anyhow::{anyhow, bail, Result};
use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::{json, Value};

use crate::checks::{self, Line, Outcome, Verdict};
use crate::db;
use crate::estimate::{self, ItemStatus};
use crate::events::{self, Event};
use crate::sourcing::{self, money};
use crate::state::{self, now, AppState};

#[derive(Clone, Debug, Serialize)]
pub struct Plan {
    pub id: String,
    pub version: i64,
    pub kind: String,
    pub title: String,
    pub trigger_ref: Option<String>,
    pub requested_by: Option<String>,
    pub basket: Vec<Line>,
    pub verdict: Value,
    pub total_cents: i64,
    pub status: String,
    pub agent_version: String,
    pub created_at: Option<NaiveDateTime>,
    pub decided_by: Option<String>,
    pub decided_at: Option<NaiveDateTime>,
    pub decision_note: Option<String>,
}

const COLS: &str = "id, version, kind, title, trigger_ref, requested_by, basket, checks, total_cents, status, agent_version, created_at, decided_by, decided_at, decision_note";

fn from_row(r: &sqlx::mysql::MySqlRow) -> Plan {
    Plan {
        id: db::s(r, "id"),
        version: db::i(r, "version"),
        kind: db::s(r, "kind"),
        title: db::s(r, "title"),
        trigger_ref: db::opt_s(r, "trigger_ref"),
        requested_by: db::opt_s(r, "requested_by"),
        basket: serde_json::from_value(db::j(r, "basket")).unwrap_or_default(),
        verdict: db::j(r, "checks"),
        total_cents: db::i(r, "total_cents"),
        status: db::s(r, "status"),
        agent_version: db::s(r, "agent_version"),
        created_at: db::dt(r, "created_at"),
        decided_by: db::opt_s(r, "decided_by"),
        decided_at: db::dt(r, "decided_at"),
        decision_note: db::opt_s(r, "decision_note"),
    }
}

pub async fn list(st: &AppState, hh: &str) -> Result<Vec<Plan>> {
    let rows = sqlx::query(&format!("SELECT {COLS} FROM plans WHERE household_id = ? ORDER BY created_at DESC"))
        .bind(hh)
        .fetch_all(&st.db)
        .await?;
    Ok(rows.iter().map(from_row).collect())
}

pub async fn get(st: &AppState, hh: &str, id: &str) -> Result<Plan> {
    let row = sqlx::query(&format!("SELECT {COLS} FROM plans WHERE household_id = ? AND id = ?"))
        .bind(hh)
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or_else(|| anyhow!("no plan {id}"))?;
    Ok(from_row(&row))
}

async fn set_status(st: &AppState, hh: &str, id: &str, status: &str, by: Option<&str>, note: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE plans SET status = ?, decided_by = COALESCE(?, decided_by), decided_at = ?, decision_note = COALESCE(?, decision_note) WHERE household_id = ? AND id = ?")
        .bind(status)
        .bind(by)
        .bind(now())
        .bind(note)
        .bind(hh)
        .bind(id)
        .execute(&st.db)
        .await?;
    Ok(())
}

/// A proposal from the model (or a rule): item + quantity + why.
#[derive(Clone, Debug)]
pub struct Proposal {
    pub item_id: String,
    pub qty: i64,
    pub why: String,
}

/// Turn proposals into priced, sourced lines using the catalog. Unknown items are dropped
/// (and reported), never invented.
pub async fn price_lines(
    st: &AppState,
    items: &[ItemStatus],
    proposals: &[Proposal],
    deadline: Option<NaiveDateTime>,
) -> Result<(Vec<Line>, Vec<String>)> {
    let catalog = sourcing::products(&st.db).await?;
    let now = now();
    let mut lines = vec![];
    let mut dropped = vec![];
    for p in proposals {
        let Some(item) = items.iter().find(|i| i.id == p.item_id) else {
            dropped.push(format!("{}: not a known household item", p.item_id));
            continue;
        };
        let choice = sourcing::choose(&item.id, &catalog, deadline, now);
        lines.push(Line {
            item_id: item.id.clone(),
            name: item.name.clone(),
            qty: p.qty,
            unit_price_cents: choice.as_ref().map(|c| c.product.price_cents).unwrap_or(0),
            is_shared_consumable: item.is_shared_consumable,
            product_id: choice.as_ref().map(|c| c.product.id.clone()),
            store: choice.as_ref().map(|c| c.product.store.clone()),
            channel: choice.as_ref().map(|c| c.product.channel.clone()),
            sourcing_note: choice.as_ref().map(|c| c.note.clone()).unwrap_or_else(|| "No store carries this".into()),
            arrives_in_time: choice.as_ref().map(|c| c.feasible),
            why: p.why.clone(),
        });
    }
    Ok((lines, dropped))
}

pub struct NewPlan {
    pub kind: String,
    pub title: String,
    pub trigger_ref: Option<String>,
    pub requested_by: Option<String>,
    pub lines: Vec<Line>,
    pub claimed_total_cents: Option<i64>,
    /// Extra checks computed before verification (e.g. the SIE decision gate).
    pub extra_checks: Vec<checks::Check>,
    pub force_refuse: Option<String>,
    pub context: Value,
}

/// Verify and store a plan; auto-approved plans are ordered immediately.
pub async fn propose(st: &AppState, hh: &str, np: NewPlan) -> Result<Plan> {
    let cfg = st.sidecar.config(hh).await;
    let members = state::members(&st.db, hh).await?;
    let spent = state::spent_this_month(&st.db, hh).await?;
    let started = Instant::now();
    let mut verdict: Verdict = checks::evaluate(&checks::Input {
        lines: np.lines.clone(),
        budget_cents: st.cfg.monthly_budget_cents,
        spent_cents: spent,
        auto_limit_cents: cfg.auto_limit_cents,
        has_approver: members.iter().any(|m| m.role == "approver"),
        claimed_total_cents: np.claimed_total_cents,
    });
    let check_us = started.elapsed().as_micros() as i64;
    verdict.checks.splice(0..0, np.extra_checks.clone());
    if let Some(reason) = &np.force_refuse {
        verdict.outcome = Outcome::Refuse;
        if !verdict.reasons.contains(reason) {
            verdict.reasons.insert(0, reason.clone());
        }
        verdict.reasons.retain(|r| r != "All checks passed");
    }
    let nothing = np.lines.is_empty() && np.force_refuse.is_none();
    if nothing {
        verdict.reasons = vec!["Nothing to buy: the flat has it, or another plan already covers it".into()];
        verdict.checks.retain(|c| c.name != "basket_not_empty");
    }
    let status = match verdict.outcome {
        _ if nothing => "nothing_to_buy",
        Outcome::AutoApprove => "auto_approved",
        Outcome::Hold => "held",
        Outcome::Refuse => "refused",
    };
    let id = format!("plan_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
    sqlx::query("INSERT INTO plans (household_id, id, version, kind, title, trigger_ref, requested_by, basket, checks, total_cents, status, agent_version, created_at, decided_by, decided_at, decision_note) VALUES (?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(hh)
        .bind(&id)
        .bind(&np.kind)
        .bind(&np.title)
        .bind(&np.trigger_ref)
        .bind(&np.requested_by)
        .bind(serde_json::to_string(&np.lines)?)
        .bind(serde_json::to_string(&verdict)?)
        .bind(verdict.total_cents)
        .bind(status)
        .bind(&cfg.agent_version)
        .bind(now())
        .bind(if status == "held" { None } else { Some("agent") })
        .bind(if status == "held" { None } else { Some(now()) })
        .bind(Some(verdict.reasons.join("; ")))
        .execute(&st.db)
        .await?;

    events::record(
        &st.db,
        hh,
        Event::new("plan_proposed", "plan", &id, format!("Proposed “{}”: {} item(s), {}", np.title, np.lines.len(), money(verdict.total_cents)))
            .data(json!({ "agent_version": cfg.agent_version, "kind": np.kind, "lines": np.lines, "context": np.context, "total_cents": verdict.total_cents, "config_source": cfg.source })),
    )
    .await?;
    let (kind, summary) = match status {
        "nothing_to_buy" => ("nothing_to_buy", "Nothing to buy: the flat has it, or another plan already covers it".to_string()),
        "auto_approved" => ("plan_auto_approved", format!("Auto-approved: {} is under the {} limit", money(verdict.total_cents), money(cfg.auto_limit_cents))),
        "held" => ("plan_held", format!("Waiting for approval: {}", verdict.reasons.join("; "))),
        _ => ("plan_refused", format!("Refused: {}", verdict.reasons.join("; "))),
    };
    events::record(
        &st.db,
        hh,
        Event::new(kind, "plan", &id, summary)
            .data(json!({ "agent_version": cfg.agent_version, "checks": verdict.checks, "outcome": verdict.outcome, "total_cents": verdict.total_cents, "auto_limit_cents": cfg.auto_limit_cents, "spent_cents": spent }))
            .latency(check_us),
    )
    .await?;
    st.notify(hh);

    if status == "auto_approved" {
        let plan = get(st, hh, &id).await?;
        let _ = place_order(st, hh, &plan, "agent").await;
    }
    get(st, hh, &id).await
}

/// True when a plan passed every check on its own and only failed at the payment step.
fn failed_after_auto_approval(plan: &Plan) -> bool {
    plan.status == "payment_failed" && plan.verdict.get("outcome").and_then(Value::as_str) == Some("auto_approve")
}

/// Retry the payment of a plan that never needed approval. Found by the Firedrill outage drill:
/// in v0 the retry went through the approver path, so a flatmate could never finish a small order
/// after an outage. From v1 on, anyone may retry it; the idempotency key still allows one charge.
pub async fn retry_payment(st: &AppState, hh: &str, plan_id: &str, actor: &str) -> Result<Plan> {
    let plan = get(st, hh, plan_id).await?;
    anyhow::ensure!(failed_after_auto_approval(&plan), "this plan cannot be retried without approval");
    events::record(&st.db, hh, Event::new("payment_retried", "plan", plan_id, format!("Retrying the payment for “{}”; it never needed approval", plan.title)).actor(actor).data(json!({ "agent_version": plan.agent_version }))).await?;
    let _ = place_order(st, hh, &plan, actor).await;
    get(st, hh, plan_id).await
}

pub async fn approve(st: &AppState, hh: &str, plan_id: &str, member_id: &str) -> Result<Plan> {
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let plan = get(st, hh, plan_id).await?;
    if failed_after_auto_approval(&plan) && st.sidecar.config(hh).await.agent_version != "v0" {
        return retry_payment(st, hh, plan_id, &member.id).await;
    }
    if member.role != "approver" {
        events::record(
            &st.db,
            hh,
            Event::new("approval_rejected", "plan", plan_id, format!("{} tried to approve, but only the house approver can", member.name)).actor(&member.id).provenance("observed"),
        )
        .await?;
        st.notify(hh);
        bail!("Only the house approver can approve purchases over the limit; {} is not the approver", member.name);
    }
    // Step-up: the human behind an over-limit approval must be freshly verified.
    if matches!(plan.status.as_str(), "held" | "payment_failed") {
        if crate::verify::current(st, hh, &member.id).await?.is_none() {
            events::record(
                &st.db,
                hh,
                Event::new("approval_blocked_unverified", "plan", plan_id, format!("Approval as {} blocked: identity not verified", member.name)).actor(&member.id).provenance("observed"),
            )
            .await?;
            st.notify(hh);
            bail!("VERIFICATION_REQUIRED: {} must verify their identity before approving {}. Open {}/phone/{} and tap “Verify it’s you”.", member.name, money(plan.total_cents), st.cfg.web_url, member.id);
        }
    }
    match plan.status.as_str() {
        "held" => {}
        "payment_failed" | "approved" => return place_order(st, hh, &plan, &member.id).await.map(|_| plan),
        "ordered" | "auto_approved" => return Ok(plan),
        other => bail!("Plan is {other}; it can't be approved"),
    }
    set_status(st, hh, plan_id, "approved", Some(&member.id), None).await?;
    events::record(
        &st.db,
        hh,
        Event::new("plan_approved", "plan", plan_id, format!("{} approved {}", member.name, money(plan.total_cents)))
            .actor(&member.id)
            .provenance("observed")
            .data(json!({ "agent_version": plan.agent_version, "total_cents": plan.total_cents })),
    )
    .await?;
    let plan = get(st, hh, plan_id).await?;
    let _ = place_order(st, hh, &plan, &member.id).await;
    st.notify(hh);
    get(st, hh, plan_id).await
}

pub async fn skip(st: &AppState, hh: &str, plan_id: &str, member_id: &str, reason: &str) -> Result<Plan> {
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let plan = get(st, hh, plan_id).await?;
    if plan.status != "held" {
        bail!("Plan is {}; nothing to skip", plan.status);
    }
    if member.role != "approver" {
        bail!("Only the house approver can decide on held plans");
    }
    set_status(st, hh, plan_id, "skipped", Some(&member.id), Some(reason)).await?;
    events::record(
        &st.db,
        hh,
        Event::new("plan_skipped", "plan", plan_id, format!("{} skipped: {}", member.name, if reason.is_empty() { "no reason given" } else { reason }))
            .actor(&member.id)
            .provenance("observed")
            .data(json!({ "agent_version": plan.agent_version })),
    )
    .await?;
    st.notify(hh);
    get(st, hh, plan_id).await
}

/// Charge once per plan version. The idempotency key is the `orders` shard + unique key, the
/// key is checked before charging, and it is sent to the payment provider.
pub async fn place_order(st: &AppState, hh: &str, plan: &Plan, actor: &str) -> Result<state::Order> {
    let _guard = st.order_lock.lock().await;
    let key = format!("{hh}:{}:v{}", plan.id, plan.version);
    if let Some(existing) = state::order_by_key(&st.db, &key).await? {
        events::record(
            &st.db,
            hh,
            Event::new("duplicate_prevented", "plan", &plan.id, format!("Already ordered ({}); not charging again", existing.payment_ref)).actor(actor),
        )
        .await?;
        return Ok(existing);
    }
    let charge = match st.payments.charge(plan.total_cents, &key, &plan.title).await {
        Ok(c) => c,
        Err(err) => {
            set_status(st, hh, &plan.id, "payment_failed", None, Some(&err.to_string())).await?;
            events::record(
                &st.db,
                hh,
                Event::new("payment_failed", "plan", &plan.id, format!("Payment failed: {err}. Nothing was charged; safe to retry."))
                    .actor(actor)
                    .data(json!({ "agent_version": plan.agent_version, "idempotency_key": key })),
            )
            .await?;
            st.notify(hh);
            return Err(err);
        }
    };
    let members = state::members(&st.db, hh).await?;
    let payer = members.iter().find(|m| m.role == "approver").map(|m| m.id.clone()).unwrap_or_else(|| actor.to_string());
    let splits: Vec<Value> = state::equal_splits(&members, plan.total_cents)
        .into_iter()
        .map(|(id, c)| json!({ "member_id": id, "share_cents": c }))
        .collect();
    let order_id = format!("ord_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);

    let mut tx = st.db.begin().await?;
    let inserted = sqlx::query("INSERT IGNORE INTO orders (idempotency_key, household_id, id, plan_id, total_cents, paid_by, payment_provider, payment_ref, splits, test_mode, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?)")
        .bind(&key)
        .bind(hh)
        .bind(&order_id)
        .bind(&plan.id)
        .bind(plan.total_cents)
        .bind(&payer)
        .bind(&charge.provider)
        .bind(&charge.reference)
        .bind(Value::Array(splits.clone()).to_string())
        .bind(now())
        .execute(&mut *tx)
        .await?
        .rows_affected();
    sqlx::query("UPDATE plans SET status = 'ordered' WHERE household_id = ? AND id = ?").bind(hh).bind(&plan.id).execute(&mut *tx).await?;
    tx.commit().await?;

    if inserted == 0 {
        events::record(&st.db, hh, Event::new("duplicate_prevented", "plan", &plan.id, "Order already existed for this plan version").actor(actor)).await?;
    } else {
        events::record(
            &st.db,
            hh,
            Event::new("order_placed", "plan", &plan.id, format!("TEST MODE order {} placed: {} split {} ways", order_id, money(plan.total_cents), members.len()))
                .actor(actor)
                .latency(charge.latency_us)
                .data(json!({ "agent_version": plan.agent_version, "order_id": order_id, "total_cents": plan.total_cents, "provider": charge.provider, "payment_ref": charge.reference, "idempotency_key": key, "splits": splits, "paid_by": payer })),
        )
        .await?;
    }
    st.notify(hh);
    state::order_by_key(&st.db, &key).await?.ok_or_else(|| anyhow!("order not found after insert"))
}

/// Items already in a plan that is waiting, approved or ordered in the last 72 h: item id -> plan title.
pub async fn covered_items(st: &AppState, hh: &str) -> Result<std::collections::HashMap<String, String>> {
    let now = now();
    let mut out = std::collections::HashMap::new();
    for p in list(st, hh).await? {
        let open = matches!(p.status.as_str(), "held" | "approved" | "auto_approved" | "ordered" | "payment_failed");
        if open && p.created_at.map_or(false, |c| (now - c).num_hours() < 72) {
            for l in &p.basket {
                out.entry(l.item_id.clone()).or_insert_with(|| p.title.clone());
            }
        }
    }
    Ok(out)
}

/// Restock scan: anything low that isn't already in an open plan becomes one plan.
pub async fn restock_scan(st: &AppState, hh: &str) -> Result<Option<Plan>> {
    let now = now();
    let items = estimate::item_statuses(&st.db, hh, now).await?;
    let open = list(st, hh).await?;
    let covered: Vec<String> = open
        .iter()
        .filter(|p| matches!(p.status.as_str(), "held" | "approved" | "auto_approved" | "ordered" | "payment_failed"))
        .filter(|p| p.created_at.map_or(false, |c| (now - c).num_hours() < 72))
        .flat_map(|p| p.basket.iter().map(|l| l.item_id.clone()))
        .collect();
    let low: Vec<&ItemStatus> = items
        .iter()
        .filter(|i| i.status == "low" && i.is_shared_consumable && !covered.contains(&i.id))
        .collect();
    if low.is_empty() {
        return Ok(None);
    }
    let deadline = low.iter().filter_map(|i| i.runs_out_on).min().filter(|d| *d > now + chrono::Duration::hours(36));
    let proposals: Vec<Proposal> = low
        .iter()
        .map(|i| Proposal { item_id: i.id.clone(), qty: 1, why: format!("{} · {}", i.basis, i.days_left.map(|d| format!("~{:.0} day(s) left", d)).unwrap_or_default()) })
        .collect();
    let (lines, _) = price_lines(st, &items, &proposals, deadline).await?;
    let names: Vec<String> = low.iter().map(|i| i.name.clone()).collect();
    let plan = propose(
        st,
        hh,
        NewPlan {
            kind: "restock".into(),
            title: format!("Restock: {}", names.join(", ")),
            trigger_ref: None,
            requested_by: None,
            lines,
            claimed_total_cents: None,
            extra_checks: vec![],
            force_refuse: None,
            context: json!({ "trigger": "run-out scan", "items": low.iter().map(|i| json!({ "id": i.id, "basis": i.basis, "days_left": i.days_left, "provenance": i.provenance })).collect::<Vec<_>>() }),
        },
    )
    .await?;
    Ok(Some(plan))
}
