//! Senso context layer (thin): cite the store policy behind an online-vs-pickup decision.
//! Disabled unless SENSO_API_KEY is set; callers treat `None` as "no citation available".

use std::time::{Duration, Instant};

use serde_json::{json, Value};

#[derive(Clone)]
pub struct Senso {
    http: reqwest::Client,
    key: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Citation {
    pub chunk_text: String,
    pub content_id: String,
    pub kb_node_id: String,
    pub version_id: String,
    pub score: f64,
    pub rank: i64,
    pub latency_us: i64,
}

impl Senso {
    pub fn new(key: Option<String>) -> Self {
        Senso { http: reqwest::Client::builder().timeout(Duration::from_secs(10)).build().unwrap(), key }
    }

    pub async fn cite(&self, query: &str) -> Option<Citation> {
        let key = self.key.as_ref()?;
        let started = Instant::now();
        let v: Value = self
            .http
            .post("https://apiv2.senso.ai/api/v1/org/search/context")
            .header("X-API-Key", key)
            .json(&json!({ "query": query, "max_results": 1, "search_type": "hybrid" }))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;
        let r = v.get("results").and_then(Value::as_array).and_then(|a| a.first())?;
        let s = |k: &str| r.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
        Some(Citation {
            chunk_text: s("chunk_text"),
            content_id: s("content_id"),
            kb_node_id: s("kb_node_id"),
            version_id: s("version_id"),
            score: r.get("score").and_then(Value::as_f64).unwrap_or(0.0),
            rank: r.get("rank").and_then(Value::as_i64).unwrap_or(0),
            latency_us: started.elapsed().as_micros() as i64,
        })
    }
}
