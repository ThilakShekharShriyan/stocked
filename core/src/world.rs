//! The outside world the agent reaches: venue and booking sites, calendars and mail.
//!
//! Two modes, and the mode is recorded on every call:
//! - `firedrill-synthetic`: Firedrill's synthetic Web Corpus, Google Calendar and Gmail, when a
//!   drill provides their invoke URLs. Real request shapes, fictional data, nothing leaves Firedrill.
//! - `simulated-local`: a small built-in stand-in for the dashboard, labelled SIMULATED.
//!
//! Every venue, cab company, airline and address here is fictional.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct World {
    http: reqwest::Client,
    web: Option<String>,
    calendar: Option<String>,
    mail: Option<String>,
    /// Local mode: venues currently showing a closure notice.
    closed: Arc<Mutex<HashSet<String>>>,
    /// Synthetic mode: flatmate id -> calendar id created at seed time.
    calendars: Arc<Mutex<HashMap<String, String>>>,
    seeded: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Flight {
    pub code: &'static str,
    pub route: &'static str,
    pub departs: &'static str,
    pub price_cents: i64,
}

pub const FLIGHTS: &[Flight] = &[
    Flight { code: "SH 212", route: "SFO → DEN", departs: "07:10", price_cents: 12_900 },
    Flight { code: "SH 318", route: "SFO → DEN", departs: "12:45", price_cents: 9_800 },
    Flight { code: "SH 440", route: "SFO → DEN", departs: "18:20", price_cents: 15_400 },
];

pub const VENUES: &[&str] = &["Luca's Trattoria", "Nonna's Kitchen", "Fern Cafe", "The Glasshouse", "Point Reyes"];
const CAB_URL: &str = "http://swiftcab.test/book";
const FLIGHTS_URL: &str = "http://skyhop.test/flights";
const FLIGHT_BOOK_URL: &str = "http://skyhop.test/book";

pub fn host(venue: &str) -> String {
    let slug: String = venue.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    format!("{slug}.test")
}

