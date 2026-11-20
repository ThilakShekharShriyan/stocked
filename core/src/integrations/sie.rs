//! Superlinked SIE Serverless over plain HTTP: planner chat, the decision gate, embeddings.
//! Cold models answer `503 PROVISIONING` with `Retry-After`; we wait up to a per-call budget.

use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

use crate::config::Config;

#[derive(Clone)]
pub struct Sie {
    http: reqwest::Client,
    base: String,
    key: Option<String>,
    pub planner_base: String,
    pub planner_key: Option<String>,
    pub planner_model: String,
    pub decide_model: String,
    pub embed_model: String,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub body: Value,
    pub latency_us: i64,
    pub credits: f64,
    pub model: String,
}

impl Sie {
    pub fn new(cfg: &Config) -> Self {
        Sie {
            http: reqwest::Client::builder().timeout(Duration::from_secs(90)).build().unwrap(),
            base: cfg.sie_base.trim_end_matches('/').to_string(),
            key: cfg.sie_key.clone(),
            planner_base: cfg.planner_base.trim_end_matches('/').to_string(),
            planner_key: cfg.planner_key.clone(),
            planner_model: cfg.planner_model.clone(),
            decide_model: cfg.decide_model.clone(),
            embed_model: cfg.embed_model.clone(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.key.is_some()
    }

    async fn post(&self, base: &str, key: &Option<String>, path: &str, body: &Value, model: &str, max_wait: Duration) -> Result<Call> {
        let key = key.as_ref().ok_or_else(|| anyhow!("no API key configured"))?;
        let started = Instant::now();
        loop {
            let resp = self
                .http
                .post(format!("{base}{path}"))
                .bearer_auth(key)
                .header("Accept", "application/json")
                .json(body)
                .timeout(max_wait.saturating_sub(started.elapsed()).max(Duration::from_secs(1)))
                .send()
                .await?;
            let status = resp.status();
            if status.as_u16() == 503 || status.as_u16() == 429 {
                let wait = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(5)
                    .clamp(1, 30);
                if started.elapsed() + Duration::from_secs(wait) > max_wait {
                    bail!("{model} is still starting up (HTTP {status})");
                }
                tokio::time::sleep(Duration::from_secs(wait)).await;
                continue;
            }
            let text = resp.text().await?;
            if !status.is_success() {
                bail!("{model} HTTP {status}: {}", text.chars().take(300).collect::<String>());
            }
            let body: Value = serde_json::from_str(&text).unwrap_or(json!({ "raw": text }));
            let credits = body
                .pointer("/usage/credits_charged")
                .and_then(Value::as_f64)
                .or_else(|| body.pointer("/usage/credits_charged_decimal").and_then(|v| v.as_str()?.parse().ok()))
                .unwrap_or(0.0);
            return Ok(Call { body, latency_us: started.elapsed().as_micros() as i64, credits, model: model.to_string() });
        }
    }

    /// OpenAI-compatible chat completion (planner). `tools` may be Value::Null.
    pub async fn chat(&self, messages: &[Value], tools: Value, json_mode: bool, max_wait: Duration) -> Result<Call> {
        let mut body = json!({ "model": self.planner_model, "messages": messages, "temperature": 0.2, "max_tokens": 1200 });
        if !tools.is_null() {
            body["tools"] = tools;
        }
        if json_mode {
            body["response_format"] = json!({ "type": "json_object" });
        }
        let model = self.planner_model.clone();
        self.post(&self.planner_base, &self.planner_key, "/v1/chat/completions", &body, &model, max_wait).await
    }

    /// Yes/no gate via GLiNER2.5-Decide. Returns (probability_yes, call).
    pub async fn decide(&self, input: &str, name: &str, instructions: &str, max_wait: Duration) -> Result<(f64, Call)> {
        let body = json!({
            "model": self.decide_model,
            "input": input,
            "questions": [{ "type": "predicate", "name": name, "instructions": instructions }]
        });
        let model = self.decide_model.clone();
        let call = self.post(&self.base, &self.key, "/v1/decisions", &body, &model, max_wait).await?;
        let p = find_probability(&call.body, name).ok_or_else(|| anyhow!("no probability in decision response: {}", call.body))?;
        Ok((p, call))
    }

    /// Pick one category via GLiNER2.5-Decide. Returns (chosen value, its probability, call).
    /// The model separates categories far better than it answers a yes/no question.
    pub async fn classify(&self, input: &str, name: &str, instructions: &str, choices: &[&str], max_wait: Duration) -> Result<(String, f64, Call)> {
        let body = json!({
            "model": self.decide_model,
            "input": input,
            "questions": [{ "type": "choice", "name": name, "instructions": instructions, "choices": choices.iter().map(|c| json!({ "value": c })).collect::<Vec<_>>() }]
        });
        let model = self.decide_model.clone();
        let call = self.post(&self.base, &self.key, "/v1/decisions", &body, &model, max_wait).await?;
        let answer = call.body.pointer("/answers/0").ok_or_else(|| anyhow!("no answer in decision response"))?;
        let choice = answer.get("choice").and_then(Value::as_str).ok_or_else(|| anyhow!("no choice in decision response"))?.to_string();
        let p = answer
            .get("probabilities")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find(|x| x.get("value").and_then(Value::as_str) == Some(choice.as_str())))
            .and_then(|x| x.get("probability").and_then(Value::as_f64))
            .unwrap_or(0.0);
        Ok((choice, p, call))
    }

    pub async fn embed(&self, texts: &[String], max_wait: Duration) -> Result<(Vec<Vec<f32>>, Call)> {
        let body = json!({ "model": self.embed_model, "input": texts });
        let model = self.embed_model.clone();
        let call = self.post(&self.base, &self.key, "/v1/embeddings", &body, &model, max_wait).await?;
        let data = call.body.get("data").and_then(Value::as_array).ok_or_else(|| anyhow!("no data in embeddings response"))?;
        let vecs = data
            .iter()
            .map(|d| d.get("embedding").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect()).unwrap_or_default())
            .collect();
        Ok((vecs, call))
    }
}

