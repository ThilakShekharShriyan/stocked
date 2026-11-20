//! Statsig, evaluated in-process with the Rust SDK (no network call per decision).
//!
//! Feature gate    `stocked_model_planner`  on  -> the open LLM proposes event baskets (agent v2-model)
//!                                          off -> the per-guest rule proposes them (agent v1-rules)
//! Dynamic config  `stocked_house_rules`    { "auto_limit_cents": 2500 }
//!
//! Every plan records the variant it ran under, and plan/order events are logged back to Statsig.

use std::collections::HashMap;
use std::sync::Arc;

use statsig_rust::{Statsig, StatsigUserBuilder};

pub const GATE_MODEL_PLANNER: &str = "stocked_model_planner";
pub const GATE_CONTINGENCY: &str = "stocked_contingency";
pub const CONFIG_HOUSE_RULES: &str = "stocked_house_rules";

#[derive(Clone)]
pub struct Flags {
    inner: Option<Arc<Statsig>>,
}

impl Flags {
    pub async fn connect(key: Option<String>) -> Self {
        let Some(key) = key else { return Flags { inner: None } };
        let statsig = Statsig::new(&key, None);
        match statsig.initialize().await {
            Ok(()) => Flags { inner: Some(Arc::new(statsig)) },
            Err(err) => {
                tracing::warn!("Statsig did not initialise ({err}); using defaults");
                Flags { inner: None }
            }
        }
    }

    pub fn connected(&self) -> bool {
        self.inner.is_some()
    }

    /// A gate's value, or None when Statsig is not connected.
    pub fn gate(&self, household: &str, name: &str) -> Option<bool> {
        let s = self.inner.as_ref()?;
        Some(s.check_gate(&StatsigUserBuilder::new_with_user_id(household.to_string()).build(), name))
    }

    /// (auto-approve limit in cents, agent version), or None when Statsig is not connected.
    pub fn house_rules(&self, household: &str, default_limit: i64) -> Option<(i64, String)> {
        let s = self.inner.as_ref()?;
        let user = StatsigUserBuilder::new_with_user_id(household.to_string()).build();
        let limit = s.get_dynamic_config(&user, CONFIG_HOUSE_RULES).get_i64("auto_limit_cents", default_limit);
        let version = if s.check_gate(&user, GATE_MODEL_PLANNER) { "v2-model" } else { "v1-rules" };
        Some((limit, version.to_string()))
    }

    pub fn log(&self, household: &str, event: &str, value: Option<String>, metadata: HashMap<String, String>) {
        if let Some(s) = &self.inner {
            let user = StatsigUserBuilder::new_with_user_id(household.to_string()).build();
            s.log_event(&user, event, value, Some(metadata));
        }
    }

    pub async fn flush(&self) {
        if let Some(s) = &self.inner {
            s.flush_events().await;
        }
    }
}