fn venue_text(venue: &str, closed: bool) -> String {
    if closed {
        format!("{venue} (fictional). NOTICE: CLOSED for a private event on the night of your booking. We apologise; existing reservations are cancelled.")
    } else {
        format!("{venue} (fictional). Open daily 5pm to 10pm. Reservations are honoured as booked.")
    }
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl World {
    pub fn from_env() -> Self {
        World {
            http: reqwest::Client::builder().timeout(Duration::from_secs(20)).build().unwrap(),
            web: var("WEB_INVOKE_URL"),
            calendar: var("CALENDAR_INVOKE_URL"),
            mail: var("MAIL_INVOKE_URL"),
            closed: Arc::new(Mutex::new(HashSet::new())),
            calendars: Arc::new(Mutex::new(HashMap::new())),
            seeded: Arc::new(Mutex::new(false)),
        }
    }

    pub fn mode(&self) -> &'static str {
        if self.web.is_some() { "firedrill-synthetic" } else { "simulated-local" }
    }

    async fn invoke(&self, url: &str, operation: &str, arguments: Value) -> Result<Value> {
        let resp = self.http.post(url).json(&json!({ "operation": operation, "arguments": arguments })).send().await?;
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(json!({}));
        if !status.is_success() {
            bail!("{operation} failed (HTTP {status}): {}", body.pointer("/error/message").and_then(Value::as_str).unwrap_or("service unavailable"));
        }
        Ok(body)
    }

    /// Publish the fictional sites and calendars a drill starts with. Failures are tolerated:
    /// a scenario may begin with a service already down.
    pub async fn seed(&self, closed_venue: Option<&str>, members: &[(String, String)], away: Option<(&str, i64)>) -> usize {
        let mut done = self.seeded.lock().await;
        if *done {
            return 0;
        }
        *done = true;
        let mut n = 0;
        if let Some(web) = &self.web {
            for v in VENUES {
                let h = host(v);
                let _ = self.invoke(web, "sites.create", json!({ "host": h, "title": v, "description": "Fictional venue" })).await;
                if self.invoke(web, "pages.publish", json!({ "url": format!("http://{h}/hours"), "title": format!("{v} — hours"), "text": venue_text(v, closed_venue == Some(*v)) })).await.is_ok() {
                    n += 1;
                }
            }
            let _ = self.invoke(web, "sites.create", json!({ "host": "swiftcab.test", "title": "SwiftCab", "description": "Fictional taxi company" })).await;
            let _ = self.invoke(web, "pages.publish", json!({ "url": CAB_URL, "title": "SwiftCab booking", "text": "SwiftCab (fictional). Standard car seats 4. Large car seats 6. Submit pickup, people and time to book.", "form": { "response_status": 200, "response_text": "Booked with SwiftCab. Reference CAB-TEST-4417." } })).await;
            let _ = self.invoke(web, "sites.create", json!({ "host": "skyhop.test", "title": "SkyHop Air", "description": "Fictional airline" })).await;
            let listing = FLIGHTS.iter().map(|f| format!("{} {} departs {} ${:.2}", f.code, f.route, f.departs, f.price_cents as f64 / 100.0)).collect::<Vec<_>>().join(". ");
            let _ = self.invoke(web, "pages.publish", json!({ "url": FLIGHTS_URL, "title": "SkyHop Air — SFO to Denver", "text": format!("SkyHop Air (fictional). Fares per person, one way. {listing}.") })).await;
            let _ = self.invoke(web, "pages.publish", json!({ "url": FLIGHT_BOOK_URL, "title": "SkyHop Air booking", "text": "Submit flight and passengers to book.", "form": { "response_status": 200, "response_text": "Booked with SkyHop Air. Reference SKY-TEST-9021." } })).await;
        }
        if let Some(cal) = &self.calendar {
            let mut ids = self.calendars.lock().await;
            for (id, name) in members {
                if let Ok(v) = self.invoke(cal, "calendars.insert", json!({ "summary": format!("{name} (fictional flatmate)") })).await {
                    if let Some(cid) = v.get("id").and_then(Value::as_str) {
                        ids.insert(id.clone(), cid.to_string());
                        n += 1;
                    }
                }
            }
            if let Some((member, nights)) = away {
                if let Some(cid) = ids.get(member) {
                    let start = chrono::Utc::now() + chrono::Duration::days(1);
                    let end = start + chrono::Duration::days(nights);
                    let _ = self.invoke(cal, "events.insert", json!({ "calendarId": cid, "summary": "Away: Denver", "start": { "dateTime": start.to_rfc3339() }, "end": { "dateTime": end.to_rfc3339() } })).await;
                }
            }
        }
        n
    }

    /// Local mode only: put up or take down a closure notice (a labelled demo control).
    pub async fn set_closed(&self, venue: &str, closed: bool) {
        let mut c = self.closed.lock().await;
        if closed { c.insert(venue.to_string()); } else { c.remove(venue); }
    }

    /// Read the venue's own page. Returns (open, notice text, url).
    pub async fn venue_status(&self, venue: &str) -> Result<(bool, String, String)> {
        let url = format!("http://{}/hours", host(venue));
        let text = match &self.web {
            Some(web) => {
                let v = self.invoke(web, "fetch", json!({ "url": url })).await?;
                let status = v.get("status").and_then(Value::as_i64).unwrap_or(0);
                if status != 200 {
                    bail!("{url} answered HTTP {status}");
                }
                v.get("content").and_then(Value::as_str).unwrap_or_default().to_string()
            }
            None => venue_text(venue, self.closed.lock().await.contains(venue)),
        };
        Ok((!text.contains("CLOSED"), text, url))
    }

    pub async fn book_cab(&self, people: i64, when: &str, pickup: &str) -> Result<String> {
        match &self.web {
            Some(web) => {
                let v = self.invoke(web, "submit", json!({ "url": CAB_URL, "fields": { "pickup": pickup, "people": people.to_string(), "time": when } })).await?;
                let status = v.get("status").and_then(Value::as_i64).unwrap_or(0);
                if status != 200 {
                    bail!("the cab site answered HTTP {status}");
                }
                Ok(v.get("content").and_then(Value::as_str).unwrap_or("Booked.").to_string())
            }
            None => Ok("Booked with SwiftCab. Reference CAB-LOCAL-SIM (SIMULATED).".into()),
        }
    }

    pub async fn flights(&self) -> Result<(Vec<Flight>, String)> {
        if let Some(web) = &self.web {
            let v = self.invoke(web, "fetch", json!({ "url": FLIGHTS_URL })).await?;
            if v.get("status").and_then(Value::as_i64) != Some(200) {
                bail!("the airline site did not answer");
            }
        }
        Ok((FLIGHTS.to_vec(), FLIGHTS_URL.to_string()))
    }

    pub async fn book_flight(&self, code: &str, passengers: i64) -> Result<String> {
        match &self.web {
            Some(web) => {
                let v = self.invoke(web, "submit", json!({ "url": FLIGHT_BOOK_URL, "fields": { "flight": code, "passengers": passengers.to_string() } })).await?;
                if v.get("status").and_then(Value::as_i64) != Some(200) {
                    bail!("the airline site did not accept the booking");
                }
                Ok(v.get("content").and_then(Value::as_str).unwrap_or("Booked.").to_string())
            }
            None => Ok("Booked with SkyHop Air. Reference SKY-LOCAL-SIM (SIMULATED).".into()),
        }
    }

    /// Synthetic mode: who has something on their calendar that day. None in local mode
    /// (the caller reads the flat's own calendar table instead).
    pub async fn busy(&self, day: chrono::NaiveDate) -> Result<Option<HashMap<String, bool>>> {
        let Some(cal) = &self.calendar else { return Ok(None) };
        let ids = self.calendars.lock().await.clone();
        if ids.is_empty() {
            bail!("no calendars are available");
        }
        let (min, max) = (format!("{day}T00:00:00Z"), format!("{day}T23:59:59Z"));
        let v = self.invoke(cal, "freebusy.query", json!({ "timeMin": min, "timeMax": max, "items": ids.values().map(|c| json!({ "id": c })).collect::<Vec<_>>() })).await?;
        Ok(Some(ids.iter().map(|(m, c)| (m.clone(), v.pointer(&format!("/calendars/{}/busy", c.replace('~', "~0").replace('/', "~1"))).and_then(Value::as_array).map_or(false, |b| !b.is_empty()))).collect()))
    }

    pub async fn send_mail(&self, to: &[String], subject: &str, body: &str) -> Result<String> {
        match &self.mail {
            Some(mail) => {
                let v = self.invoke(mail, "messages.send", json!({ "userId": "me", "to": to, "subject": subject, "body": body })).await?;
                Ok(v.get("id").and_then(Value::as_str).ok_or_else(|| anyhow!("mail service returned no id"))?.to_string())
            }
            None => Ok("msg-local-sim".into()),
        }
    }
}
