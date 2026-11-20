//! The verifier. The model only proposes a basket; these pure functions decide whether it
//! auto-approves, waits for the approver, or is refused. Every check reports a number.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    pub item_id: String,
    pub name: String,
    pub qty: i64,
    pub unit_price_cents: i64,
    pub is_shared_consumable: bool,
    /// None when no catalog product matched (price cannot be verified)
    pub product_id: Option<String>,
    pub store: Option<String>,
    pub channel: Option<String>,
    pub sourcing_note: String,
    pub arrives_in_time: Option<bool>,
    pub why: String,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub lines: Vec<Line>,
    pub budget_cents: i64,
    pub spent_cents: i64,
    pub auto_limit_cents: i64,
    pub has_approver: bool,
    /// Model's own claimed total, if it gave one — recorded, never trusted.
    pub claimed_total_cents: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    CannotMeasure,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub status: Status,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    AutoApprove,
    Hold,
    Refuse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verdict {
    pub outcome: Outcome,
    pub total_cents: i64,
    pub checks: Vec<Check>,
    pub reasons: Vec<String>,
}

pub const MAX_QTY_PER_LINE: i64 = 24;

fn check(name: &str, status: Status, detail: impl Into<String>) -> Check {
    Check { name: name.into(), status, detail: detail.into() }
}

fn money(c: i64) -> String {
    crate::sourcing::money(c)
}

