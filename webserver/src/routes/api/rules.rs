use std::sync::Arc;

use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;

use accounters::models::rules::Rule;

#[derive(Deserialize)]
pub struct RuleCreateRequest {
    regex: String,
    category: i32,
}

pub async fn create(
    State(db): State<Arc<SqlitePool>>,
    Json(rule): Json<RuleCreateRequest>,
) -> impl IntoResponse {
    match Rule::new(db.as_ref(), rule.regex, rule.category).await {
        Ok(r) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            serde_json::to_string(&r).unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e:?}"),
        ),
    }
}

pub async fn list(State(db): State<Arc<SqlitePool>>) -> impl IntoResponse {
    match Rule::list(db.as_ref()).await {
        Ok(rule_list) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            serde_json::to_string(&rule_list).unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e:?}"),
        ),
    }
}
