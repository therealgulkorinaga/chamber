//! Chambers Level 1 API adapter.
//!
//! Stateless HTTP forwarder built with axum that wraps the Rust Runtime struct.
//! Serves a JSON API at /api/* and static UI files at /.

mod handlers;
mod state;

use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chambers_server=info,tower_http=info".into()),
        )
        .init();

    let mut runtime = Runtime::new();
    runtime
        .load_grammar(decision_chamber_grammar())
        .expect("failed to load Decision Chamber grammar");

    let shared: AppState = Arc::new(Mutex::new(runtime));

    // World-specific routes (with path param)
    let world_routes = Router::new()
        .route("/", get(handlers::get_world))
        .route("/objects", get(handlers::get_objects))
        .route("/graph", get(handlers::get_graph))
        .route("/summary", get(handlers::get_summary))
        .route("/legal-actions", get(handlers::get_legal_actions))
        .route("/convergence", get(handlers::get_convergence))
        .route("/submit", post(handlers::submit_operation))
        .route("/advance", post(handlers::advance_phase))
        .route("/burn", post(handlers::burn_world))
        .route("/residue", get(handlers::get_residue))
        .route("/burn-view", get(handlers::get_burn_view))
        .route("/audit", get(handlers::get_audit));

    // Embed UI HTML at compile time
    let ui_html: &'static str = include_str!("../../../ui/index.html");

    let app = Router::new()
        .route("/", get(move || async move { Html(ui_html) }))
        .route("/api/worlds", post(handlers::create_world))
        .route("/api/grammars", get(handlers::list_grammars))
        .route("/api/vault", get(handlers::get_vault))
        .nest("/api/worlds/:id", world_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(shared);

    let addr = "127.0.0.1:3000";
    println!();
    println!("============================================");
    println!("  Chambers Phase 0 — Level 1 API Adapter");
    println!("  http://{}", addr);
    println!("============================================");
    println!();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
