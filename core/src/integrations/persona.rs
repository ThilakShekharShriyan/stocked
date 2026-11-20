//! Persona: verify the human behind an agent's action.
//! An inquiry is created server-side; the person completes Persona's hosted flow; we poll the result.
//! Sandbox keys (`persona_sandbox_...`) return simulated results.

use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

const BASE: &str = "https://api.withpersona.com/api/v1";

#[derive(Clone)]
pub struct Persona {
    http: reqwest::Client,
    key: Option<String>,
    template: Option<String>,
}

impl Persona {
    pub fn new(key: Option<String>, template: Option<String>) -> Self {
        Persona { http: reqwest::Client::builder().timeout(Duration::from_secs(15)).build().unwrap(), key, template }
    }

    pub fn configured(&self) -> bool {
        self.key.is_some() && self.template.is_some()
    }

    /// A key without an inquiry template is used for Relay claims ("is a live human"), which the
    /// dashboard's server runs through Persona's Node SDK.
    pub fn relay(&self) -> bool {
        self.key.is_some() && self.template.is_none()
    }

    pub fn key_matches(&self, candidate: &str) -> bool {
        self.key.as_deref() == Some(candidate)
    }

    pub fn sandbox(&self) -> bool {
        self.key.as_deref().map_or(false, |k| k.starts_with("persona_sandbox"))
    }

    /// Returns (inquiry id, hosted-flow URL).
    pub async fn create_inquiry(&self, reference_id: &str, redirect: &str) -> Result<(String, String)> {
        let (key, template) = (self.key.as_ref().ok_or_else(|| anyhow!("Persona not configured"))?, self.template.as_ref().ok_or_else(|| anyhow!("Persona template not configured"))?);
        let resp = self
            .http
            .post(format!("{BASE}/inquiries"))
            .bearer_auth(key)
            .json(&json!({
                "data": { "attributes": { "inquiry-template-id": template } },
                "meta": { "auto-create-account-reference-id": reference_id }
            }))
            .send()
            .await?;
        let status = resp.status();
        let v: Value = resp.json().await.unwrap_or(json!({}));
        if !status.is_success() {
            bail!("Persona HTTP {status}: {}", v.to_string().chars().take(200).collect::<String>());
        }
        let id = v.pointer("/data/id").and_then(Value::as_str).ok_or_else(|| anyhow!("Persona returned no inquiry id"))?.to_string();
        let mut url = reqwest::Url::parse("https://inquiry.withpersona.com/verify")?;
        url.query_pairs_mut().append_pair("inquiry-id", &id).append_pair("redirect-uri", redirect);
        Ok((id, url.to_string()))
    }

    /// Inquiry status as Persona reports it (created, pending, completed, approved, declined, failed, expired).
    pub async fn inquiry_status(&self, inquiry_id: &str) -> Result<String> {
        let key = self.key.as_ref().ok_or_else(|| anyhow!("Persona not configured"))?;
        let v: Value = self.http.get(format!("{BASE}/inquiries/{inquiry_id}")).bearer_auth(key).send().await?.json().await?;
        Ok(v.pointer("/data/attributes/status").and_then(Value::as_str).unwrap_or("unknown").to_string())
    }
}
