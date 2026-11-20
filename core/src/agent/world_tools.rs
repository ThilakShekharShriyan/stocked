//! Agent actions that reach the outside world (see `crate::world`). Each records an audit event
//! saying which service was called, in which mode, and what came back.

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

use crate::events::{self, Event};
use crate::outings;
use crate::sourcing::money;
use crate::state::{self, now, AppState};
use crate::world::FLIGHTS;

async fn log(st: &AppState, hh: &str, subject_type: &str, subject: &str, service: &str, op: &str, summary: String, extra: Value) -> Result<()> {
    let mut data = json!({ "service": service, "operation": op, "mode": st.world.mode() });
    if let (Some(d), Some(e)) = (data.as_object_mut(), extra.as_object()) {
        d.extend(e.clone());
    }
    events::record(&st.db, hh, Event::new("world_call", subject_type, subject, summary).provenance("observed").data(data)).await?;
    st.notify(hh);
    Ok(())
}

/// Read the venue's own page. If it shows a closure notice, the contingency path runs.
pub async fn check_venue(st: &AppState, hh: &str, outing_id: &str, reporter: &str) -> Result<Value> {
    let o = outings::get(st, hh, outing_id).await?;
    let (open, notice, url) = match st.world.venue_status(&o.venue).await {
        Ok(x) => x,
        Err(err) => {
            log(st, hh, "outing", outing_id, "web", "fetch", format!("Could not read {}’s page: {err}", o.venue), json!({ "ok": false })).await?;
            bail!("Could not reach {}’s page ({err}). Its status is unknown; nothing was changed.", o.venue);
        }
    };
    log(st, hh, "outing", outing_id, "web", "fetch", format!("Read {url}: {}", if open { "open as booked" } else { "CLOSED notice" }), json!({ "url": url, "open": open, "page_text": notice })).await?;
    if open {
        return Ok(json!({ "venue": o.venue, "open": true, "page_says": notice, "source": url }));
    }
    let after = outings::venue_closed(st, hh, outing_id, reporter).await?;
    Ok(json!({
        "venue_checked": o.venue, "open": false, "page_says": notice, "source": url,
        "now": { "venue": after.venue, "each_pays": money(after.quote.per_person_cents), "total": money(after.quote.total_cents), "status": after.status, "disruption": after.booking.get("disruption") },
        "moved_to_backup": after.venue != o.venue,
    }))
}

/// Book the ride the plan chose. A cab is booked on the cab company's site; transit needs no booking.
pub async fn book_ride(st: &AppState, hh: &str, outing_id: &str) -> Result<Value> {
    let o = outings::get(st, hh, outing_id).await?;
    anyhow::ensure!(o.status == "booked", "The outing is not booked yet, so there is no ride to arrange");
    if o.quote.ride.mode != "cab" {
        return Ok(json!({ "ride": o.quote.ride.label, "booking_needed": false, "note": "Transit needs no booking." }));
    }
    if let Some(r) = o.booking.pointer("/ride/confirmation").and_then(Value::as_str) {
        return Ok(json!({ "ride": o.quote.ride.label, "already_booked": true, "confirmation": r }));
    }
    let when = o.starts_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default();
    match st.world.book_cab(o.quote.headcount, &when, "14 Alder St").await {
        Ok(confirmation) => {
            outings::set_ride_confirmation(st, hh, outing_id, &confirmation).await?;
            log(st, hh, "outing", outing_id, "web", "submit", format!("TEST: cab booked for {} — {confirmation}", o.quote.headcount), json!({ "people": o.quote.headcount, "ok": true })).await?;
            Ok(json!({ "ride": o.quote.ride.label, "booked": true, "confirmation": confirmation, "people": o.quote.headcount }))
        }
        Err(err) => {
            log(st, hh, "outing", outing_id, "web", "submit", format!("Cab booking failed: {err}"), json!({ "ok": false })).await?;
            bail!("The cab could not be booked ({err}). No cab is reserved. Transit ({} each) needs no booking and is the fallback.", money(o.pricing_transit_cents()));
        }
    }
}

