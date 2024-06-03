use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;

use accounters::models::categories::Category;

#[derive(Deserialize)]
pub struct CategoryCreateRequest {
    name: String,
    description: String,
}

pub async fn create(
    State(db): State<Arc<SqlitePool>>,
    Json(new_category): Json<CategoryCreateRequest>,
) -> impl IntoResponse {
    match Category::new(db.as_ref(), &new_category.name, &new_category.description).await {
        Ok(_) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}

pub async fn list(State(db): State<Arc<SqlitePool>>) -> impl IntoResponse {
    match Category::list(db.as_ref()).await {
        Ok(c) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            serde_json::to_string(&c).unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e:?}"),
        ),
    }
}
