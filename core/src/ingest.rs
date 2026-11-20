//! Receipt ingestion: raw receipt text → verified purchase rows.
//! Extraction is done by a small SIE model (via the sidecar); a line only counts if the exact
//! span is found verbatim in the receipt. If the model is unavailable, a plain parser is used
//! and the row says so.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::agent::planner;
use crate::estimate;
use crate::events::{self, Event};
use crate::plans;
use crate::state::{self, now, AppState};

#[derive(Deserialize)]
pub struct ReceiptIn {
    pub member: String,
    pub text: String,
    #[serde(default)]
    pub store: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
}

struct ExtractedLine {
    span: String,
    product: String,
    qty: i64,
    unit_price_cents: i64,
    values_in_text: bool,
}

/// Fallback parser: "<product>   x<qty>   $<line total>".
fn rule_extract(text: &str) -> Vec<ExtractedLine> {
    text.lines()
        .map(str::trim)
        .filter(|l| l.contains('$'))
        .filter(|l| {
            let low = l.to_lowercase();
            !["total", "tax", "shipping", "paid"].iter().any(|w| low.starts_with(w)) && !low.starts_with("subtotal")
        })
        .filter_map(|l| {
            let dollar = l.rfind('$')?;
            let price: f64 = l[dollar + 1..].trim().split_whitespace().next()?.parse().ok()?;
            let before = l[..dollar].trim();
            let (product, qty) = match before.rsplit_once(" x") {
                Some((p, q)) if q.trim().parse::<i64>().is_ok() => (p.trim(), q.trim().parse::<i64>().unwrap()),
                _ => (before, 1),
            };
            let qty = qty.max(1);
            Some(ExtractedLine { span: l.to_string(), product: product.to_string(), qty, unit_price_cents: ((price * 100.0).round() as i64) / qty, values_in_text: true })
        })
        .collect()
}

fn tokens(t: &str) -> Vec<String> {
    t.to_lowercase().split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| w.len() > 1).map(str::to_string).collect()
}

/// A receipt line counts as an item only if most of its words match a catalog product title.
fn match_line(catalog: &[crate::sourcing::Product], product_text: &str) -> Option<String> {
    let words = tokens(product_text);
    if words.is_empty() {
        return None;
    }
    catalog
        .iter()
        .map(|p| {
            let title = tokens(&p.title);
            (words.iter().filter(|w| title.contains(w)).count() * 100 / words.len(), p)
        })
        .filter(|(score, _)| *score >= 60)
        .max_by_key(|(score, _)| *score)
        .map(|(_, p)| p.item_id.clone())
}

