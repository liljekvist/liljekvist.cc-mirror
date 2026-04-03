pub mod ws;

use axum::extract::State;
use axum::response::IntoResponse;
use axum_template::RenderHtml;

use crate::ascii_art;
use crate::models::IndexModel;
use crate::state::{AppEngine, AppState};

/// GET / — renders the home page.
pub async fn index(State(state): State<AppState>, engine: AppEngine) -> impl IntoResponse {
    let model = IndexModel {
        title: "liljekvist.cc".to_string(),
        message: String::new(),
        ascii_art: ascii_art::random(&state.ascii_styles).to_string(),
    };
    RenderHtml("index.html", engine, model)
}
