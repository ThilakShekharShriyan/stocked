//! Shared application state and household-level queries (members, orders, balances).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::{Datelike, NaiveDateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};

use crate::config::Config;
use crate::db::{self, Db};
use crate::integrations::{persona::Persona, senso::Senso, sidecar::Sidecar, sie::Sie};
use crate::payments::Payments;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub db: Db,
    pub sie: Sie,
    pub sidecar: Sidecar,
    pub senso: Senso,
    pub persona: Persona,
    pub payments: Payments,
    pub world: crate::world::World,
    /// Household ids whose state changed (drives the dashboard's live updates).
    pub changes: broadcast::Sender<String>,
    /// Firedrill/chat sessions: conversation history per session id.
    pub sessions: Arc<Mutex<HashMap<String, Vec<Value>>>>,
    /// Serialises order placement per household (belt and braces on top of the unique key).
    pub order_lock: Arc<Mutex<()>>,
    /// Serialises creation of per-session (drill) households.
    pub seed_lock: Arc<Mutex<()>>,
}

impl AppState {
    pub fn notify(&self, hh: &str) {
        let _ = self.changes.send(hh.to_string());
    }
}

pub fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

#[derive(Clone, Debug, Serialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: String,
}

pub async fn members(db: &Db, hh: &str) -> Result<Vec<Member>> {
    let rows = sqlx::query("SELECT id, name, role FROM members WHERE household_id = ? ORDER BY id").bind(hh).fetch_all(db).await?;
    let mut m: Vec<Member> = rows.iter().map(|r| Member { id: db::s(r, "id"), name: db::s(r, "name"), role: db::s(r, "role") }).collect();
    // Approver first, then seed order.
    let order = ["maya", "sam", "priya", "leo"];
    m.sort_by_key(|x| order.iter().position(|o| *o == x.id).unwrap_or(99));
    Ok(m)
}

pub async fn find_member(db: &Db, hh: &str, name_or_id: &str) -> Result<Option<Member>> {
    let needle = name_or_id.trim().to_lowercase();
    Ok(members(db, hh).await?.into_iter().find(|m| m.id == needle || m.name.to_lowercase() == needle))
}

#[derive(Clone, Debug, Serialize)]
pub struct Order {
    pub id: String,
    pub plan_id: String,
    pub idempotency_key: String,
    pub total_cents: i64,
    pub paid_by: String,
    pub payment_provider: String,
    pub payment_ref: String,
    pub splits: Value,
    pub test_mode: bool,
    pub created_at: Option<NaiveDateTime>,
}

fn order_from_row(r: &sqlx::mysql::MySqlRow) -> Order {
    Order {
        id: db::s(r, "id"),
        plan_id: db::s(r, "plan_id"),
        idempotency_key: db::s(r, "idempotency_key"),
        total_cents: db::i(r, "total_cents"),
        paid_by: db::s(r, "paid_by"),
        payment_provider: db::s(r, "payment_provider"),
        payment_ref: db::s(r, "payment_ref"),
        splits: db::j(r, "splits"),
        test_mode: db::b(r, "test_mode"),
        created_at: db::dt(r, "created_at"),
    }
}

const ORDER_COLS: &str = "id, plan_id, idempotency_key, total_cents, paid_by, payment_provider, payment_ref, splits, test_mode, created_at";

pub async fn orders(db: &Db, hh: &str) -> Result<Vec<Order>> {
    let rows = sqlx::query(&format!("SELECT {ORDER_COLS} FROM orders WHERE household_id = ? ORDER BY created_at DESC"))
        .bind(hh)
        .fetch_all(db)
        .await?;
    Ok(rows.iter().map(order_from_row).collect())
}

pub async fn order_by_key(db: &Db, key: &str) -> Result<Option<Order>> {
    let row = sqlx::query(&format!("SELECT {ORDER_COLS} FROM orders WHERE idempotency_key = ?")).bind(key).fetch_optional(db).await?;
    Ok(row.as_ref().map(order_from_row))
}