/// Decision responses are searched defensively: `answers[]` entries with a matching name,
/// or any `probability` field.
fn find_probability(v: &Value, name: &str) -> Option<f64> {
    if let Some(arr) = v.get("answers").and_then(Value::as_array) {
        for a in arr {
            let matches = a.get("name").and_then(Value::as_str).map_or(true, |n| n == name);
            if matches {
                if let Some(p) = a.get("probability").and_then(Value::as_f64) {
                    return Some(p);
                }
                if let Some(p) = a.pointer("/probabilities/true").and_then(Value::as_f64) {
                    return Some(p);
                }
                if let Some(c) = a.get("choice").and_then(Value::as_str) {
                    return Some(if c.eq_ignore_ascii_case("true") || c.eq_ignore_ascii_case("yes") { 1.0 } else { 0.0 });
                }
            }
        }
    }
    match v {
        Value::Object(m) => m.values().find_map(|x| if let Some(p) = x.get("probability").and_then(Value::as_f64) { Some(p) } else { find_probability(x, name) }),
        Value::Array(a) => a.iter().find_map(|x| find_probability(x, name)),
        _ => None,
    }
}

/// Pull the assistant message out of a chat completion.
pub fn message(call: &Call) -> Value {
    call.body.pointer("/choices/0/message").cloned().unwrap_or(json!({ "role": "assistant", "content": "" }))
}

/// Parse a JSON object from model text (tolerates ```json fences and <think> blocks).
pub fn parse_json_text(text: &str) -> Option<Value> {
    let t = match text.rfind("</think>") {
        Some(i) => &text[i + 8..],
        None => text,
    };
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    serde_json::from_str(&t[start..=end]).ok()
}
