//! Client for the Python sidecar (Airbyte MCP, Statsig, SIE extract/rerank).
//! Every call degrades gracefully: if the sidecar is down, the core keeps working on defaults.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::config::Config;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HouseConfig {
    pub auto_limit_cents: i64,
    pub agent_version: String,
    /// where the values came from: "statsig" | "default"
    pub source: String,
}

#[derive(Clone)]
pub struct Sidecar {
    http: reqwest::Client,
    url: String,
    defaults: HouseConfig,
    cache: Arc<RwLock<Option<(Instant, HouseConfig)>>>,
    flags: crate::integrations::statsig::Flags,
}

impl Sidecar {
    pub fn new(cfg: &Config, flags: crate::integrations::statsig::Flags) -> Self {
        Sidecar {
            http: reqwest::Client::builder().timeout(Duration::from_secs(120)).build().unwrap(),
            url: cfg.sidecar_url.trim_end_matches('/').to_string(),
            defaults: HouseConfig {
                auto_limit_cents: cfg.default_auto_limit_cents,
                agent_version: cfg.default_agent_version.clone(),
                source: "default".into(),
            },
            cache: Arc::new(RwLock::new(None)),
            flags,
        }
    }

    /// Should the agent switch to a stored backup by itself when a plan falls through?
    /// A simulation round pins it (on from v2); otherwise the Statsig gate decides; on by default.
    pub fn contingency(&self, user: &str) -> bool {
        if let Ok(v) = std::env::var("AGENT_VARIANT") {
            if !v.trim().is_empty() {
                return !matches!(v.trim(), "v0" | "v1");
            }
        }
        self.flags.gate(user, crate::integrations::statsig::GATE_CONTINGENCY).unwrap_or(true)
    }

    /// Statsig-backed household config, cached for 15 s. `user` is the household id.
    pub async fn config(&self, user: &str) -> HouseConfig {
        // A simulation run pins its variant so every session in a round runs the same agent.
        if let Ok(v) = std::env::var("AGENT_VARIANT") {
            if !v.trim().is_empty() {
                let limit = self.flags.house_rules(user, self.defaults.auto_limit_cents).map(|(l, _)| l).unwrap_or(self.defaults.auto_limit_cents);
                return HouseConfig { auto_limit_cents: limit, agent_version: v.trim().to_string(), source: "round-override".into() };
            }
        }
        // Statsig evaluated in-process (Rust SDK): no cache or network hop needed.
        if let Some((auto_limit_cents, agent_version)) = self.flags.house_rules(user, self.defaults.auto_limit_cents) {
            return HouseConfig { auto_limit_cents, agent_version, source: "statsig".into() };
        }
        if let Some((at, c)) = self.cache.read().await.clone() {
            if at.elapsed() < Duration::from_secs(15) {
                return c;
            }
        }
        let fetched = async {
            let v: Value = self
                .http
                .get(format!("{}/config", self.url))
                .query(&[("user", user)])
                .timeout(Duration::from_secs(3))
                .send()
                .await
                .ok()?
                .json()
                .await
                .ok()?;
            Some(HouseConfig {
                auto_limit_cents: v.get("auto_limit_cents")?.as_i64()?,
                agent_version: v.get("agent_version")?.as_str()?.to_string(),
                source: v.get("source").and_then(Value::as_str).unwrap_or("statsig").to_string(),
            })
        }
        .await;
        let c = fetched.unwrap_or_else(|| self.defaults.clone());
        *self.cache.write().await = Some((Instant::now(), c.clone()));
        c
    }

    pub async fn post(&self, path: &str, body: Value) -> anyhow::Result<Value> {
        let resp = self.http.post(format!("{}{}", self.url, path)).json(&body).send().await?;
        let status = resp.status();
        let v: Value = resp.json().await.unwrap_or(json!({}));
        if !status.is_success() {
            anyhow::bail!("sidecar {path} HTTP {status}: {v}");
        }
        Ok(v)
    }

    pub async fn health(&self) -> Value {
        match self.http.get(format!("{}/health", self.url)).timeout(Duration::from_secs(2)).send().await {
            Ok(r) => r.json().await.unwrap_or(json!({ "ok": false })),
            Err(_) => json!({ "ok": false, "error": "sidecar not running" }),
        }
    }
}
