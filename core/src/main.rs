//! Stocked core: the household agent. Rust owns every rule and the hot path;
//! models only propose. See README.md for the architecture.

mod agent;
mod checks;
mod config;
mod db;
mod estimate;
mod events;
mod ingest;
mod integrations;
mod outings;
mod payments;
mod plans;
mod routes;
mod seed;
mod sourcing;
mod state;
mod verify;
mod world;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into())).init();
    let cfg = config::Config::from_env();

    let db = db::connect(&cfg).await?;
    db::apply_schema(&db).await?;
    if seed::seed_products(&db).await? {
        tracing::info!("seeded store catalog");
    }
    if !seed::household_exists(&db, &cfg.demo_household).await? {
        seed::seed_household(&db, &cfg.demo_household).await?;
        tracing::info!("seeded demo household {}", cfg.demo_household);
    }

    let flags = integrations::statsig::Flags::connect(cfg.statsig_key.clone()).await;
    tracing::info!("statsig: {}", if flags.connected() { "connected" } else { "not connected (defaults)" });
    let (changes, _) = broadcast::channel(64);
    let st = AppState {
        sie: integrations::sie::Sie::new(&cfg),
        sidecar: integrations::sidecar::Sidecar::new(&cfg, flags.clone()),
        senso: integrations::senso::Senso::new(cfg.senso_key.clone()),
        persona: integrations::persona::Persona::new(cfg.persona_key.clone(), cfg.persona_template.clone()),
        payments: payments::Payments::new(cfg.payments_invoke_url.clone()),
        world: world::World::from_env(),
        db: db.clone(),
        changes,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        order_lock: Arc::new(Mutex::new(())),
        seed_lock: Arc::new(Mutex::new(())),
        cfg: cfg.clone(),
    };
    integrations::amplitude::spawn_forwarder(db.clone(), cfg.amplitude_key.clone(), flags.clone());
    let closing = (st.clone(), flags);

    let app = routes::api::router(st).layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.port)).await?;
    tracing::info!("stocked-core listening on :{} (payments: {})", cfg.port, if cfg.payments_invoke_url.is_some() { "firedrill synthetic stripe" } else { "mock test mode" });
    axum::serve(listener, app).with_graceful_shutdown(shutdown()).await?;

    // On the way out (a drill process is stopped after each simulated session), close every open
    // conversation so analytics sees a finished session, then send what is still queued.
    let (st, flags) = closing;
    let open: Vec<String> = st.sessions.lock().await.keys().cloned().collect();
    for session in open {
        let hh = agent::chat::household_for_session(&session);
        let version = st.sidecar.config(&hh).await.agent_version;
        let _ = events::record(&st.db, &hh, events::Event::new("session_end", "session", &session, "Conversation ended").data(serde_json::json!({ "session": session, "turn": 0, "trace": "", "agent_version": version }))).await;
    }
    let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(4)).build()?;
    integrations::amplitude::flush_once(&st.db, cfg.amplitude_key.as_ref(), &flags, &http).await;
    Ok(())
}

async fn shutdown() {
    let ctrl_c = async { let _ = tokio::signal::ctrl_c().await; };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
}
