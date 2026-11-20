//! Outings: plans whose cost depends on who is going (a dinner reservation, a cab, a day trip).
//!
//! Each flatmate has an agent that answers for them from two facts it owns: their calendar and
//! their personal spending cap. The house agent then picks the ride that keeps the most people
//! within budget, books for the real headcount, and re-plans when the headcount changes.
//! All arithmetic is plain code (see `quote`), so "what if we are five?" is answered instantly.

use anyhow::{anyhow, bail, Result};
use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db;
use crate::events::{self, Event};
use crate::sourcing::{fmt_when, money};
use crate::state::{self, now, AppState, Member};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pricing {
    /// Reservation or entry cost per person.
    pub per_head_cents: i64,
    /// Round-trip fare for a standard car (up to 4 people).
    pub cab_cents: i64,
    /// Round-trip fare for a large car (5–6 people).
    pub cab_xl_cents: i64,
    /// Round-trip transit fare per person.
    pub transit_cents: i64,
    /// How much longer transit takes, in minutes.
    pub transit_extra_min: i64,
    /// Where to go instead if the venue falls through.
    #[serde(default)]
    pub backup: Option<Backup>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub venue: String,
    pub per_head_cents: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invitee {
    pub member_id: String,
    pub name: String,
    /// going | declined
    pub status: String,
    pub reason: String,
    /// "agent" when the person's agent answered for them, or the member id when they tapped.
    pub by: String,
    pub cap_cents: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RideOption {
    pub mode: String, // cab | transit
    pub label: String,
    pub total_cents: i64,
    pub per_person_cents: i64,
    pub extra_min: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    pub headcount: i64,
    pub reservation_cents: i64,
    pub ride: RideOption,
    pub alternatives: Vec<RideOption>,
    pub total_cents: i64,
    pub per_person_cents: i64,
    /// Member ids whose cap is below the per-person cost.
    pub over_cap: Vec<String>,
    pub why: String,
}

fn ceil_div(a: i64, b: i64) -> i64 {
    (a + b - 1) / b.max(1)
}

pub fn ride_options(p: &Pricing, n: i64) -> Vec<RideOption> {
    let n = n.max(1);
    let (cab_total, cab_label) = match n {
        1..=4 => (p.cab_cents, "Cab".to_string()),
        5..=6 => (p.cab_xl_cents, "Large cab (5–6 seats)".to_string()),
        _ => (p.cab_cents * ceil_div(n, 4), format!("{} cabs", ceil_div(n, 4))),
    };
    vec![
        RideOption { mode: "cab".into(), label: cab_label, total_cents: cab_total, per_person_cents: ceil_div(cab_total, n), extra_min: 0 },
        RideOption { mode: "transit".into(), label: "Transit".into(), total_cents: p.transit_cents * n, per_person_cents: p.transit_cents, extra_min: p.transit_extra_min },
    ]
}

/// Choose the ride for a given set of caps (one per person going).
/// 1. Keep the most people within their cap. 2. If transit adds over 45 minutes and the cab costs
/// at most $8 more each, take the cab. 3. Otherwise the cheaper one; ties go to the cab.
pub fn quote(p: &Pricing, caps: &[(String, i64)]) -> Quote {
    let n = caps.len().max(1) as i64;
    let options = ride_options(p, n);
    let per_person = |o: &RideOption| p.per_head_cents + o.per_person_cents;
    let over = |o: &RideOption| caps.iter().filter(|(_, cap)| *cap < per_person(o)).map(|(id, _)| id.clone()).collect::<Vec<_>>();
    let (cab, transit) = (&options[0], &options[1]);
    let (cab_over, transit_over) = (over(cab).len(), over(transit).len());
    let (chosen, why) = if cab_over != transit_over {
        let (win, lose) = if cab_over < transit_over { (cab, transit) } else { (transit, cab) };
        (win.clone(), format!("{} keeps everyone in budget at {} each; {} would be {} each", win.label, money(per_person(win)), lose.label.to_lowercase(), money(per_person(lose))))
    } else if transit.extra_min > 45 && cab.per_person_cents - transit.per_person_cents <= 800 && cab.per_person_cents >= transit.per_person_cents {
        (cab.clone(), format!("{} saves {} minutes for {} more each", cab.label, transit.extra_min, money(cab.per_person_cents - transit.per_person_cents)))
    } else if cab.per_person_cents <= transit.per_person_cents {
        (cab.clone(), format!("{} split {} ways is {} each, cheaper than transit at {}", cab.label, n, money(cab.per_person_cents), money(transit.per_person_cents)))
    } else {
        (transit.clone(), format!("Transit is {} each; a {} split {} ways would be {}", money(transit.per_person_cents), cab.label.to_lowercase(), n, money(cab.per_person_cents)))
    };
    let reservation = p.per_head_cents * n;
    Quote {
        headcount: n,
        reservation_cents: reservation,
        per_person_cents: per_person(&chosen),
        total_cents: reservation + chosen.total_cents,
        over_cap: over(&chosen),
        alternatives: options.into_iter().filter(|o| o.mode != chosen.mode).collect(),
        ride: chosen,
        why,
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Outing {
    pub id: String,
    pub version: i64,
    pub title: String,
    pub kind: String,
    pub venue: String,
    pub starts_at: Option<NaiveDateTime>,
    pub organizer: String,
    pub pricing: Pricing,
    pub invitees: Vec<Invitee>,
    pub quote: Quote,
    pub booking: Value,
    pub status: String,
    pub source: String,
    /// The same outing priced for every possible headcount (drives the slider).
    pub what_if: Vec<Quote>,
}

const COLS: &str = "id, version, title, kind, venue, starts_at, organizer, pricing, invitees, quote, booking, status, source";

fn from_row(r: &sqlx::mysql::MySqlRow) -> Option<Outing> {
    let pricing: Pricing = serde_json::from_value(db::j(r, "pricing")).ok()?;
    let invitees: Vec<Invitee> = serde_json::from_value(db::j(r, "invitees")).ok()?;
    let quote: Quote = serde_json::from_value(db::j(r, "quote")).ok()?;
    let what_if = what_if(&pricing, &invitees);
    Some(Outing {
        id: db::s(r, "id"),
        version: db::i(r, "version"),
        title: db::s(r, "title"),
        kind: db::s(r, "kind"),
        venue: db::s(r, "venue"),
        starts_at: db::dt(r, "starts_at"),
        organizer: db::s(r, "organizer"),
        pricing,
        invitees,
        quote,
        booking: db::j(r, "booking"),
        status: db::s(r, "status"),
        source: db::s(r, "source"),
        what_if,
    })
}

/// Price the outing for 1..=everyone. People already going are counted first, then the others.
pub fn what_if(p: &Pricing, invitees: &[Invitee]) -> Vec<Quote> {
    let mut order: Vec<&Invitee> = invitees.iter().filter(|i| i.status == "going").collect();
    order.extend(invitees.iter().filter(|i| i.status != "going"));
    // Up to two extra guests beyond the flat, so "what if we are five or six?" has an answer.
    let mut caps: Vec<(String, i64)> = order.iter().map(|i| (i.member_id.clone(), i.cap_cents)).collect();
    caps.push(("guest_1".into(), 1_000_000));
    caps.push(("guest_2".into(), 1_000_000));
    (1..=caps.len()).map(|n| quote(p, &caps[..n])).collect()
}

pub async fn list(st: &AppState, hh: &str) -> Result<Vec<Outing>> {
    let rows = sqlx::query(&format!("SELECT {COLS} FROM outings WHERE household_id = ? ORDER BY starts_at")).bind(hh).fetch_all(&st.db).await?;
    Ok(rows.iter().filter_map(from_row).collect())
}

pub async fn get(st: &AppState, hh: &str, id: &str) -> Result<Outing> {
    list(st, hh).await?.into_iter().find(|o| o.id == id).ok_or_else(|| anyhow!("no outing {id}"))
}

/// What a person's agent knows: their cap (from the seed) and their own calendar.
fn personal_cap(member_id: &str) -> i64 {
    crate::seed::PERSONAL_CAPS.iter().find(|(id, _)| *id == member_id).map(|(_, c)| *c).unwrap_or(4_000)
}

pub(crate) async fn calendar_conflict(st: &AppState, hh: &str, member: &Member, at: NaiveDateTime) -> Result<Option<String>> {
    let rows = sqlx::query("SELECT title, kind, starts_at, nights FROM calendar_events WHERE household_id = ? AND host_member_id = ?").bind(hh).bind(&member.id).fetch_all(&st.db).await?;
    for r in &rows {
        let (Some(start), kind, nights) = (db::dt(r, "starts_at"), db::s(r, "kind"), db::i(r, "nights")) else { continue };
        let title = db::s(r, "title");
        let same_day = start.date() == at.date();
        let during = at.date() >= start.date() && at.date() <= (start + Duration::days(nights)).date();
        if kind == "away" && during {
            return Ok(Some(format!("“{title}” is on {}’s calendar that day", member.name)));
        }
        if (kind == "party" || kind == "dinner") && same_day {
            return Ok(Some(format!("{} is hosting “{title}” that day", member.name)));
        }
    }
    Ok(None)
}

/// Settle who is going: anyone whose cap is below the per-person cost drops out, lowest cap first,
/// and the outing is re-priced until everyone left can afford it.
fn settle(p: &Pricing, invitees: &mut [Invitee]) -> Quote {
    loop {
        let caps: Vec<(String, i64)> = invitees.iter().filter(|i| i.status == "going").map(|i| (i.member_id.clone(), i.cap_cents)).collect();
        let q = quote(p, &caps);
        let Some(out_id) = q.over_cap.iter().min_by_key(|id| caps.iter().find(|(c, _)| c == *id).map(|(_, cap)| *cap).unwrap_or(0)).cloned() else { return q };
        if caps.len() <= 1 {
            return q;
        }
        if let Some(i) = invitees.iter_mut().find(|i| i.member_id == out_id) {
            i.status = "declined".into();
            i.reason = format!("{} each is over my {} limit for an outing", money(q.per_person_cents), money(i.cap_cents));
            i.by = "agent".into();
        }
    }
}

#[derive(Deserialize)]
pub struct NewOuting {
    pub title: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub days_from_now: Option<i64>,
    #[serde(default)]
    pub organizer: Option<String>,
    #[serde(default)]
    pub pricing: Option<Pricing>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
}

pub async fn create(st: &AppState, hh: &str, n: NewOuting) -> Result<Outing> {
    if let Some(sr) = &n.source_ref {
        let seen: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outings WHERE household_id = ? AND source_ref = ?").bind(hh).bind(sr).fetch_one(&st.db).await.unwrap_or(0);
        if seen > 0 {
            bail!("already planned");
        }
    }
    let members = state::members(&st.db, hh).await?;
    let organizer = n.organizer.clone().unwrap_or_else(|| "priya".into());
    let starts = (now() + Duration::days(n.days_from_now.unwrap_or(3))).date().and_hms_opt(19, 30, 0).unwrap();
    let pricing = n.pricing.clone().unwrap_or(Pricing { per_head_cents: 2800, cab_cents: 2400, cab_xl_cents: 3600, transit_cents: 650, transit_extra_min: 25, backup: Some(Backup { venue: "Nonna's Kitchen".into(), per_head_cents: 2600 }) });
    let id = format!("out_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);

    // Each person's agent answers from their own calendar.
    let mut invitees = vec![];
    for m in &members {
        let conflict = calendar_conflict(st, hh, m, starts).await?;
        invitees.push(Invitee {
            member_id: m.id.clone(),
            name: m.name.clone(),
            status: if conflict.is_some() { "declined".into() } else { "going".into() },
            reason: conflict.unwrap_or_else(|| "Free that evening".into()),
            by: "agent".into(),
            cap_cents: personal_cap(&m.id),
        });
    }
    let quote = settle(&pricing, &mut invitees);
    sqlx::query("INSERT INTO outings (household_id, id, version, title, kind, venue, starts_at, organizer, pricing, invitees, quote, booking, status, source, source_ref, created_at) VALUES (?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, NULL, 'planned', ?, ?, ?)")
        .bind(hh).bind(&id).bind(&n.title).bind(n.kind.as_deref().unwrap_or("dinner")).bind(n.venue.as_deref().unwrap_or("")).bind(starts).bind(&organizer)
        .bind(serde_json::to_string(&pricing)?).bind(serde_json::to_string(&invitees)?).bind(serde_json::to_string(&quote)?)
        .bind(n.source.as_deref().unwrap_or("manual")).bind(&n.source_ref).bind(now())
        .execute(&st.db)
        .await?;
    events::record(&st.db, hh, Event::new("outing_proposed", "outing", &id, format!("{} proposed “{}” · {}", organizer, n.title, fmt_when(starts))).actor(&organizer).provenance("observed").data(json!({ "source": n.source, "pricing": pricing }))).await?;
    for i in &invitees {
        events::record(
            &st.db,
            hh,
            Event::new("agent_rsvp", "outing", &id, format!("{}’s agent: {} — {}", i.name, if i.status == "going" { "in" } else { "out" }, i.reason))
                .actor(&i.member_id)
                .provenance("inferred")
                .data(json!({ "status": i.status, "cap_cents": i.cap_cents, "agent": format!("{}-agent", i.member_id) })),
        )
        .await?;
    }
    record_quote(st, hh, &id, &quote, "Planned").await?;
    st.notify(hh);
    get(st, hh, &id).await
}

async fn record_quote(st: &AppState, hh: &str, id: &str, q: &Quote, verb: &str) -> Result<()> {
    events::record(
        &st.db,
        hh,
        Event::new("outing_priced", "outing", id, format!("{verb} for {}: {} each ({} total). {}", q.headcount, money(q.per_person_cents), money(q.total_cents), q.why)).data(json!({ "quote": q })),
    )
    .await?;
    Ok(())
}

async fn save(st: &AppState, hh: &str, o: &Outing) -> Result<()> {
    sqlx::query("UPDATE outings SET venue = ?, pricing = ?, version = ?, invitees = ?, quote = ?, booking = ?, status = ? WHERE household_id = ? AND id = ?")
        .bind(&o.venue)
        .bind(serde_json::to_string(&o.pricing)?)
        .bind(o.version)
        .bind(serde_json::to_string(&o.invitees)?)
        .bind(serde_json::to_string(&o.quote)?)
        .bind(if o.booking.is_null() { None } else { Some(o.booking.to_string()) })
        .bind(&o.status)
        .bind(hh)
        .bind(&o.id)
        .execute(&st.db)
        .await?;
    Ok(())
}

fn splits(o: &Outing) -> Vec<Value> {
    let going: Vec<Member> = o.invitees.iter().filter(|i| i.status == "going").map(|i| Member { id: i.member_id.clone(), name: i.name.clone(), role: "member".into() }).collect();
    state::equal_splits(&going, o.quote.total_cents).into_iter().map(|(id, c)| json!({ "member_id": id, "share_cents": c })).collect()
}

fn booking_for(o: &Outing, reservation_ref: &str, payment_ref: &str) -> Value {
    json!({
        "test_mode": true,
        "reservation": { "ref": reservation_ref, "party_size": o.quote.headcount, "venue": o.venue },
        "ride": { "mode": o.quote.ride.mode, "label": o.quote.ride.label, "ref": format!("ride_test_{}_{}", o.quote.ride.mode, o.version) },
        "payment_ref": payment_ref,
        "total_cents": o.quote.total_cents,
        "per_person_cents": o.quote.per_person_cents,
        "version": o.version,
    })
}

/// A person (not their agent) changes their answer. A booked outing is re-planned in place:
/// the reservation's party size, the ride and everyone's share all follow the new headcount.
pub async fn rsvp(st: &AppState, hh: &str, id: &str, member_id: &str, status: &str) -> Result<Outing> {
    anyhow::ensure!(status == "going" || status == "declined", "status must be 'going' or 'declined'");
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let mut o = get(st, hh, id).await?;
    let before = o.quote.clone();
    {
        let inv = o.invitees.iter_mut().find(|i| i.member_id == member.id).ok_or_else(|| anyhow!("{} is not invited", member.name))?;
        if inv.status == status {
            return Ok(o);
        }
        inv.status = status.into();
        inv.reason = if status == "going" { "Said they’re in".into() } else { "Said they can’t make it".into() };
        inv.by = member.id.clone();
    }
    events::record(&st.db, hh, Event::new("rsvp_changed", "outing", id, format!("{} {}", member.name, if status == "going" { "is in" } else { "can’t make it" })).actor(&member.id).provenance("observed")).await?;
    let mut invitees = o.invitees.clone();
    o.quote = settle(&o.pricing, &mut invitees);
    for (new, old) in invitees.iter().zip(o.invitees.iter()) {
        if new.status != old.status {
            events::record(&st.db, hh, Event::new("agent_rsvp", "outing", id, format!("{}’s agent: out — {}", new.name, new.reason)).actor(&new.member_id).provenance("inferred")).await?;
        }
    }
    o.invitees = invitees;
    o.version += 1;
    record_quote(st, hh, id, &o.quote, "Re-planned").await?;

    if o.status == "booked" {
        let reservation_ref = o.booking.pointer("/reservation/ref").and_then(Value::as_str).unwrap_or("rsv_test").to_string();
        let payment_ref = o.booking.get("payment_ref").and_then(Value::as_str).unwrap_or_default().to_string();
        let diff = o.quote.total_cents - before.total_cents;
        // Money follows the headcount: the difference is charged or refunded for real (in test mode),
        // once per plan version. If that fails, the booking stays as it was.
        let key = format!("{hh}:{}:v{}", o.id, o.version);
        let moved = if diff > 0 {
            st.payments.charge(diff, &key, &format!("{} (headcount change)", o.title)).await
        } else if diff < 0 {
            st.payments.refund(&payment_ref, -diff, &key).await
        } else {
            Ok(crate::payments::Charge { provider: st.payments.provider_name().into(), reference: String::new(), latency_us: 0, raw: json!({}) })
        };
        let moved = match moved {
            Ok(m) => m,
            Err(err) => {
                events::record(&st.db, hh, Event::new("payment_failed", "outing", id, format!("Could not {} for the new headcount: {err}. The booking is unchanged.", if diff > 0 { "charge the difference" } else { "issue the refund" })).actor(&member.id)).await?;
                st.notify(hh);
                bail!("The booking could not be updated because the payment step failed; it is unchanged and nothing was charged or refunded.");
            }
        };
        o.booking = booking_for(&o, &reservation_ref, &payment_ref);
        let key = format!("{hh}:{}", o.id);
        sqlx::query("UPDATE orders SET total_cents = ?, splits = ? WHERE idempotency_key = ?").bind(o.quote.total_cents).bind(Value::Array(splits(&o)).to_string()).bind(&key).execute(&st.db).await?;
        let mut changes = vec![format!("table for {} → {}", before.headcount, o.quote.headcount)];
        if before.ride.mode != o.quote.ride.mode {
            changes.push(format!("{} → {}", before.ride.label, o.quote.ride.label));
        }
        changes.push(if diff < 0 { format!("{} refunded", money(-diff)) } else { format!("{} more charged", money(diff)) });
        changes.push(format!("{} each (was {})", money(o.quote.per_person_cents), money(before.per_person_cents)));
        events::record(
            &st.db,
            hh,
            Event::new("booking_changed", "outing", id, format!("TEST MODE booking updated: {}", changes.join(" · ")))
                .latency(moved.latency_us)
                .data(json!({ "before": before, "after": o.quote, "reservation_ref": reservation_ref, "money_ref": moved.reference, "difference_cents": diff, "idempotency_key": key })),
        )
        .await?;
    }
    save(st, hh, &o).await?;
    st.notify(hh);
    get(st, hh, id).await
}

impl Outing {
    pub fn pricing_transit_cents(&self) -> i64 {
        self.pricing.transit_cents
    }
}

pub async fn set_ride_confirmation(st: &AppState, hh: &str, id: &str, confirmation: &str) -> Result<()> {
    let mut o = get(st, hh, id).await?;
    if o.booking.is_null() {
        o.booking = json!({});
    }
    o.booking["ride"]["confirmation"] = json!(confirmation);
    save(st, hh, &o).await
}

/// The venue has closed or cancelled. With contingency on, the agent moves the plan to the stored
/// backup by itself, but only when that costs no more than what was already agreed and keeps
/// everyone within their own limit; the difference is refunded. Otherwise it changes nothing and
/// hands the decision to the organiser.
pub async fn venue_closed(st: &AppState, hh: &str, id: &str, reporter: &str) -> Result<Outing> {
    let who = state::find_member(&st.db, hh, reporter).await?.map(|m| m.id).unwrap_or_else(|| reporter.to_string());
    let mut o = get(st, hh, id).await?;
    let closed = o.venue.clone();
    events::record(&st.db, hh, Event::new("venue_closed", "outing", id, format!("{closed} is closed for “{}”", o.title)).actor(&who).provenance("observed")).await?;
    let organiser = o.invitees.iter().find(|i| i.member_id == o.organizer).map(|i| i.name.clone()).unwrap_or_else(|| o.organizer.clone());
    let hold = |o: &mut Outing, text: String| {
        let mut b = if o.booking.is_null() { json!({}) } else { o.booking.clone() };
        b["disruption"] = json!(text);
        o.booking = b;
    };
    let Some(backup) = o.pricing.backup.clone() else {
        hold(&mut o, format!("{closed} is closed and no backup is stored. {organiser} needs to choose what to do."));
        events::record(&st.db, hh, Event::new("needs_organiser", "outing", id, format!("No backup stored; asked {organiser} to decide"))).await?;
        save(st, hh, &o).await?;
        st.notify(hh);
        return get(st, hh, id).await;
    };
    let mut pricing = o.pricing.clone();
    pricing.per_head_cents = backup.per_head_cents;
    pricing.backup = None;
    let mut invitees = o.invitees.clone();
    let quote = settle(&pricing, &mut invitees);
    let same_people = quote.headcount == o.quote.headcount;
    let offer = format!("{} at {} each ({} total)", backup.venue, money(quote.per_person_cents), money(quote.total_cents));
    if !st.sidecar.contingency(hh) {
        hold(&mut o, format!("{closed} is closed. A backup is stored ({offer}), but switching by itself is turned off, so {organiser} needs to decide."));
        events::record(&st.db, hh, Event::new("needs_organiser", "outing", id, format!("Backup available ({offer}) but automatic switching is off; asked {organiser}"))).await?;
        save(st, hh, &o).await?;
        st.notify(hh);
        return get(st, hh, id).await;
    }
    if !same_people || quote.total_cents > o.quote.total_cents {
        let agreed = money(o.quote.total_cents);
        hold(&mut o, format!("{closed} is closed. The backup is {offer}, which is more than the {agreed} agreed{}. Nothing was changed or charged; {organiser} needs to approve it.", if same_people { "" } else { " or over someone's limit" }));
        events::record(&st.db, hh, Event::new("backup_needs_approval", "outing", id, format!("Backup {offer} exceeds what was agreed; not switching without {organiser}")).data(json!({ "backup_quote": quote, "agreed_cents": o.quote.total_cents }))).await?;
        save(st, hh, &o).await?;
        st.notify(hh);
        return get(st, hh, id).await;
    }
    let before = o.quote.clone();
    let diff = quote.total_cents - before.total_cents; // <= 0
    o.version += 1;
    let mut money_ref = String::new();
    if o.status == "booked" && diff < 0 {
        let payment_ref = o.booking.get("payment_ref").and_then(Value::as_str).unwrap_or_default().to_string();
        let key = format!("{hh}:{}:v{}", o.id, o.version);
        match st.payments.refund(&payment_ref, -diff, &key).await {
            Ok(r) => money_ref = r.reference,
            Err(err) => {
                events::record(&st.db, hh, Event::new("payment_failed", "outing", id, format!("Could not refund the difference for the backup: {err}. The booking is unchanged."))).await?;
                st.notify(hh);
                bail!("The switch to the backup could not be completed because the refund step failed; the booking is unchanged.");
            }
        }
    }
    o.venue = backup.venue.clone();
    o.pricing = pricing;
    o.invitees = invitees;
    o.quote = quote;
    if o.status == "booked" {
        let payment_ref = o.booking.get("payment_ref").and_then(Value::as_str).unwrap_or_default().to_string();
        o.booking = booking_for(&o, &format!("rsv_test_{}", &uuid::Uuid::new_v4().simple().to_string()[..8]), &payment_ref);
        sqlx::query("UPDATE orders SET total_cents = ?, splits = ? WHERE idempotency_key = ?").bind(o.quote.total_cents).bind(Value::Array(splits(&o)).to_string()).bind(format!("{hh}:{}", o.id)).execute(&st.db).await?;
    }
    events::record(
        &st.db,
        hh,
        Event::new("replanned_to_backup", "outing", id, format!("TEST MODE: moved to the backup, {} — {} each (was {}){}", o.venue, money(o.quote.per_person_cents), money(before.per_person_cents), if diff < 0 { format!(", {} refunded", money(-diff)) } else { String::new() }))
            .data(json!({ "closed": closed, "before": before, "after": o.quote, "refund_ref": money_ref, "difference_cents": diff })),
    )
    .await?;
    save(st, hh, &o).await?;
    st.notify(hh);
    get(st, hh, id).await
}

/// Book for the current headcount: one reservation, one ride, one charge. Bookings over the house
/// auto-approve limit need the organizer's identity check, like any other over-limit purchase.
pub async fn book(st: &AppState, hh: &str, id: &str, member_id: &str) -> Result<Outing> {
    book_inner(st, hh, id, member_id, false).await
}

/// Simulation set-up only: a household that starts with an outing its organiser already booked.
pub async fn book_as_setup(st: &AppState, hh: &str, id: &str) -> Result<Outing> {
    let organiser = get(st, hh, id).await?.organizer;
    events::record(&st.db, hh, Event::new("simulation_setup", "outing", id, "Simulation set-up: this outing starts already booked by its organiser")).await?;
    book_inner(st, hh, id, &organiser, true).await
}

async fn book_inner(st: &AppState, hh: &str, id: &str, member_id: &str, setup: bool) -> Result<Outing> {
    let member = state::find_member(&st.db, hh, member_id).await?.ok_or_else(|| anyhow!("unknown flatmate {member_id}"))?;
    let mut o = get(st, hh, id).await?;
    if o.status == "booked" {
        events::record(&st.db, hh, Event::new("duplicate_prevented", "outing", id, "Already booked; not booking or charging again").actor(&member.id)).await?;
        return Ok(o);
    }
    anyhow::ensure!(member.id == o.organizer, "Only the organizer ({}) can book this", o.organizer);
    anyhow::ensure!(o.quote.over_cap.is_empty(), "Someone going is over their limit; re-plan first");
    let cfg = st.sidecar.config(hh).await;
    if !setup && o.quote.total_cents > cfg.auto_limit_cents && crate::verify::current(st, hh, &member.id).await?.is_none() {
        events::record(&st.db, hh, Event::new("approval_blocked_unverified", "outing", id, format!("Booking as {} blocked: identity not verified", member.name)).actor(&member.id).provenance("observed")).await?;
        st.notify(hh);
        bail!("VERIFICATION_REQUIRED: {} must verify their identity before booking {}. Open {}/phone/{} and tap “Verify it’s you”.", member.name, money(o.quote.total_cents), st.cfg.web_url, member.id);
    }
    let _guard = st.order_lock.lock().await;
    let key = format!("{hh}:{}", o.id);
    let charge = match st.payments.charge(o.quote.total_cents, &key, &o.title).await {
        Ok(c) => c,
        Err(err) => {
            events::record(&st.db, hh, Event::new("payment_failed", "outing", id, format!("Payment failed: {err}. Nothing was booked or charged; safe to retry.")).actor(&member.id)).await?;
            st.notify(hh);
            return Err(err);
        }
    };
    let reservation_ref = format!("rsv_test_{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
    o.booking = booking_for(&o, &reservation_ref, &charge.reference);
    o.status = "booked".into();
    let mut tx = st.db.begin().await?;
    sqlx::query("INSERT IGNORE INTO orders (idempotency_key, household_id, id, plan_id, total_cents, paid_by, payment_provider, payment_ref, splits, test_mode, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?)")
        .bind(&key).bind(hh).bind(format!("ord_{}", &uuid::Uuid::new_v4().simple().to_string()[..10])).bind(&o.id).bind(o.quote.total_cents).bind(&member.id).bind(&charge.provider).bind(&charge.reference).bind(Value::Array(splits(&o)).to_string()).bind(now())
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE outings SET booking = ?, status = 'booked' WHERE household_id = ? AND id = ?").bind(o.booking.to_string()).bind(hh).bind(&o.id).execute(&mut *tx).await?;
    tx.commit().await?;
    events::record(
        &st.db,
        hh,
        Event::new("outing_booked", "outing", id, format!("TEST MODE: table for {} at {} ({}), {} booked, {} charged — {} each", o.quote.headcount, o.venue, reservation_ref, o.quote.ride.label.to_lowercase(), money(o.quote.total_cents), money(o.quote.per_person_cents)))
            .actor(&member.id)
            .latency(charge.latency_us)
            .data(json!({ "booking": o.booking, "idempotency_key": key, "total_cents": o.quote.total_cents })),
    )
    .await?;
    st.notify(hh);
    get(st, hh, id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dinner() -> Pricing {
        Pricing { per_head_cents: 2800, cab_cents: 2400, cab_xl_cents: 3600, transit_cents: 650, transit_extra_min: 25, backup: None }
    }
    fn caps(v: &[i64]) -> Vec<(String, i64)> {
        v.iter().enumerate().map(|(i, c)| (format!("m{i}"), *c)).collect()
    }

    #[test]
    fn ride_flips_with_headcount() {
        assert_eq!(quote(&dinner(), &caps(&[9000; 3])).ride.mode, "transit"); // $6.50 vs $8.00 each
        assert_eq!(quote(&dinner(), &caps(&[9000; 4])).ride.mode, "cab"); // $6.00 vs $6.50
        assert_eq!(quote(&dinner(), &caps(&[9000; 5])).ride.mode, "transit"); // large cab $7.20
        assert_eq!(quote(&dinner(), &caps(&[9000; 6])).ride.mode, "cab"); // large cab $6.00
    }

    #[test]
    fn picks_the_ride_that_keeps_everyone_in_budget() {
        // Three people: a cab is $36 each, over one $35 cap; transit is $34.50 and keeps them in.
        let q = quote(&dinner(), &caps(&[6000, 3500, 5000]));
        assert_eq!(q.ride.mode, "transit");
        assert!(q.over_cap.is_empty());
        assert_eq!(q.per_person_cents, 3450);
    }

    #[test]
    fn long_transit_loses_to_a_slightly_dearer_cab() {
        let trip = Pricing { per_head_cents: 1200, cab_cents: 9600, cab_xl_cents: 13200, transit_cents: 1800, transit_extra_min: 70, backup: None };
        assert_eq!(quote(&trip, &caps(&[9000; 4])).ride.mode, "cab"); // $24 vs $18, 70 min saved
        assert_eq!(quote(&trip, &caps(&[9000; 3])).ride.mode, "transit"); // $32 vs $18: too much more
    }

    #[test]
    fn over_cap_person_drops_out_and_price_is_recomputed() {
        let mut inv: Vec<Invitee> = [("a", 9000), ("b", 9000), ("c", 3000)]
            .iter()
            .map(|(id, cap)| Invitee { member_id: id.to_string(), name: id.to_string(), status: "going".into(), reason: String::new(), by: "agent".into(), cap_cents: *cap })
            .collect();
        let q = settle(&dinner(), &mut inv);
        assert_eq!(inv[2].status, "declined");
        assert_eq!(q.headcount, 2);
        assert!(q.over_cap.is_empty());
    }

    #[test]
    fn totals_add_up() {
        let q = quote(&dinner(), &caps(&[9000; 4]));
        assert_eq!(q.total_cents, 4 * 2800 + 2400);
        assert_eq!(q.per_person_cents, 2800 + 600);
    }
}
