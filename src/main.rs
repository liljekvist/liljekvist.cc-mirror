mod commands;
mod config;
mod handlers;
mod models;
mod state;

use axum::{Router, routing::get};
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let cfg = config::get();

    if cfg.debug_logging {
        println!("[dev]  config: {cfg:?}");
    }

    let state = AppState::new(&cfg);

    let app: Router<()> = Router::new()
        .route("/", get(handlers::index))
        .route("/ws", get(handlers::ws::ws_handler))
        .with_state(state);

    let listener = TcpListener::bind(cfg.bind_addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {e}", cfg.bind_addr));

    println!("Listening on http://{}", cfg.bind_addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
