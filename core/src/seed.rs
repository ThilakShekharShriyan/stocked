//! Fictional demo flat ("14 Alder St") and a fictional store catalog.
//! Dates are generated relative to *now* so the demo always shows the same story.
//! Each Firedrill drill session gets its own cloned household (same seed, different household_id).

use anyhow::Result;
use chrono::{Duration, NaiveDateTime, Utc};

use crate::db::Db;

pub struct SeedItem {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub unit: &'static str,
    pub shared: bool,
    /// (interval_days, days_since_last, qty_per_purchase, receipt line text, price cents) — None = no history
    pub history: Option<(i64, i64, i64, &'static str, i64)>,
}

pub const MEMBERS: &[(&str, &str, &str)] = &[
    ("maya", "Maya", "approver"),
    ("sam", "Sam", "member"),
    ("priya", "Priya", "member"),
    ("leo", "Leo", "member"),
];

/// Each person's own limit for a single outing; only their agent uses it.
pub const PERSONAL_CAPS: &[(&str, i64)] = &[("maya", 6_000), ("sam", 3_500), ("priya", 5_000), ("leo", 4_500)];

pub const ITEMS: &[SeedItem] = &[
    SeedItem { id: "dish_soap", name: "Dish soap", category: "kitchen", unit: "bottle", shared: true, history: Some((11, 9, 2, "BrightSuds Dish Soap 28 oz   x2   $7.98", 399)) },
    SeedItem { id: "toilet_paper", name: "Toilet paper", category: "bathroom", unit: "12-roll pack", shared: true, history: Some((14, 13, 1, "CloudSoft Toilet Paper 12 rolls   x1   $11.49", 1149)) },
    SeedItem { id: "trash_bags", name: "Trash bags", category: "cleaning", unit: "box", shared: true, history: Some((21, 18, 1, "TuffSack Kitchen Bags 40 ct   x1   $8.99", 899)) },
    SeedItem { id: "coffee", name: "Coffee", category: "pantry", unit: "bag", shared: true, history: Some((8, 2, 1, "Morning Hill Ground Coffee 12 oz   x1   $9.49", 949)) },
    SeedItem { id: "hand_soap", name: "Hand soap", category: "bathroom", unit: "bottle", shared: true, history: Some((18, 12, 2, "Meadow Hand Soap 11 oz   x2   $6.98", 349)) },
    SeedItem { id: "paper_towels", name: "Paper towels", category: "kitchen", unit: "6-roll pack", shared: true, history: Some((12, 5, 1, "Absorbo Paper Towels 6 rolls   x1   $12.99", 1299)) },
    SeedItem { id: "laundry_detergent", name: "Laundry detergent", category: "cleaning", unit: "bottle", shared: true, history: Some((30, 10, 1, "FreshFold Laundry Detergent 64 loads   x1   $14.99", 1499)) },
    SeedItem { id: "olive_oil", name: "Olive oil", category: "pantry", unit: "bottle", shared: true, history: Some((25, 5, 1, "Verde Olive Oil 1 L   x1   $10.99", 1099)) },
    SeedItem { id: "sponges", name: "Sponges", category: "kitchen", unit: "6-pack", shared: true, history: Some((20, 3, 1, "ScrubPro Sponges 6 ct   x1   $4.49", 449)) },
    // Party supplies: no purchase history, only bought for events.
    SeedItem { id: "chips", name: "Chips", category: "party", unit: "bag", shared: true, history: None },
    SeedItem { id: "soda", name: "Soda", category: "party", unit: "2 L bottle", shared: true, history: None },
    SeedItem { id: "paper_plates", name: "Paper plates", category: "party", unit: "50-pack", shared: true, history: None },
    SeedItem { id: "cups", name: "Cups", category: "party", unit: "50-pack", shared: true, history: None },
    SeedItem { id: "napkins", name: "Napkins", category: "party", unit: "100-pack", shared: true, history: None },
    SeedItem { id: "ice", name: "Ice", category: "party", unit: "5 lb bag", shared: true, history: None },
    // Not a shared consumable: exists so requests for it can be matched and refused.
    SeedItem { id: "speaker", name: "Bluetooth speaker", category: "electronics", unit: "each", shared: false, history: None },
];

/// (product id, item id, store, channel, title, description, price cents, delivery days, cutoff hour)
pub const PRODUCTS: &[(&str, &str, &str, &str, &str, &str, i64, i64, i64)] = &[
    ("sk_dish_soap", "dish_soap", "ShipKit", "online", "BrightSuds Dish Soap 28 oz", "Liquid dish soap for washing dishes by hand, grease cutting", 399, 2, 18),
    ("cm_dish_soap", "dish_soap", "CornerMart", "instore", "BrightSuds Dish Soap 28 oz", "Liquid dish soap for washing dishes by hand, grease cutting", 429, 0, 8),
    ("sk_toilet_paper", "toilet_paper", "ShipKit", "online", "CloudSoft Toilet Paper 12 rolls", "Two-ply bathroom tissue, 12 double rolls", 1149, 2, 18),
    ("cm_toilet_paper", "toilet_paper", "CornerMart", "instore", "CloudSoft Toilet Paper 12 rolls", "Two-ply bathroom tissue, 12 double rolls", 1099, 0, 8),
    ("sk_trash_bags", "trash_bags", "ShipKit", "online", "TuffSack Kitchen Bags 40 ct", "13 gallon drawstring kitchen trash bags", 899, 2, 18),
    ("cm_trash_bags", "trash_bags", "CornerMart", "instore", "TuffSack Kitchen Bags 40 ct", "13 gallon drawstring kitchen trash bags", 949, 0, 8),
    ("sk_coffee", "coffee", "ShipKit", "online", "Morning Hill Ground Coffee 12 oz", "Medium roast ground coffee", 949, 2, 18),
    ("cm_coffee", "coffee", "CornerMart", "instore", "Morning Hill Ground Coffee 12 oz", "Medium roast ground coffee", 899, 0, 8),
    ("sk_hand_soap", "hand_soap", "ShipKit", "online", "Meadow Hand Soap 11 oz", "Liquid hand soap pump bottle", 349, 2, 18),
    ("sk_paper_towels", "paper_towels", "ShipKit", "online", "Absorbo Paper Towels 6 rolls", "Kitchen paper towels for spills and cleanup", 1299, 2, 18),
    ("cm_paper_towels", "paper_towels", "CornerMart", "instore", "Absorbo Paper Towels 6 rolls", "Kitchen paper towels for spills and cleanup", 1349, 0, 8),
    ("sk_laundry_detergent", "laundry_detergent", "ShipKit", "online", "FreshFold Laundry Detergent 64 loads", "Liquid laundry detergent", 1499, 2, 18),
    ("sk_olive_oil", "olive_oil", "ShipKit", "online", "Verde Olive Oil 1 L", "Extra virgin olive oil for cooking", 1099, 2, 18),
    ("sk_sponges", "sponges", "ShipKit", "online", "ScrubPro Sponges 6 ct", "Non-scratch kitchen sponges", 449, 2, 18),
    ("sk_chips", "chips", "ShipKit", "online", "Crunchwell Tortilla Chips family size", "Party size tortilla chips snack bag", 399, 2, 18),
    ("cm_chips", "chips", "CornerMart", "instore", "Crunchwell Tortilla Chips family size", "Party size tortilla chips snack bag", 349, 0, 8),
    ("sk_soda", "soda", "ShipKit", "online", "Fizzwell Cola 2 L", "Cola soft drink bottle for parties", 249, 2, 18),
    ("cm_soda", "soda", "CornerMart", "instore", "Fizzwell Cola 2 L", "Cola soft drink bottle for parties", 229, 0, 8),
    ("sk_paper_plates", "paper_plates", "ShipKit", "online", "Sturdy Paper Plates 50 ct", "Disposable paper plates for parties", 599, 2, 18),
    ("sk_cups", "cups", "ShipKit", "online", "Party Cups 16 oz 50 ct", "Disposable plastic party cups", 499, 2, 18),
    ("cm_cups", "cups", "CornerMart", "instore", "Party Cups 16 oz 50 ct", "Disposable plastic party cups", 549, 0, 8),
    ("sk_napkins", "napkins", "ShipKit", "online", "Everyday Napkins 100 ct", "Paper napkins", 349, 2, 18),
    ("cm_ice", "ice", "CornerMart", "instore", "Party Ice 5 lb", "Bagged ice for drinks; in-store only", 299, 0, 8),
    ("sk_speaker", "speaker", "ShipKit", "online", "Boomlet Bluetooth Party Speaker", "Portable wireless bluetooth speaker with bass", 8999, 2, 18),
];

const RECEIPT_STORES: &[(&str, &str)] = &[("ShipKit", "orders@shipkit.example"), ("CornerMart", "receipts@cornermart.example")];

fn days_ago(now: NaiveDateTime, d: i64) -> NaiveDateTime {
    now - Duration::days(d)
}

/// Remove every row of a household (used by reset and before re-seeding drill households).
pub async fn wipe_household(db: &Db, hh: &str) -> Result<()> {
    for t in ["members", "items", "receipts", "purchases", "calendar_events", "plans", "orders", "agent_events", "verifications", "outings"] {
        sqlx::query(&format!("DELETE FROM {t} WHERE household_id = ?")).bind(hh).execute(db).await?;
    }
    Ok(())
}

pub async fn household_exists(db: &Db, hh: &str) -> Result<bool> {
    let n: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE household_id = ?")
        .bind(hh)
        .fetch_one(db)
        .await
        .ok();
    Ok(n.unwrap_or(0) > 0)
}

pub async fn seed_household(db: &Db, hh: &str) -> Result<()> {
    wipe_household(db, hh).await?;
    let now = Utc::now().naive_utc();

    let mut q = sqlx::QueryBuilder::<sqlx::MySql>::new("INSERT INTO members (household_id, id, name, role) ");
    q.push_values(MEMBERS.iter(), |mut b, (id, name, role)| {
        b.push_bind(hh).push_bind(*id).push_bind(*name).push_bind(*role);
    });
    q.build().execute(db).await?;

    let mut q = sqlx::QueryBuilder::<sqlx::MySql>::new("INSERT INTO items (household_id, id, name, category, unit, is_shared_consumable) ");
    q.push_values(ITEMS.iter(), |mut b, it| {
        b.push_bind(hh).push_bind(it.id).push_bind(it.name).push_bind(it.category).push_bind(it.unit).push_bind(it.shared);
    });
    q.build().execute(db).await?;

    // Four past purchases per staple at a steady interval, the latest `since_last` days ago.
    struct R { id: String, member: &'static str, store: &'static str, at: NaiveDateTime, subject: String, raw: String }
    struct P { receipt: String, item: &'static str, qty: i64, price: i64, at: NaiveDateTime, line: &'static str, verified: bool }
    let (mut receipts, mut purchases) = (vec![], vec![]);
    let mut payer = 0usize;
    for it in ITEMS {
        let Some((interval, since_last, qty, line, price)) = it.history else { continue };
        for k in (0..4).rev() {
            let at = days_ago(now, since_last + k * interval);
            let (store, from) = RECEIPT_STORES[(payer + k as usize) % RECEIPT_STORES.len()];
            let member = MEMBERS[payer % MEMBERS.len()];
            payer += 1;
            let id = uuid::Uuid::new_v4().to_string();
            let subject = format!("Your {store} order is confirmed");
            let raw = format!(
                "From: {from}\nSubject: {subject}\nDate: {}\n\nHi {},\nThanks for shopping with {store}.\n\n  {line}\n\nPaid with card ending 4242.\n{store} — 14 Alder St",
                at.format("%a %b %-d %Y"),
                member.1
            );
            purchases.push(P { receipt: id.clone(), item: it.id, qty, price, at, line, verified: raw.contains(line) });
            receipts.push(R { id, member: member.0, store, at, subject, raw });
        }
    }
    let mut q = sqlx::QueryBuilder::<sqlx::MySql>::new("INSERT INTO receipts (household_id, id, source, source_ref, member_id, store, received_at, subject, raw_text) ");
    q.push_values(receipts.iter(), |mut b, r| {
        b.push_bind(hh).push_bind(&r.id).push_bind("seeded").push_bind(None::<String>).push_bind(r.member).push_bind(r.store).push_bind(r.at).push_bind(&r.subject).push_bind(&r.raw);
    });
    q.build().execute(db).await?;
    let mut q = sqlx::QueryBuilder::<sqlx::MySql>::new("INSERT INTO purchases (household_id, id, receipt_id, item_id, qty, unit_price_cents, purchased_at, evidence_span, verified, extractor) ");
    q.push_values(purchases.iter(), |mut b, p| {
        b.push_bind(hh).push_bind(uuid::Uuid::new_v4().to_string()).push_bind(&p.receipt).push_bind(p.item).push_bind(p.qty).push_bind(p.price).push_bind(p.at).push_bind(p.line).push_bind(p.verified).push_bind("seed");
    });
    q.build().execute(db).await?;
    Ok(())
}

/// Store catalog is global (not per household). Inserted once.
pub async fn seed_products(db: &Db) -> Result<bool> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products").fetch_one(db).await.unwrap_or(0);
    if n > 0 {
        return Ok(false);
    }
    for (id, item, store, channel, title, desc, price, days, cutoff) in PRODUCTS {
        sqlx::query("INSERT INTO products (id, item_id, store, channel, title, description, price_cents, delivery_days, cutoff_hour) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(id).bind(item).bind(store).bind(channel).bind(title).bind(desc).bind(price).bind(days).bind(cutoff)
            .execute(db).await?;
    }
    Ok(true)
}