pub async fn spent_this_month(db: &Db, hh: &str) -> Result<i64> {
    let n = now();
    let start = n.date().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let v: Option<i64> = sqlx::query_scalar("SELECT CAST(COALESCE(SUM(total_cents), 0) AS SIGNED) FROM orders WHERE household_id = ? AND created_at >= ? AND plan_id NOT LIKE 'out_%'")
        .bind(hh)
        .bind(start)
        .fetch_one(db)
        .await
        .ok();
    Ok(v.unwrap_or(0))
}

#[derive(Clone, Debug, Serialize)]
pub struct Balance {
    pub member_id: String,
    pub name: String,
    /// positive = is owed money, negative = owes
    pub net_cents: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Settlement {
    pub from: String,
    pub to: String,
    pub cents: i64,
}

/// Net balances from order splits, plus the minimal "who pays whom" list.
pub fn balances(members: &[Member], orders: &[Order]) -> (Vec<Balance>, Vec<Settlement>) {
    let mut net: HashMap<String, i64> = members.iter().map(|m| (m.id.clone(), 0)).collect();
    for o in orders {
        *net.entry(o.paid_by.clone()).or_default() += o.total_cents;
        if let Some(arr) = o.splits.as_array() {
            for s in arr {
                let id = s.get("member_id").and_then(Value::as_str).unwrap_or_default().to_string();
                let share = s.get("share_cents").and_then(Value::as_i64).unwrap_or(0);
                *net.entry(id).or_default() -= share;
            }
        }
    }
    let bal: Vec<Balance> = members
        .iter()
        .map(|m| Balance { member_id: m.id.clone(), name: m.name.clone(), net_cents: *net.get(&m.id).unwrap_or(&0) })
        .collect();
    let mut debtors: Vec<(String, i64)> = bal.iter().filter(|b| b.net_cents < 0).map(|b| (b.member_id.clone(), -b.net_cents)).collect();
    let mut creditors: Vec<(String, i64)> = bal.iter().filter(|b| b.net_cents > 0).map(|b| (b.member_id.clone(), b.net_cents)).collect();
    let mut settle = vec![];
    let (mut i, mut j) = (0, 0);
    while i < debtors.len() && j < creditors.len() {
        let amt = debtors[i].1.min(creditors[j].1);
        if amt > 0 {
            settle.push(Settlement { from: debtors[i].0.clone(), to: creditors[j].0.clone(), cents: amt });
        }
        debtors[i].1 -= amt;
        creditors[j].1 -= amt;
        if debtors[i].1 == 0 {
            i += 1;
        }
        if creditors[j].1 == 0 {
            j += 1;
        }
    }
    (bal, settle)
}

/// Equal split, remainder cents to the first members so shares always sum to the total.
pub fn equal_splits(members: &[Member], total_cents: i64) -> Vec<(String, i64)> {
    let n = members.len().max(1) as i64;
    let base = total_cents / n;
    let mut rem = total_cents - base * n;
    members
        .iter()
        .map(|m| {
            let extra = if rem > 0 { 1 } else { 0 };
            rem -= extra;
            (m.id.clone(), base + extra)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn m(id: &str) -> Member {
        Member { id: id.into(), name: id.into(), role: "member".into() }
    }

    #[test]
    fn splits_sum_to_total() {
        let ms = vec![m("a"), m("b"), m("c")];
        let s = equal_splits(&ms, 1000);
        assert_eq!(s.iter().map(|x| x.1).sum::<i64>(), 1000);
    }

    #[test]
    fn payer_is_owed_by_others() {
        let ms = vec![m("maya"), m("sam")];
        let o = Order {
            id: "o".into(), plan_id: "p".into(), idempotency_key: "k".into(), total_cents: 1000, paid_by: "maya".into(),
            payment_provider: "mock".into(), payment_ref: "r".into(),
            splits: json!([{ "member_id": "maya", "share_cents": 500 }, { "member_id": "sam", "share_cents": 500 }]),
            test_mode: true, created_at: None,
        };
        let (bal, settle) = balances(&ms, &[o]);
        assert_eq!(bal[0].net_cents, 500);
        assert_eq!(settle.len(), 1);
        assert_eq!(settle[0].from, "sam");
        assert_eq!(settle[0].cents, 500);
    }
}
