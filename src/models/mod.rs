use serde::Serialize;

/// Model for the index/home page.
#[derive(Debug, Serialize)]
pub struct IndexModel {
    pub title: String,
    pub message: String,
}

