use axum::extract::FromRef;
use axum_template::engine::Engine;
use tera::Tera;

use crate::config::Config;

/// Type alias for the Tera-backed template engine.
pub type AppEngine = Engine<Tera>;

/// Shared application state.
/// `FromRef` allows axum-template to extract `AppEngine` directly from state.
#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
}

impl FromRef<AppState> for AppEngine {
    fn from_ref(state: &AppState) -> Self {
        state.engine.clone()
    }
}

impl AppState {
    /// Build application state from the provided configuration.
    pub fn new(config: &Config) -> Self {
        let mut tera =
            Tera::new(config.template_glob).expect("Failed to parse Tera templates");

        if config.template_autoreload {
            // In dev mode, always reload templates from disk so edits are
            // picked up without restarting the server.
            tera.full_reload().expect("Failed to reload Tera templates");
        }

        AppState {
            engine: Engine::from(tera),
        }
    }
}






