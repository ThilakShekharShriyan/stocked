//! All runtime configuration comes from env (repo-root `.env.local` is loaded first).

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    // SingleStore
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    // Superlinked SIE
    pub sie_base: String,
    pub sie_key: Option<String>,
    pub decide_model: String,
    pub embed_model: String,
    // Planner (any OpenAI-compatible endpoint; defaults to SIE)
    pub planner_base: String,
    pub planner_key: Option<String>,
    pub planner_model: String,
    // Other sponsors
    pub amplitude_key: Option<String>,
    pub sidecar_url: String,
    pub senso_key: Option<String>,
    pub statsig_key: Option<String>,
    pub persona_key: Option<String>,
    pub persona_template: Option<String>,
    pub web_url: String,
    // Firedrill injects this when it runs the agent against synthetic Stripe
    pub payments_invoke_url: Option<String>,
    // Household defaults
    pub demo_household: String,
    pub monthly_budget_cents: i64,
    pub default_auto_limit_cents: i64,
    pub default_agent_version: String,
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn first(names: &[&str]) -> Option<String> {
    names.iter().find_map(|n| var(n))
}

impl Config {
    pub fn from_env() -> Self {
        // Repo root .env.local, then core/.env if present. Missing files are fine.
        for p in ["../.env.local", ".env.local", ".env"] {
            let _ = dotenvy::from_filename(p);
        }
        let sie_key = first(&["SIE_API_KEY", "SUPERLINKED"]);
        let sie_base = var("SIE_BASE_URL").unwrap_or_else(|| "https://api.superlinked.com".into());
        Config {
            port: var("PORT").and_then(|p| p.parse().ok()).unwrap_or(8080),
            // SINGLESTORE_ADDR (an IP) overrides the hostname when local DNS cannot resolve it.
            db_host: first(&["SINGLESTORE_ADDR", "SINGLESTORE_HOST"]).unwrap_or_default(),
            db_port: var("SINGLESTORE_PORT").and_then(|p| p.parse().ok()).unwrap_or(3306),
            db_user: var("SINGLESTORE_USER").unwrap_or_else(|| "admin".into()),
            db_password: var("SINGLESTORE_PASSWORD").unwrap_or_default(),
            db_name: var("SINGLESTORE_DB").unwrap_or_else(|| "stocked".into()),
            planner_base: var("PLANNER_BASE_URL").unwrap_or_else(|| sie_base.clone()),
            planner_key: var("PLANNER_API_KEY").or_else(|| sie_key.clone()),
            planner_model: var("PLANNER_MODEL").unwrap_or_else(|| "Qwen/Qwen3.8-27B-FP8".into()),
            sie_base,
            sie_key,
            decide_model: var("SIE_DECIDE_MODEL").unwrap_or_else(|| "fastino/GLiNER2.5-Decide".into()),
            embed_model: var("SIE_EMBED_MODEL").unwrap_or_else(|| "BAAI/bge-m3".into()),
            amplitude_key: first(&["AMPLITUDE_API_KEY", "AMPLITUDE"]),
            sidecar_url: var("SIDECAR_URL").unwrap_or_else(|| "http://127.0.0.1:8090".into()),
            senso_key: first(&["SENSO_API_KEY", "SENSO"]),
            statsig_key: first(&["STATSIG_SDK_KEY", "STATSIG"]),
            persona_key: first(&["PERSONA_API_KEY", "PERSONA"]),
            persona_template: first(&["PERSONA_TEMPLATE_ID", "PERSONA_TEMPLATE"]),
            web_url: var("WEB_URL").unwrap_or_else(|| "http://localhost:3000".into()),
            payments_invoke_url: var("PAYMENTS_INVOKE_URL"),
            demo_household: var("DEMO_HOUSEHOLD").unwrap_or_else(|| "hh_demo".into()),
            monthly_budget_cents: var("MONTHLY_BUDGET_CENTS").and_then(|v| v.parse().ok()).unwrap_or(30_000),
            default_auto_limit_cents: var("AUTO_LIMIT_CENTS").and_then(|v| v.parse().ok()).unwrap_or(2_500),
            default_agent_version: var("AGENT_VERSION").unwrap_or_else(|| "v2".into()),
        }
    }
}