pub async fn ingest_receipt(st: &AppState, hh: &str, r: ReceiptIn) -> Result<Value> {
    let member = state::find_member(&st.db, hh, &r.member).await?.ok_or_else(|| anyhow!("unknown flatmate {}", r.member))?;
    let source = r.source.clone().unwrap_or_else(|| "manual".into());
    let store = r.store.clone().unwrap_or_else(|| "Unknown store".into());
    let subject = r.subject.clone().unwrap_or_else(|| format!("Receipt from {store}"));
    if let Some(sr) = &r.source_ref {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM receipts WHERE household_id = ? AND source_ref = ?").bind(hh).bind(sr).fetch_one(&st.db).await.unwrap_or(0);
        if n > 0 {
            return Ok(json!({ "skipped": true, "reason": "already ingested", "source_ref": sr }));
        }
    }
    let receipt_id = format!("rcp_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
    sqlx::query("INSERT INTO receipts (household_id, id, source, source_ref, member_id, store, received_at, subject, raw_text) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(hh).bind(&receipt_id).bind(&source).bind(&r.source_ref).bind(&member.id).bind(&store).bind(now()).bind(&subject).bind(&r.text)
        .execute(&st.db)
        .await?;
    events::record(
        &st.db,
        hh,
        Event::new("receipt_ingested", "receipt", &receipt_id, format!("Receipt from {store} read from {}’s inbox", member.name))
            .actor(&member.id)
            .provenance("observed")
            .data(json!({ "source": source, "source_ref": r.source_ref })),
    )
    .await?;

    // Small-model extraction, with a rule fallback.
    let (lines, extractor, latency_us) = match st.sidecar.post("/extract", json!({ "text": r.text })).await {
        Ok(v) => {
            let model = v.get("model").and_then(Value::as_str).unwrap_or("sie").to_string();
            let lines = v
                .get("lines")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|l| {
                            Some(ExtractedLine {
                                span: l.get("span")?.as_str()?.to_string(),
                                product: l.get("product").and_then(Value::as_str).unwrap_or_default().to_string(),
                                qty: l.get("qty").and_then(Value::as_i64).unwrap_or(1).max(1),
                                unit_price_cents: l.get("unit_price_cents").and_then(Value::as_i64).unwrap_or(0),
                                values_in_text: l.get("values_in_text").and_then(Value::as_bool).unwrap_or(false),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            (lines, model, v.get("latency_us").and_then(Value::as_i64).unwrap_or(0))
        }
        Err(err) => {
            events::record(&st.db, hh, Event::new("model_failed", "receipt", &receipt_id, format!("Extraction model unavailable ({err}); used the plain parser"))).await?;
            (rule_extract(&r.text), "rule-parser".to_string(), 0)
        }
    };

    let items = estimate::item_statuses(&st.db, hh, now()).await?;
    let catalog = crate::sourcing::products(&st.db).await?;
    let mut out = vec![];
    for l in &lines {
        let verbatim = r.text.contains(&l.span);
        let matched = match_line(&catalog, if l.product.is_empty() { &l.span } else { &l.product }).and_then(|id| items.iter().find(|i| i.id == id));
        let verified = verbatim && l.values_in_text && matched.is_some() && l.unit_price_cents > 0;
        let item_id = matched.map(|i| i.id.clone()).unwrap_or_else(|| "unmatched".into());
        sqlx::query("INSERT INTO purchases (household_id, id, receipt_id, item_id, qty, unit_price_cents, purchased_at, evidence_span, verified, extractor) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(hh).bind(uuid::Uuid::new_v4().to_string()).bind(&receipt_id).bind(&item_id).bind(l.qty).bind(l.unit_price_cents).bind(now()).bind(&l.span).bind(verified).bind(&extractor)
            .execute(&st.db)
            .await?;
        let summary = match (matched, verified) {
            (Some(i), true) => format!("Extracted {} ×{} — line found verbatim in the receipt", i.name, l.qty),
            (Some(i), false) => format!("Extracted {} ×{} but could not verify it against the receipt; excluded from estimates", i.name, l.qty),
            (None, _) => format!("Line “{}” is not a tracked household item; ignored", l.span),
        };
        let mut ev = Event::new("line_extracted", "item", &item_id, &summary).provenance("inferred").data(json!({ "receipt_id": receipt_id, "span": l.span, "verified": verified, "extractor": extractor }));
        if latency_us > 0 {
            ev = ev.model(&extractor, latency_us / lines.len().max(1) as i64, 0.0);
        }
        events::record(&st.db, hh, ev).await?;
        out.push(json!({ "item_id": item_id, "qty": l.qty, "unit_price_cents": l.unit_price_cents, "verified": verified, "span": l.span }));
    }
    st.notify(hh);
    let _ = plans::restock_scan(st, hh).await;
    Ok(json!({ "receipt_id": receipt_id, "extractor": extractor, "lines": out }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_product_sharing_a_brand_word_is_not_matched() {
        let p = |title: &str, item: &str| crate::sourcing::Product { id: item.into(), item_id: item.into(), store: "s".into(), channel: "online".into(), title: title.into(), price_cents: 1, delivery_days: 0, cutoff_hour: 0 };
        let catalog = vec![p("Boomlet Bluetooth Party Speaker", "speaker"), p("ScrubPro Sponges 6 ct", "sponges")];
        assert_eq!(match_line(&catalog, "Boomlet Phone Case"), None);
        assert_eq!(match_line(&catalog, "ScrubPro Sponges 6 ct"), Some("sponges".into()));
    }

    #[test]
    fn rule_parser_reads_qty_and_unit_price() {
        let lines = rule_extract("Thanks!\n\n  BrightSuds Dish Soap 28 oz   x2   $7.98\n\nPaid with card ending 4242.\nTotal $7.98");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].product, "BrightSuds Dish Soap 28 oz");
        assert_eq!(lines[0].qty, 2);
        assert_eq!(lines[0].unit_price_cents, 399);
    }
}

/// Run the Airbyte sync (in the sidecar) and ingest what it returns: receipts become purchases,
/// invites become calendar events with a plan. Already-seen messages are skipped.
pub async fn airbyte_sync(st: &AppState, hh: &str) -> Result<Value> {
    let out = st.sidecar.post("/airbyte/sync", json!({})).await?;
    let records = out.get("records").and_then(Value::as_array).cloned().unwrap_or_default();
    events::record(
        &st.db,
        hh,
        Event::new("airbyte_sync", "system", "airbyte", format!("Airbyte synced {} inbox record(s) via {}", records.len(), out.get("connector").and_then(Value::as_str).unwrap_or("source")))
            .latency(out.get("latency_us").and_then(Value::as_i64).unwrap_or(0))
            .provenance("observed"),
    )
    .await?;
    let s = |r: &Value, k: &str| r.get(k).and_then(Value::as_str).map(str::to_string);
    let mut results = vec![];
    for r in &records {
        let msg = s(r, "message_id");
        match s(r, "type").as_deref() {
            Some("receipt") => {
                let res = ingest_receipt(st, hh, ReceiptIn { member: s(r, "to").unwrap_or_default(), text: s(r, "body").unwrap_or_default(), store: s(r, "store"), subject: s(r, "subject"), source: Some("airbyte".into()), source_ref: msg.clone() }).await;
                results.push(res.unwrap_or_else(|e| json!({ "error": e.to_string(), "source_ref": msg })));
            }
            Some("outing") => {
                let res = crate::outings::create(
                    st,
                    hh,
                    crate::outings::NewOuting {
                        title: s(r, "title").unwrap_or_else(|| "Outing".into()),
                        kind: s(r, "kind"),
                        venue: s(r, "venue"),
                        days_from_now: r.get("days_from_now").and_then(Value::as_i64),
                        organizer: s(r, "to"),
                        pricing: r.get("pricing").and_then(|p| p.as_str()).and_then(|p| serde_json::from_str(p).ok()).or_else(|| r.get("pricing").cloned().and_then(|p| serde_json::from_value(p).ok())),
                        source: Some("airbyte".into()),
                        source_ref: msg.clone(),
                    },
                )
                .await;
                results.push(match res {
                    Ok(o) => json!({ "outing_id": o.id, "going": o.quote.headcount, "per_person_cents": o.quote.per_person_cents, "ride": o.quote.ride.mode }),
                    Err(e) => json!({ "skipped": true, "reason": e.to_string(), "source_ref": msg }),
                });
            }
            Some("invite") | Some("calendar") => {
                let seen: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM calendar_events WHERE household_id = ? AND source_ref = ?").bind(hh).bind(&msg).fetch_one(&st.db).await.unwrap_or(0);
                if seen > 0 {
                    results.push(json!({ "skipped": true, "source_ref": msg }));
                    continue;
                }
                let days = r.get("days_from_now").and_then(Value::as_i64).unwrap_or(4);
                let starts = (now() + chrono::Duration::days(days)).date().and_hms_opt(19, 0, 0).unwrap();
                let kind = s(r, "kind").unwrap_or_else(|| "party".into());
                let id = planner::add_calendar_event(st, hh, &s(r, "title").unwrap_or_else(|| "Event".into()), starts, r.get("guests").and_then(Value::as_i64).unwrap_or(0), &kind, r.get("nights").and_then(Value::as_i64).unwrap_or(0), &s(r, "to").unwrap_or_default(), "airbyte", msg.as_deref()).await?;
                if kind == "away" {
                    events::record(&st.db, hh, Event::new("nothing_to_buy", "event", &id, "Someone is away: nothing to buy")).await?;
                    results.push(json!({ "event_id": id, "status": "nothing_to_buy" }));
                } else {
                    let plan = planner::plan_for_event(st, hh, &id, true).await?;
                    results.push(json!({ "event_id": id, "plan_id": plan.id, "status": plan.status, "items": plan.basket.len() }));
                }
            }
            _ => results.push(json!({ "ignored": true, "source_ref": msg })),
        }
    }
    Ok(json!({ "synced": records.len(), "results": results }))
}