/// Who has something on their calendar on a given day.
pub async fn check_calendars(st: &AppState, hh: &str, days_from_now: i64) -> Result<Value> {
    let day = (now() + chrono::Duration::days(days_from_now)).date();
    let members = state::members(&st.db, hh).await?;
    let busy: Vec<(String, bool)> = match st.world.busy(day).await {
        Ok(Some(map)) => members.iter().map(|m| (m.name.clone(), *map.get(&m.id).unwrap_or(&false))).collect(),
        Ok(None) => {
            let mut out = vec![];
            for m in &members {
                out.push((m.name.clone(), outings::calendar_conflict(st, hh, m, day.and_hms_opt(19, 0, 0).unwrap()).await?.is_some()));
            }
            out
        }
        Err(err) => {
            log(st, hh, "system", "calendar", "calendar", "freebusy.query", format!("Calendars could not be read: {err}"), json!({ "ok": false })).await?;
            bail!("Calendars could not be read ({err}); availability is unknown.");
        }
    };
    let free: Vec<&str> = busy.iter().filter(|(_, b)| !b).map(|(n, _)| n.as_str()).collect();
    log(st, hh, "system", "calendar", "calendar", "freebusy.query", format!("Checked {} calendars for {day}: {} free", busy.len(), free.len()), json!({ "day": day.to_string() })).await?;
    Ok(json!({ "day": day.to_string(), "free": free, "busy": busy.iter().filter(|(_, b)| *b).map(|(n, _)| n.as_str()).collect::<Vec<_>>() }))
}

pub async fn find_flights(st: &AppState, hh: &str) -> Result<Value> {
    let (flights, url) = st.world.flights().await?;
    log(st, hh, "system", "flights", "web", "fetch", format!("Read {url}: {} fictional flights", flights.len()), json!({ "url": url })).await?;
    Ok(json!({ "source": url, "note": "Fictional airline. Fares are per person, one way.", "flights": flights.iter().map(|f| json!({ "flight": f.code, "route": f.route, "departs": f.departs, "fare": money(f.price_cents) })).collect::<Vec<_>>() }))
}

/// Flights always cost more than the auto-approve limit, so the person booking must have passed
/// the identity check. Nothing is submitted or charged otherwise.
pub async fn book_flight(st: &AppState, hh: &str, member: &str, code: &str, passengers: i64) -> Result<Value> {
    let m = state::find_member(&st.db, hh, member).await?.ok_or_else(|| anyhow!("unknown flatmate {member}"))?;
    let f = FLIGHTS.iter().find(|f| f.code.replace(' ', "").eq_ignore_ascii_case(&code.replace(' ', ""))).ok_or_else(|| anyhow!("no such flight {code}"))?;
    let passengers = passengers.clamp(1, 6);
    let total = f.price_cents * passengers;
    let limit = st.sidecar.config(hh).await.auto_limit_cents;
    if total > limit && crate::verify::current(st, hh, &m.id).await?.is_none() {
        events::record(&st.db, hh, Event::new("approval_blocked_unverified", "system", "flights", format!("Flight booking as {} blocked: {} is over the {} limit and identity is not verified", m.name, money(total), money(limit))).actor(&m.id).provenance("observed")).await?;
        st.notify(hh);
        bail!("VERIFICATION_REQUIRED: {} for {passengers} on {} is over the {} auto-approve limit. {} must verify their identity in the app first: {}/phone/{}. Nothing was booked or charged.", money(total), f.code, money(limit), m.name, st.cfg.web_url, m.id);
    }
    let key = format!("{hh}:flight:{}:{passengers}", f.code.replace(' ', ""));
    let charge = st.payments.charge(total, &key, &format!("Flight {}", f.code)).await?;
    let confirmation = st.world.book_flight(f.code, passengers).await?;
    log(st, hh, "system", "flights", "web", "submit", format!("TEST: {} booked for {passengers}, {} charged — {confirmation}", f.code, money(total)), json!({ "payment_ref": charge.reference, "total_cents": total })).await?;
    Ok(json!({ "booked": true, "flight": f.code, "passengers": passengers, "total": money(total), "confirmation": confirmation }))
}

/// Email the people going with what is booked.
pub async fn send_confirmation(st: &AppState, hh: &str, outing_id: &str) -> Result<Value> {
    let o = outings::get(st, hh, outing_id).await?;
    let to: Vec<String> = o.invitees.iter().filter(|i| i.status == "going").map(|i| format!("{}@alder.test", i.member_id)).collect();
    let body = format!("{} at {} for {}. {} · {} each. TEST MODE: no real booking or payment.", o.title, o.venue, o.quote.headcount, o.quote.ride.label, money(o.quote.per_person_cents));
    let id = st.world.send_mail(&to, &format!("Confirmed: {}", o.title), &body).await?;
    log(st, hh, "outing", outing_id, "mail", "messages.send", format!("Sent the plan to {} people", to.len()), json!({ "message_id": id, "to": to })).await?;
    Ok(json!({ "sent": true, "to": to, "message_id": id }))
}
