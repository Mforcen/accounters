use std::sync::Arc;

use axum::extract::{Json, State};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::users::UserToken;
use accounters::models::rules::Rule;

#[derive(Deserialize)]
pub struct RuleCreateRequest {
    regex: String,
    category: i32,
}

pub async fn create(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Json(rule): Json<RuleCreateRequest>,
) -> (StatusCode, String) {
    match Rule::new(db.as_ref(), uid.user_id, rule.regex, rule.category).await {
        Ok(r) => (StatusCode::OK, serde_json::to_string(&r).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}

pub async fn list(State(db): State<Arc<SqlitePool>>, uid: UserToken) -> (StatusCode, String) {
    match Rule::list_by_user(db.as_ref(), uid.user_id).await {
        Ok(rule_list) => (StatusCode::OK, serde_json::to_string(&rule_list).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}