pub fn evaluate(inp: &Input) -> Verdict {
    let mut checks = vec![];
    let mut refuse = vec![];
    let mut hold = vec![];

    if inp.lines.is_empty() {
        checks.push(check("basket_not_empty", Status::Fail, "Nothing to buy"));
        return Verdict { outcome: Outcome::Refuse, total_cents: 0, checks, reasons: vec!["Nothing to buy".into()] };
    }

    // 1. Shared consumables only.
    let not_shared: Vec<&str> = inp.lines.iter().filter(|l| !l.is_shared_consumable).map(|l| l.name.as_str()).collect();
    if not_shared.is_empty() {
        checks.push(check("shared_consumables_only", Status::Pass, format!("{} item(s), all shared household supplies", inp.lines.len())));
    } else {
        let msg = format!("Not a shared household supply: {}", not_shared.join(", "));
        checks.push(check("shared_consumables_only", Status::Fail, &msg));
        refuse.push(msg);
    }

    // 2. Every price comes from the catalog.
    let unpriced: Vec<&str> = inp.lines.iter().filter(|l| l.product_id.is_none()).map(|l| l.name.as_str()).collect();
    if unpriced.is_empty() {
        checks.push(check("prices_from_catalog", Status::Pass, "Every line priced from the store catalog"));
    } else {
        let msg = format!("No catalog price for: {}", unpriced.join(", "));
        checks.push(check("prices_from_catalog", Status::CannotMeasure, &msg));
        hold.push(msg);
    }

    // 3. Quantities are sane.
    let big: Vec<String> = inp.lines.iter().filter(|l| l.qty < 1 || l.qty > MAX_QTY_PER_LINE).map(|l| format!("{} ×{}", l.name, l.qty)).collect();
    if big.is_empty() {
        checks.push(check("quantities_sane", Status::Pass, format!("All quantities between 1 and {MAX_QTY_PER_LINE}")));
    } else {
        let msg = format!("Unusual quantity: {}", big.join(", "));
        checks.push(check("quantities_sane", Status::Fail, &msg));
        hold.push(msg);
    }

    // 4. Total recomputed in code.
    let total: i64 = inp.lines.iter().map(|l| l.qty * l.unit_price_cents).sum();
    match inp.claimed_total_cents {
        Some(c) if c != total => checks.push(check(
            "total_recomputed",
            Status::Pass,
            format!("Recomputed {} from catalog prices (model said {}; ignored)", money(total), money(c)),
        )),
        _ => checks.push(check("total_recomputed", Status::Pass, format!("Recomputed {} from catalog prices", money(total)))),
    }

    // 5. Arrives in time.
    let late: Vec<&str> = inp.lines.iter().filter(|l| l.arrives_in_time == Some(false)).map(|l| l.name.as_str()).collect();
    if late.is_empty() {
        checks.push(check("arrives_in_time", Status::Pass, "Every item arrives or can be picked up before it's needed"));
    } else {
        let msg = format!("Can't arrive in time: {}", late.join(", "));
        checks.push(check("arrives_in_time", Status::Fail, &msg));
        hold.push(msg);
    }

    // 6. Monthly budget.
    let remaining = inp.budget_cents - inp.spent_cents;
    if total <= remaining {
        checks.push(check("within_monthly_budget", Status::Pass, format!("{} of {} left this month", money(remaining), money(inp.budget_cents))));
    } else {
        let msg = format!("Over the monthly budget: {} needed, {} left", money(total), money(remaining));
        checks.push(check("within_monthly_budget", Status::Fail, &msg));
        refuse.push(msg);
    }

    // 7. Auto-approve limit.
    let over_limit = total > inp.auto_limit_cents;
    if over_limit {
        let msg = format!("{} is over the {} auto-approve limit", money(total), money(inp.auto_limit_cents));
        checks.push(check("under_auto_limit", Status::Fail, &msg));
        hold.push(msg);
    } else {
        checks.push(check("under_auto_limit", Status::Pass, format!("{} is under the {} auto-approve limit", money(total), money(inp.auto_limit_cents))));
    }

    // 8. Someone can approve.
    if !hold.is_empty() && refuse.is_empty() {
        if inp.has_approver {
            checks.push(check("approver_available", Status::Pass, "House approver will be asked"));
        } else {
            let msg = "No approver in this household".to_string();
            checks.push(check("approver_available", Status::Fail, &msg));
            refuse.push(msg);
        }
    }

    let (outcome, reasons) = if !refuse.is_empty() {
        (Outcome::Refuse, refuse)
    } else if !hold.is_empty() {
        (Outcome::Hold, hold)
    } else {
        (Outcome::AutoApprove, vec!["All checks passed".into()])
    };
    Verdict { outcome, total_cents: total, checks, reasons }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(name: &str, qty: i64, price: i64, shared: bool) -> Line {
        Line {
            item_id: name.into(),
            name: name.into(),
            qty,
            unit_price_cents: price,
            is_shared_consumable: shared,
            product_id: Some(format!("p_{name}")),
            store: Some("ShipKit".into()),
            channel: Some("online".into()),
            sourcing_note: String::new(),
            arrives_in_time: Some(true),
            why: String::new(),
        }
    }

    fn input(lines: Vec<Line>) -> Input {
        Input { lines, budget_cents: 30_000, spent_cents: 0, auto_limit_cents: 4_000, has_approver: true, claimed_total_cents: None }
    }

    #[test]
    fn small_basket_auto_approves() {
        let v = evaluate(&input(vec![line("dish soap", 2, 399, true)]));
        assert_eq!(v.outcome, Outcome::AutoApprove);
        assert_eq!(v.total_cents, 798);
    }

    #[test]
    fn over_limit_is_held_for_approver() {
        let v = evaluate(&input(vec![line("chips", 3, 349, true), line("plates", 1, 599, true), line("tp", 3, 1149, true)]));
        assert_eq!(v.outcome, Outcome::Hold);
    }

    #[test]
    fn non_consumable_is_refused() {
        let v = evaluate(&input(vec![line("speaker", 1, 8999, false)]));
        assert_eq!(v.outcome, Outcome::Refuse);
        assert!(v.reasons.iter().any(|r| r.contains("Not a shared household supply")));
    }

    #[test]
    fn over_monthly_budget_is_refused() {
        let mut i = input(vec![line("tp", 2, 1149, true)]);
        i.spent_cents = 29_000;
        assert_eq!(evaluate(&i).outcome, Outcome::Refuse);
    }

    #[test]
    fn model_total_is_ignored() {
        let mut i = input(vec![line("soap", 2, 399, true)]);
        i.claimed_total_cents = Some(100);
        let v = evaluate(&i);
        assert_eq!(v.total_cents, 798);
    }

    #[test]
    fn unpriced_line_cannot_auto_approve() {
        let mut l = line("mystery", 1, 0, true);
        l.product_id = None;
        let v = evaluate(&input(vec![l]));
        assert_eq!(v.outcome, Outcome::Hold);
    }

    #[test]
    fn no_approver_refuses_held_plan() {
        let mut i = input(vec![line("tp", 4, 1149, true)]);
        i.has_approver = false;
        assert_eq!(evaluate(&i).outcome, Outcome::Refuse);
    }
}
