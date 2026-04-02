pub mod ws;

use axum::response::IntoResponse;
use axum_template::RenderHtml;

use crate::models::IndexModel;
use crate::state::AppEngine;

/// GET / — renders the home page.
pub async fn index(engine: AppEngine) -> impl IntoResponse {
    let model = IndexModel {
        title: "liljekvist.cc".to_string(),
        message: String::new(),
    };
    RenderHtml("index.html", engine, model)
}
