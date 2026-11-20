//! Forwards `agent_events` to Amplitude's HTTP API.
//! Model and tool steps become `[Agent]` events (Agent Analytics); decision steps become plain
//! product events so the plan → approval → order funnel can be split by agent version.

use std::time::Duration;

use serde_json::{json, Value};

use crate::db::Db;
use crate::events::{self, StoredEvent};

const AGENT_ID: &str = "stocked-household-agent";

fn product_event_name(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "plan_proposed" => "Plan Proposed",
        "plan_held" => "Plan Held For Approval",
        "plan_auto_approved" => "Plan Auto Approved",
        "plan_approved" => "Plan Approved",
        "plan_skipped" => "Plan Skipped",
        "plan_refused" => "Plan Refused",
        "order_placed" => "Order Placed",
        "duplicate_prevented" => "Duplicate Charge Prevented",
        "payment_failed" => "Payment Failed",
        "payment_retried" => "Payment Retried",
        "stock_corrected" => "Stock Corrected",
        "receipt_ingested" => "Receipt Ingested",
        "invite_received" => "Invite Received",
        "outing_booked" => "Outing Booked",
        "venue_closed" => "Venue Closed",
        "replanned_to_backup" => "Replanned To Backup",
        "backup_needs_approval" => "Backup Needs Approval",
        "needs_organiser" => "Needs Organiser",
        "booking_changed" => "Booking Re-planned",
        "identity_verified" => "Identity Verified",
        "approval_blocked_unverified" => "Approval Blocked Unverified",
        _ => return None,
    })
}

/// Chat turns map onto Amplitude's Agent Analytics taxonomy: User Message, Tool Call, AI Response, Score.
fn session_event(hh: &str, e: &StoredEvent, time_ms: i64) -> Option<Value> {
    let ev = &e.event;
    let d = &ev.data;
    let session = d.get("session").and_then(Value::as_str)?;
    let version = d.get("agent_version").and_then(Value::as_str).unwrap_or("");
    let mut props = json!({
        "[Agent] Session ID": session,
        "[Agent] Turn ID": d.get("turn").and_then(Value::as_i64).unwrap_or(1),
        "[Agent] Trace ID": d.get("trace").and_then(Value::as_str).unwrap_or(""),
        "[Agent] Agent ID": AGENT_ID,
        "[Agent] Env": "hackathon",
        "[Agent] Runtime": "rust",
        "[Agent] Experiment Key": crate::integrations::statsig::GATE_MODEL_PLANNER,
        "[Agent] Experiment Variant": version,
        "[Agent] Experiment Provider": "statsig",
        "agent_version": version,
        "household_id": hh,
    });
    let event_type = match ev.kind.as_str() {
        "chat_message" => {
            props["[Agent] Message ID"] = json!(e.id);
            props["$llm_message"] = json!({ "text": d.get("text").and_then(Value::as_str).unwrap_or(&ev.summary) });
            "[Agent] User Message"
        }
        "tool_call" => {
            props["[Agent] Invocation ID"] = json!(e.id);
            props["[Agent] Tool Name"] = d.get("tool").cloned().unwrap_or(json!("unknown"));
            props["[Agent] Tool Success"] = d.get("success").cloned().unwrap_or(json!(true));
            props["[Agent] Latency Ms"] = json!(ev.latency_us as f64 / 1000.0);
            props["[Agent] Parent Message ID"] = d.get("parent_message_id").cloned().unwrap_or(json!(""));
            props["[Agent] Tool Input"] = json!(d.get("input").map(|v| v.to_string()).unwrap_or_default());
            props["[Agent] Tool Output"] = json!(d.get("output").map(|v| v.to_string().chars().take(900).collect::<String>()).unwrap_or_default());
            "[Agent] Tool Call"
        }
        "agent_reply" => {
            let (tin, tout) = (d.get("input_tokens").and_then(Value::as_i64).unwrap_or(0), d.get("output_tokens").and_then(Value::as_i64).unwrap_or(0));
            props["[Agent] Message ID"] = json!(e.id);
            props["[Agent] Model Name"] = json!(ev.model.clone().unwrap_or_default());
            props["[Agent] Provider"] = json!("superlinked-sie");
            props["[Agent] Latency Ms"] = json!(d.get("turn_latency_us").and_then(Value::as_i64).unwrap_or(ev.latency_us) as f64 / 1000.0);
            props["[Agent] Input Tokens"] = json!(tin);
            props["[Agent] Output Tokens"] = json!(tout);
            props["[Agent] Total Tokens"] = json!(tin + tout);
            props["$llm_message"] = json!({ "text": d.get("text").and_then(Value::as_str).unwrap_or(&ev.summary) });
            "[Agent] AI Response"
        }
        "session_end" => {
            props["[Agent] Session Trigger"] = json!("headless");
            "[Agent] Session End"
        }
        "turn_score" => {
            props["[Agent] Score Name"] = d.get("score").cloned().unwrap_or(json!("score"));
            props["[Agent] Score Value"] = d.get("value").cloned().unwrap_or(json!(0));
            props["[Agent] Target ID"] = d.get("target_message_id").cloned().unwrap_or(json!(""));
            props["[Agent] Target Type"] = json!("message");
            props["[Agent] Evaluation Source"] = json!("reviewer");
            props["[Agent] Comment"] = json!(format!("Automated code check: {}", d.get("comment").and_then(Value::as_str).unwrap_or("")));
            "[Agent] Score"
        }
        _ => return None,
    };
    Some(json!({ "event_type": event_type, "user_id": hh, "time": time_ms, "insert_id": format!("{}-s", e.id), "event_properties": props }))
}

