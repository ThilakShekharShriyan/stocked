//! The proposing side of the agent. The model (or a rule fallback) proposes; `plans::propose`
//! verifies. Nothing here can place an order on its own.

use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use serde_json::{json, Value};

use crate::checks::{Check, Status};
use crate::db;
use crate::estimate::{self, ItemStatus};
use crate::events::{self, Event};
use crate::integrations::sie;
use crate::plans::{self, NewPlan, Plan, Proposal};
use crate::state::{self, now, AppState};

#[derive(Clone, Debug, serde::Serialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub starts_at: Option<NaiveDateTime>,
    pub guests: i64,
    pub kind: String,
    pub nights: i64,
    pub host_member_id: String,
    pub source: String,
}

pub async fn calendar_events(st: &AppState, hh: &str) -> Result<Vec<CalendarEvent>> {
    let rows = sqlx::query("SELECT id, title, starts_at, guests, kind, nights, host_member_id, source FROM calendar_events WHERE household_id = ? ORDER BY starts_at")
        .bind(hh)
        .fetch_all(&st.db)
        .await?;
    Ok(rows
        .iter()
        .map(|r| CalendarEvent {
            id: db::s(r, "id"),
            title: db::s(r, "title"),
            starts_at: db::dt(r, "starts_at"),
            guests: db::i(r, "guests"),
            kind: db::s(r, "kind"),
            nights: db::i(r, "nights"),
            host_member_id: db::s(r, "host_member_id"),
            source: db::s(r, "source"),
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
pub async fn add_calendar_event(st: &AppState, hh: &str, title: &str, starts_at: NaiveDateTime, guests: i64, kind: &str, nights: i64, host: &str, source: &str, source_ref: Option<&str>) -> Result<String> {
    let id = format!("evt_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
    sqlx::query("INSERT INTO calendar_events (household_id, id, title, starts_at, guests, kind, nights, host_member_id, source, source_ref) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(hh).bind(&id).bind(title).bind(starts_at).bind(guests).bind(kind).bind(nights).bind(host).bind(source).bind(source_ref)
        .execute(&st.db)
        .await?;
    events::record(
        &st.db,
        hh,
        Event::new("invite_received", "event", &id, format!("On {host}’s calendar: {title} · {} · {}", crate::sourcing::fmt_when(starts_at), describe(kind, guests, nights)))
            .actor(host)
            .provenance("observed")
            .data(json!({ "source": source, "source_ref": source_ref, "guests": guests, "kind": kind, "nights": nights, "calendar": host })),
    )
    .await?;
    st.notify(hh);
    Ok(id)
}

pub fn describe(kind: &str, guests: i64, nights: i64) -> String {
    match kind {
        "houseguests" => format!("{guests} guest(s) staying {nights} night(s)"),
        "away" => format!("away for {nights} night(s)"),
        "dinner" => format!("dinner for {guests}"),
        _ => format!("{guests} guests"),
    }
}

fn ceil_div(a: i64, b: i64) -> i64 {
    (a + b - 1) / b
}

/// Rule fallback when the model is unavailable: per-guest ratios.
fn rule_proposals(kind: &str, guests: i64, nights: i64) -> Vec<Proposal> {
    let g = guests.max(1);
    let p = |id: &str, qty: i64, why: String| Proposal { item_id: id.into(), qty, why };
    match kind {
        "houseguests" => {
            return vec![
                p("toilet_paper", 1, format!("{g} extra people for {nights} night(s)")),
                p("coffee", 1, format!("{g} extra coffee drinkers for {nights} morning(s)")),
                p("hand_soap", 1, "extra use while guests stay".into()),
                p("paper_towels", 1, "extra use while guests stay".into()),
            ]
        }
        "dinner" => {
            return vec![
                p("napkins", 1, format!("dinner for {g}")),
                p("soda", ceil_div(g, 3), format!("1 bottle per 3 people × {g}")),
                p("olive_oil", 1, "cooking for a group".into()),
                p("dish_soap", 1, "washing up after dinner".into()),
            ]
        }
        _ => {}
    }
    vec![
        p("chips", ceil_div(g, 4), format!("1 bag per 4 guests × {g}")),
        p("soda", ceil_div(g, 4), format!("1 bottle per 4 guests × {g}")),
        p("paper_plates", ceil_div(g, 50), "one pack covers 50".into()),
        p("cups", ceil_div(g, 50), "one pack covers 50".into()),
        p("napkins", 1, "one pack".into()),
        p("ice", ceil_div(g, 6), format!("1 bag per 6 guests × {g}")),
        p("toilet_paper", 1, "extra guests".into()),
        p("trash_bags", 1, "clean-up".into()),
    ]
}

async fn model_proposals(st: &AppState, ev: &CalendarEvent, items: &[ItemStatus]) -> Result<(Vec<Proposal>, Option<i64>, sie::Call)> {
    let catalog: Vec<Value> = items
        .iter()
        .filter(|i| i.is_shared_consumable)
        .map(|i| json!({ "item_id": i.id, "name": i.name, "unit": i.unit, "category": i.category }))
        .collect();
    let messages = vec![
        json!({ "role": "system", "content": "You plan shared-household shopping for flatmates. Reply with one JSON object and nothing else." }),
        json!({ "role": "user", "content": format!(
            "Event on a flatmate's calendar: {} ({}).\nItems the flat can buy (use only these item_id values):\n{}\n\nPropose what this event needs in total, ignoring what is already at home (stock is subtracted later). Keep it modest and practical.\nReturn: {{\"items\":[{{\"item_id\":\"...\",\"qty\":1,\"why\":\"short reason\"}}],\"total_usd\":0}}",
            ev.title, describe(&ev.kind, ev.guests, ev.nights), serde_json::to_string(&catalog)?
        ) }),
    ];
    let call = st.sie.chat(&messages, Value::Null, true, Duration::from_secs(25)).await?;
    let text = sie::message(&call).get("content").and_then(Value::as_str).unwrap_or_default().to_string();
    let v = sie::parse_json_text(&text).ok_or_else(|| anyhow!("planner did not return JSON"))?;
    let proposals: Vec<Proposal> = v
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("planner JSON has no items"))?
        .iter()
        .filter_map(|x| {
            Some(Proposal {
                item_id: x.get("item_id")?.as_str()?.to_string(),
                qty: x.get("qty").and_then(|q| q.as_i64().or_else(|| q.as_f64().map(|f| f.ceil() as i64))).unwrap_or(1),
                why: x.get("why").and_then(Value::as_str).unwrap_or_default().to_string(),
            })
        })
        .collect();
    anyhow::ensure!(!proposals.is_empty(), "planner proposed nothing");
    let claimed = v.get("total_usd").and_then(Value::as_f64).filter(|t| *t > 0.0).map(|t| (t * 100.0).round() as i64);
    Ok((proposals, claimed, call))
}

/// Event → plan. Needs are proposed, then stock the flat already has is subtracted in code.
pub async fn plan_for_event(st: &AppState, hh: &str, event_id: &str, use_model: bool) -> Result<Plan> {
    let ev = calendar_events(st, hh).await?.into_iter().find(|e| e.id == event_id).ok_or_else(|| anyhow!("no event {event_id}"))?;
    let items = estimate::item_statuses(&st.db, hh, now()).await?;

    let (mut proposals, mut claimed, mut planner) = (rule_proposals(&ev.kind, ev.guests, ev.nights), None, "rule: per-guest ratios".to_string());
    let variant = st.sidecar.config(hh).await.agent_version;
    if use_model && variant != "v1-rules" && st.sie.planner_key.is_some() {
        match model_proposals(st, &ev, &items).await {
            Ok((p, c, call)) => {
                events::record(
                    &st.db,
                    hh,
                    Event::new("model_proposed_basket", "event", &ev.id, format!("Model proposed {} item(s) for {}", p.len(), ev.title))
                        .model(&call.model, call.latency_us, call.credits)
                        .provenance("inferred")
                        .data(json!({ "proposals": p.iter().map(|x| json!({ "item_id": x.item_id, "qty": x.qty, "why": x.why })).collect::<Vec<_>>(), "claimed_total_cents": c })),
                )
                .await?;
                planner = call.model.clone();
                proposals = p;
                claimed = c;
            }
            Err(err) => {
                events::record(&st.db, hh, Event::new("model_failed", "event", &ev.id, format!("Planner model unavailable ({err}); used the per-guest rule instead"))).await?;
            }
        }
    }

    // Stock subtraction: staples the flat still has plenty of are removed, and so is anything
    // another plan (for another flatmate's event, or a restock) already covers.
    let covered = plans::covered_items(st, hh).await?;
    let mut kept = vec![];
    let mut in_stock = vec![];
    let mut already_covered = vec![];
    for p in proposals {
        if let Some(title) = covered.get(&p.item_id) {
            already_covered.push(json!({ "item_id": p.item_id, "covered_by": title }));
            continue;
        }
        match items.iter().find(|i| i.id == p.item_id) {
            Some(i) if i.purchases > 0 && i.status == "ok" => {
                in_stock.push(json!({ "item": i.name, "days_left": i.days_left.map(|d| d.round()), "basis": i.basis }))
            }
            _ => kept.push(p),
        }
    }
    let (lines, dropped) = plans::price_lines(st, &items, &kept, ev.starts_at).await?;
    let mut extra = vec![];
    if !already_covered.is_empty() {
        extra.push(Check { name: "no_double_buying".into(), status: Status::Pass, detail: format!("Skipped {} item(s) another plan already covers", already_covered.len()) });
    }
    if lines.is_empty() {
        // Nothing left to buy: record why instead of creating an empty plan.
        events::record(&st.db, hh, Event::new("nothing_to_buy", "event", &ev.id, format!("{}: nothing to buy — the flat has it or another plan covers it", ev.title)).data(json!({ "already_in_stock": in_stock, "already_covered": already_covered }))).await?;
    }
    if !in_stock.is_empty() || !dropped.is_empty() {
        extra.push(Check {
            name: "stock_subtracted".into(),
            status: Status::Pass,
            detail: format!(
                "Skipped {} item(s) already at home{}",
                in_stock.len(),
                if dropped.is_empty() { String::new() } else { format!("; ignored {} unknown item(s)", dropped.len()) }
            ),
        });
    }
    plans::propose(
        st,
        hh,
        NewPlan {
            kind: "event".into(),
            title: format!("{} · {}", ev.title, describe(&ev.kind, ev.guests, ev.nights)),
            trigger_ref: Some(ev.id.clone()),
            requested_by: Some(ev.host_member_id.clone()),
            lines,
            claimed_total_cents: claimed,
            extra_checks: extra,
            force_refuse: None,
            context: json!({ "event": ev, "planner": planner, "already_in_stock": in_stock, "already_covered": already_covered, "ignored": dropped }),
        },
    )
    .await
}

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.trim_end_matches('s').to_string())
        .collect()
}

/// Match free text to a household item: SingleStore full-text search over the catalog first,
/// then a plain name match.
pub async fn match_item<'a>(st: &AppState, items: &'a [ItemStatus], text: &str) -> Option<&'a ItemStatus> {
    let ws = words(text);
    if ws.is_empty() {
        return None;
    }
    let by_name = items
        .iter()
        .map(|i| {
            let name_words = words(&i.name);
            let hits = name_words.iter().filter(|w| ws.contains(w)).count();
            (hits * 100 / name_words.len().max(1), i)
        })
        .filter(|(score, _)| *score >= 50)
        .max_by_key(|(score, _)| *score)
        .map(|(_, i)| i);
    if by_name.is_some() {
        return by_name;
    }
    let terms = ws.join(" OR ");
    let query = format!("title:({terms}) OR description:({terms})");
    let sql = format!(
        "SELECT item_id, MATCH(TABLE products) AGAINST ('{query}') AS score FROM products WHERE MATCH(TABLE products) AGAINST ('{query}') ORDER BY score DESC LIMIT 1"
    );
    let row = sqlx::query(&sql).fetch_optional(&st.db).await.ok().flatten()?;
    let item_id = db::s(&row, "item_id");
    items.iter().find(|i| i.id == item_id)
}

/// A flatmate asks for something. Unknown items and non-consumables are refused with reasons.
pub async fn handle_request(st: &AppState, hh: &str, member_id: &str, text: &str, qty: i64) -> Result<Plan> {
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let items = estimate::item_statuses(&st.db, hh, now()).await?;
    let matched = match_item(st, &items, text).await;

    // SIE decision gate: is this a shared household consumable? Catalog flag is the fallback.
    let mut extra = vec![];
    let mut force_refuse = None;
    let started = Instant::now();
    const SHARED: &[&str] = &["cleaning or bathroom supply", "food or drink", "kitchen or party supply"];
    const NOT_SHARED: &[&str] = &["electronics", "personal accessory", "furniture or appliance"];
    let choices: Vec<&str> = SHARED.iter().chain(NOT_SHARED.iter()).copied().collect();
    match st.sie.classify(text, "category", "What kind of product is this?", &choices, Duration::from_millis(2500)).await {
        Ok((category, p, call)) => {
            let yes = SHARED.contains(&category.as_str());
            events::record(
                &st.db,
                hh,
                Event::new("decision_gate", "request", &member.id, format!("“{text}” classified as {category} ({:.0}%) — {}", p * 100.0, if yes { "a shared supply" } else { "not a shared supply" }))
                    .model(&call.model, call.latency_us, call.credits)
                    .provenance("inferred")
                    .data(json!({ "category": category, "probability": p, "text": text })),
            )
            .await?;
            extra.push(Check {
                name: "decision_model".into(),
                status: if yes { Status::Pass } else { Status::Fail },
                detail: format!("{} classifies this as “{category}” ({:.0}%)", call.model, p * 100.0),
            });
            if !yes && matched.map_or(true, |m| !m.is_shared_consumable) {
                force_refuse = Some("Not a shared household supply".to_string());
            }
        }
        Err(err) => extra.push(Check {
            name: "decision_model".into(),
            status: Status::CannotMeasure,
            detail: format!("Decision model unavailable after {} ms ({err}); used the catalog flag", started.elapsed().as_millis()),
        }),
    }

    // Asking again after a payment failure retries that order (from v1 on) instead of calling it a duplicate.
    if let Some(item) = matched {
        if st.sidecar.config(hh).await.agent_version != "v0" {
            let failed = plans::list(st, hh).await?.into_iter().find(|p| p.status == "payment_failed" && p.basket.iter().any(|l| l.item_id == item.id) && p.verdict.get("outcome").and_then(Value::as_str) == Some("auto_approve"));
            if let Some(p) = failed {
                return plans::retry_payment(st, hh, &p.id, &member.id).await;
            }
        }
    }
    // Two flatmates (or their agents) asking for the same thing must not buy it twice.
    if let Some(item) = matched {
        if let Some(by) = plans::covered_items(st, hh).await?.get(&item.id) {
            extra.push(Check { name: "no_double_buying".into(), status: Status::Fail, detail: format!("{} is already covered by “{by}”", item.name) });
            force_refuse.get_or_insert_with(|| format!("Already covered by “{by}”; not buying it twice"));
        }
    }
    let (lines, title) = match matched {
        Some(item) => {
            let (lines, _) = plans::price_lines(st, &items, &[Proposal { item_id: item.id.clone(), qty: qty.max(1), why: format!("Requested by {}", member.name) }], None).await?;
            (lines, format!("{} asked for {}", member.name, item.name))
        }
        None => {
            force_refuse.get_or_insert_with(|| "Not something the flat stocks; no catalog price to verify".to_string());
            (
                vec![crate::checks::Line {
                    item_id: "unknown".into(),
                    name: text.chars().take(60).collect(),
                    qty: qty.max(1),
                    unit_price_cents: 0,
                    is_shared_consumable: false,
                    product_id: None,
                    store: None,
                    channel: None,
                    sourcing_note: "No store in the catalog carries this".into(),
                    arrives_in_time: None,
                    why: format!("Requested by {}", member.name),
                }],
                format!("{} asked for “{}”", member.name, text.chars().take(40).collect::<String>()),
            )
        }
    };
    plans::propose(
        st,
        hh,
        NewPlan {
            kind: "request".into(),
            title,
            trigger_ref: None,
            requested_by: Some(member.id.clone()),
            lines,
            claimed_total_cents: None,
            extra_checks: extra,
            force_refuse,
            context: json!({ "request_text": text, "requested_by": member.id }),
        },
    )
    .await
}

/// One-tap correction ("we're out" / "plenty left"). Recorded as observed, not inferred.
pub async fn correct_stock(st: &AppState, hh: &str, member_id: &str, item_text: &str, level: &str) -> Result<String> {
    anyhow::ensure!(level == "out" || level == "plenty", "level must be 'out' or 'plenty'");
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let items = estimate::item_statuses(&st.db, hh, now()).await?;
    let item = match items.iter().find(|i| i.id == item_text) {
        Some(i) => i,
        None => match_item(st, &items, item_text).await.ok_or_else(|| anyhow!("no household item matches “{item_text}”"))?,
    };
    sqlx::query("UPDATE items SET correction = ?, correction_at = ?, corrected_by = ? WHERE household_id = ? AND id = ?")
        .bind(level).bind(now()).bind(&member.id).bind(hh).bind(&item.id)
        .execute(&st.db)
        .await?;
    let summary = if level == "out" { format!("{} says we're out of {}", member.name, item.name) } else { format!("{} says there's plenty of {}", member.name, item.name) };
    events::record(&st.db, hh, Event::new("stock_corrected", "item", &item.id, &summary).actor(&member.id).provenance("observed").data(json!({ "level": level }))).await?;
    st.notify(hh);
    if level == "out" {
        let _ = plans::restock_scan(st, hh).await;
    }
    Ok(summary)
}
