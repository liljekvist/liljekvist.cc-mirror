use axum::extract::FromRef;
use axum_template::engine::Engine;
use tera::Tera;

use crate::config::Config;

pub type AppEngine = Engine<Tera>;

#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
    pub ascii_styles: Vec<String>,
    pub secret: &'static str,
}

impl FromRef<AppState> for AppEngine {
    fn from_ref(state: &AppState) -> Self {
        state.engine.clone()
    }
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        let mut tera =
            Tera::new(config.template_glob).expect("Failed to parse Tera templates");

        if config.template_autoreload {
            tera.full_reload().expect("Failed to reload Tera templates");
        }

        let ascii_styles = crate::ascii_art::load(std::path::Path::new(config.ascii_art_dir));

        AppState {
            engine: Engine::from(tera),
            ascii_styles,
            secret: config.secret,
        }
    }
}






