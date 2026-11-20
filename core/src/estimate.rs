//! Run-out estimates. Aggregation happens in SQL; the rule is a pure function (unit-tested).
//! Nobody enters quantities: an item's rhythm comes from how often the flat re-buys it.

use anyhow::Result;
use chrono::{Duration, NaiveDateTime};
use serde::Serialize;

use crate::db::{self, Db};

#[derive(Clone, Debug, Serialize)]
pub struct ItemStatus {
    pub id: String,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub is_shared_consumable: bool,
    pub purchases: i64,
    pub interval_days: Option<f64>,
    pub last_bought: Option<NaiveDateTime>,
    pub runs_out_on: Option<NaiveDateTime>,
    pub days_left: Option<f64>,
    /// low | soon | ok | cannot_estimate | event_only
    pub status: String,
    pub basis: String,
    /// observed (someone tapped) | inferred (from receipts)
    pub provenance: String,
    pub correction: Option<String>,
    pub corrected_by: Option<String>,
}

pub struct History {
    pub purchases: i64,
    pub first: Option<NaiveDateTime>,
    pub last: Option<NaiveDateTime>,
}

pub struct Correction {
    pub level: String,
    pub at: NaiveDateTime,
}

/// Pure estimate rule.
pub fn estimate(h: &History, c: Option<&Correction>, now: NaiveDateTime) -> (Option<f64>, Option<NaiveDateTime>, String, String) {
    let (interval, mut runs_out, mut basis) = match (h.purchases, h.first, h.last) {
        (n, Some(first), Some(last)) if n >= 2 => {
            let span = (last - first).num_seconds() as f64 / 86_400.0;
            let interval = span / (n - 1) as f64;
            let runs_out = last + Duration::seconds((interval * 86_400.0) as i64);
            (Some(interval), Some(runs_out), format!("Bought every ~{:.0} days ({} receipts)", interval, n))
        }
        (n, _, _) => (None, None, format!("Cannot estimate: only {n} receipt(s)")),
    };
    let mut provenance = "inferred".to_string();
    if let Some(c) = c {
        let after_last = h.last.map_or(true, |l| c.at > l);
        if after_last {
            provenance = "observed".into();
            match c.level.as_str() {
                "out" => {
                    runs_out = Some(c.at);
                    basis = format!("Marked out by a flatmate · usually lasts ~{:.0} days", interval.unwrap_or(0.0));
                }
                "plenty" => {
                    let extra = interval.unwrap_or(7.0);
                    let pushed = c.at + Duration::seconds((extra * 86_400.0) as i64);
                    runs_out = Some(runs_out.map_or(pushed, |r| r.max(pushed)));
                    basis = "Confirmed plenty left by a flatmate".into();
                }
                _ => {}
            }
        }
    }
    let _ = now;
    (interval, runs_out, basis, provenance)
}

pub fn status_for(days_left: Option<f64>, has_history: bool) -> String {
    match days_left {
        Some(d) if d <= 3.0 => "low",
        Some(d) if d <= 7.0 => "soon",
        Some(_) => "ok",
        None if has_history => "cannot_estimate",
        None => "event_only",
    }
    .into()
}

pub async fn item_statuses(db: &Db, hh: &str, now: NaiveDateTime) -> Result<Vec<ItemStatus>> {
    let rows = sqlx::query(
        "SELECT i.id, i.name, i.category, i.unit, i.is_shared_consumable, i.correction, i.correction_at, i.corrected_by,
                COUNT(p.id) AS n, MIN(p.purchased_at) AS first_at, MAX(p.purchased_at) AS last_at
         FROM items i
         LEFT JOIN purchases p ON p.household_id = i.household_id AND p.item_id = i.id AND p.verified = 1
         WHERE i.household_id = ?
         GROUP BY i.id, i.name, i.category, i.unit, i.is_shared_consumable, i.correction, i.correction_at, i.corrected_by",
    )
    .bind(hh)
    .fetch_all(db)
    .await?;

    let mut out: Vec<ItemStatus> = rows
        .iter()
        .map(|r| {
            let h = History { purchases: db::i(r, "n"), first: db::dt(r, "first_at"), last: db::dt(r, "last_at") };
            let correction = db::opt_s(r, "correction");
            let corr = match (&correction, db::dt(r, "correction_at")) {
                (Some(level), Some(at)) => Some(Correction { level: level.clone(), at }),
                _ => None,
            };
            let (interval, runs_out, basis, provenance) = estimate(&h, corr.as_ref(), now);
            let days_left = runs_out.map(|r| ((r - now).num_seconds() as f64 / 86_400.0).max(0.0));
            ItemStatus {
                id: db::s(r, "id"),
                name: db::s(r, "name"),
                category: db::s(r, "category"),
                unit: db::s(r, "unit"),
                is_shared_consumable: db::b(r, "is_shared_consumable"),
                purchases: h.purchases,
                interval_days: interval,
                last_bought: h.last,
                runs_out_on: runs_out,
                days_left,
                status: status_for(days_left, h.purchases > 0),
                basis,
                provenance,
                correction,
                corrected_by: db::opt_s(r, "corrected_by"),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        a.days_left.unwrap_or(f64::MAX).partial_cmp(&b.days_left.unwrap_or(f64::MAX)).unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn t(d: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(12, 0, 0).unwrap()
    }

    #[test]
    fn steady_interval_predicts_next_run_out() {
        let h = History { purchases: 3, first: Some(t(1)), last: Some(t(11)) };
        let (interval, runs_out, _, prov) = estimate(&h, None, t(12));
        assert_eq!(interval, Some(5.0));
        assert_eq!(runs_out, Some(t(16)));
        assert_eq!(prov, "inferred");
    }

    #[test]
    fn single_receipt_cannot_estimate() {
        let h = History { purchases: 1, first: Some(t(1)), last: Some(t(1)) };
        let (_, runs_out, basis, _) = estimate(&h, None, t(2));
        assert!(runs_out.is_none());
        assert!(basis.starts_with("Cannot estimate"));
    }

    #[test]
    fn out_correction_overrides_inference() {
        let h = History { purchases: 3, first: Some(t(1)), last: Some(t(11)) };
        let c = Correction { level: "out".into(), at: t(13) };
        let (_, runs_out, _, prov) = estimate(&h, Some(&c), t(13));
        assert_eq!(runs_out, Some(t(13)));
        assert_eq!(prov, "observed");
    }

    #[test]
    fn correction_before_latest_purchase_is_ignored() {
        let h = History { purchases: 3, first: Some(t(1)), last: Some(t(11)) };
        let c = Correction { level: "out".into(), at: t(5) };
        let (_, runs_out, _, prov) = estimate(&h, Some(&c), t(12));
        assert_eq!(runs_out, Some(t(16)));
        assert_eq!(prov, "inferred");
    }
}