fn to_amplitude(hh: &str, e: &StoredEvent) -> Vec<Value> {
    let ev = &e.event;
    let time_ms = e.ts.map(|t| t.and_utc().timestamp_millis()).unwrap_or(0);
    if let Some(v) = session_event(hh, e, time_ms) {
        return vec![v];
    }
    let user_id = hh.to_string();
    let version = ev.data.get("agent_version").and_then(Value::as_str).unwrap_or("").to_string();
    let mut out = vec![];
    if let Some(name) = product_event_name(&ev.kind) {
        let mut props = json!({ "household_id": hh, "subject_type": ev.subject_type, "subject_id": ev.subject_id, "actor": ev.actor, "summary": ev.summary, "provenance": ev.provenance, "agent_version": version });
        if let Some(t) = ev.data.get("total_cents") {
            props["total_usd"] = json!(t.as_f64().unwrap_or(0.0) / 100.0);
        }
        out.push(json!({ "event_type": name, "user_id": user_id, "time": time_ms, "insert_id": format!("{}-p", e.id), "event_properties": props }));
    }
    out
}

/// Send everything not yet forwarded. Returns how many events were handled.
pub async fn flush_once(db: &Db, api_key: Option<&String>, flags: &crate::integrations::statsig::Flags, http: &reqwest::Client) -> usize {
    let batch = match events::unforwarded(db, 200).await {
        Ok(b) if !b.is_empty() => b,
        _ => return 0,
    };
    let ids: Vec<String> = batch.iter().map(|(_, e)| e.id.clone()).collect();
    // Statsig: plan and order outcomes, tagged with the variant they ran under.
    for (hh, e) in &batch {
        if let Some(name) = product_event_name(&e.event.kind) {
            let mut meta = std::collections::HashMap::from([("subject_id".to_string(), e.event.subject_id.clone()), ("actor".to_string(), e.event.actor.clone())]);
            if let Some(v) = e.event.data.get("agent_version").and_then(Value::as_str) {
                meta.insert("agent_version".into(), v.to_string());
            }
            let value = e.event.data.get("total_cents").and_then(Value::as_i64).map(|c| format!("{:.2}", c as f64 / 100.0));
            flags.log(hh, name, value, meta);
        }
    }
    flags.flush().await;
    let amp: Vec<Value> = batch.iter().flat_map(|(hh, e)| to_amplitude(hh, e)).collect();
    if let (false, Some(key)) = (amp.is_empty(), api_key) {
        match http.post("https://api2.amplitude.com/2/httpapi").json(&json!({ "api_key": key, "events": amp })).send().await {
            Ok(r) if r.status().is_success() => {}
            Ok(r) => {
                tracing::warn!("amplitude HTTP {}: {}", r.status(), r.text().await.unwrap_or_default());
                return 0;
            }
            Err(err) => {
                tracing::warn!("amplitude send failed: {err}");
                return 0;
            }
        }
    }
    if let Err(err) = events::mark_forwarded(db, &ids).await {
        tracing::warn!("mark_forwarded failed: {err}");
    }
    ids.len()
}

pub fn spawn_forwarder(db: Db, api_key: Option<String>, flags: crate::integrations::statsig::Flags) {
    if api_key.is_none() && !flags.connected() {
        tracing::info!("no analytics keys set — event forwarding disabled");
        return;
    }
    tokio::spawn(async move {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
        loop {
            tokio::time::sleep(Duration::from_secs(4)).await;
            flush_once(&db, api_key.as_ref(), &flags, &http).await;
        }
    });
}
