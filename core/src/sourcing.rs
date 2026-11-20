//! Online vs in-store, and by when. Deterministic: prices and dates come from the catalog,
//! never from the model.

use anyhow::Result;
use chrono::{Datelike, Duration, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};

use crate::db::{self, Db};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub item_id: String,
    pub store: String,
    pub channel: String,
    pub title: String,
    pub price_cents: i64,
    pub delivery_days: i64,
    pub cutoff_hour: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Option_ {
    pub product: Product,
    pub order_by: Option<NaiveDateTime>,
    pub arrives_at: NaiveDateTime,
    pub feasible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub product: Product,
    pub order_by: Option<NaiveDateTime>,
    pub arrives_at: NaiveDateTime,
    pub feasible: bool,
    pub note: String,
    pub alternatives: Vec<String>,
}

pub async fn products(db: &Db) -> Result<Vec<Product>> {
    let rows = sqlx::query("SELECT id, item_id, store, channel, title, price_cents, delivery_days, cutoff_hour FROM products")
        .fetch_all(db)
        .await?;
    Ok(rows
        .iter()
        .map(|r| Product {
            id: db::s(r, "id"),
            item_id: db::s(r, "item_id"),
            store: db::s(r, "store"),
            channel: db::s(r, "channel"),
            title: db::s(r, "title"),
            price_cents: db::i(r, "price_cents"),
            delivery_days: db::i(r, "delivery_days"),
            cutoff_hour: db::i(r, "cutoff_hour"),
        })
        .collect())
}

fn at_hour(dt: NaiveDateTime, hour: i64) -> NaiveDateTime {
    dt.date().and_hms_opt(hour.clamp(0, 23) as u32, 0, 0).unwrap()
}

pub fn fmt_when(dt: NaiveDateTime) -> String {
    let h = dt.hour();
    let (h12, ampm) = match h {
        0 => (12, "am"),
        1..=11 => (h, "am"),
        12 => (12, "pm"),
        _ => (h - 12, "pm"),
    };
    format!("{} {}{}", dt.weekday(), h12, ampm)
}

pub fn money(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, (cents % 100).abs())
}

pub fn option_for(p: &Product, deadline: Option<NaiveDateTime>, now: NaiveDateTime) -> Option_ {
    if p.channel == "online" {
        let today_cutoff = at_hour(now, p.cutoff_hour);
        let earliest_order = if now < today_cutoff { today_cutoff } else { today_cutoff + Duration::days(1) };
        let earliest_arrival = at_hour(earliest_order + Duration::days(p.delivery_days), 18);
        match deadline {
            Some(d) => {
                let latest_order = at_hour(d - Duration::days(p.delivery_days), p.cutoff_hour);
                let feasible = latest_order >= now && earliest_arrival <= d;
                let arrives = at_hour(latest_order + Duration::days(p.delivery_days), 18).min(d);
                Option_ { product: p.clone(), order_by: Some(latest_order), arrives_at: arrives, feasible }
            }
            None => Option_ { product: p.clone(), order_by: Some(earliest_order), arrives_at: earliest_arrival, feasible: true },
        }
    } else {
        // In-store pickup: opening hour on the deadline day, or the next opening otherwise.
        let pickup = match deadline {
            Some(d) => at_hour(d, p.cutoff_hour),
            None => {
                let open = at_hour(now, p.cutoff_hour);
                if now.hour() < 20 { now.max(open) } else { open + Duration::days(1) }
            }
        };
        let feasible = pickup >= now && deadline.map_or(true, |d| pickup <= d);
        Option_ { product: p.clone(), order_by: None, arrives_at: pickup, feasible }
    }
}

/// Cheapest feasible option; ties go to online (nobody has to go to the store).
pub fn choose(item_id: &str, catalog: &[Product], deadline: Option<NaiveDateTime>, now: NaiveDateTime) -> Option<Choice> {
    let mut opts: Vec<Option_> = catalog.iter().filter(|p| p.item_id == item_id).map(|p| option_for(p, deadline, now)).collect();
    if opts.is_empty() {
        return None;
    }
    opts.sort_by_key(|o| (!o.feasible, o.product.price_cents, o.product.channel != "online"));
    let best = opts[0].clone();
    let alternatives: Vec<String> = opts[1..]
        .iter()
        .map(|o| format!("{} {} {}{}", o.product.store, o.product.channel, money(o.product.price_cents), if o.feasible { "" } else { " (too late)" }))
        .collect();
    let mut note = if best.product.channel == "online" {
        format!(
            "Order online at {} by {} · arrives {}",
            best.product.store,
            best.order_by.map(fmt_when).unwrap_or_default(),
            fmt_when(best.arrives_at)
        )
    } else {
        format!("Pick up at {} {}", best.product.store, fmt_when(best.arrives_at))
    };
    if let Some(other) = opts[1..].iter().find(|o| o.feasible) {
        let diff = other.product.price_cents - best.product.price_cents;
        if diff > 0 {
            note.push_str(&format!(" · {} cheaper than {}", money(diff), other.product.store));
        }
    }
    if !best.feasible {
        note = format!("No option arrives in time ({note})");
    }
    Some(Choice { product: best.product, order_by: best.order_by, arrives_at: best.arrives_at, feasible: best.feasible, note, alternatives })
}
