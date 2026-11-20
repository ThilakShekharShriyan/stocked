//! Payments are always test-mode. Two providers:
//! - Firedrill synthetic Stripe, when Firedrill injects `PAYMENTS_INVOKE_URL` during a drill;
//! - a local mock otherwise (deterministic per idempotency key).

use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct Payments {
    http: reqwest::Client,
    invoke_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Charge {
    pub provider: String,
    pub reference: String,
    pub latency_us: i64,
    pub raw: Value,
}

impl Payments {
    pub fn new(invoke_url: Option<String>) -> Self {
        Payments { http: reqwest::Client::builder().timeout(Duration::from_secs(20)).build().unwrap(), invoke_url }
    }

    pub fn provider_name(&self) -> &'static str {
        if self.invoke_url.is_some() { "firedrill_synthetic_stripe" } else { "mock_test_mode" }
    }

    pub async fn charge(&self, amount_cents: i64, idempotency_key: &str, description: &str) -> Result<Charge> {
        let started = Instant::now();
        match &self.invoke_url {
            Some(url) => {
                let body = json!({
                    "operation": "payment_intents.create",
                    "arguments": {
                        "amount": amount_cents,
                        "currency": "usd",
                        "description": description,
                        // Confirm at once with Stripe's test card, so the charge is captured and refundable.
                        "payment_method": "pm_card_visa",
                        "confirm": true,
                        "idempotency_key": idempotency_key
                    }
                });
                let resp = self.http.post(url).header("Idempotency-Key", idempotency_key).json(&body).send().await?;
                let status = resp.status();
                let raw: Value = resp.json().await.unwrap_or(json!({}));
                if !status.is_success() {
                    bail!("payments unavailable (HTTP {status})");
                }
                let reference = ["/id", "/result/id", "/data/id", "/output/id"]
                    .iter()
                    .find_map(|p| raw.pointer(p).and_then(Value::as_str))
                    .unwrap_or("pi_synthetic")
                    .to_string();
                Ok(Charge { provider: self.provider_name().into(), reference, latency_us: started.elapsed().as_micros() as i64, raw })
            }
            None => {
                let short: String = idempotency_key.chars().filter(|c| c.is_ascii_alphanumeric()).take(16).collect();
                Ok(Charge {
                    provider: self.provider_name().into(),
                    reference: format!("pi_test_{short}"),
                    latency_us: started.elapsed().as_micros() as i64,
                    raw: json!({ "test_mode": true }),
                })
            }
        }
    }

    /// Refund part of an earlier charge. Test mode only, like every charge.
    pub async fn refund(&self, payment_ref: &str, amount_cents: i64, idempotency_key: &str) -> Result<Charge> {
        let started = Instant::now();
        match &self.invoke_url {
            Some(url) => {
                let body = json!({ "operation": "refunds.create", "arguments": { "payment_intent": payment_ref, "amount": amount_cents, "idempotency_key": idempotency_key } });
                let resp = self.http.post(url).header("Idempotency-Key", idempotency_key).json(&body).send().await?;
                let status = resp.status();
                let raw: Value = resp.json().await.unwrap_or(json!({}));
                if !status.is_success() {
                    bail!("refund unavailable (HTTP {status}): {}", raw.to_string().chars().take(160).collect::<String>());
                }
                let reference = ["/id", "/result/id", "/data/id", "/output/id"].iter().find_map(|p| raw.pointer(p).and_then(Value::as_str)).unwrap_or("re_synthetic").to_string();
                Ok(Charge { provider: self.provider_name().into(), reference, latency_us: started.elapsed().as_micros() as i64, raw })
            }
            None => {
                let short: String = idempotency_key.chars().filter(|c| c.is_ascii_alphanumeric()).rev().take(12).collect();
                Ok(Charge { provider: self.provider_name().into(), reference: format!("re_test_{short}"), latency_us: started.elapsed().as_micros() as i64, raw: json!({ "test_mode": true }) })
            }
        }
    }
}
