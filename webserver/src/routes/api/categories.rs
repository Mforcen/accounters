use std::sync::Arc;

use axum::{extract::State, Json};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::users::UserToken;
use accounters::models::categories::Category;

#[derive(Deserialize)]
pub struct CategoryCreateRequest {
    name: String,
    description: String,
}

pub async fn create(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Json(new_category): Json<CategoryCreateRequest>,
) -> (StatusCode, String) {
    match Category::new(db.as_ref(), &new_category.name, &new_category.description).await {
        Ok(_) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}

pub async fn list(State(db): State<Arc<SqlitePool>>, uid: UserToken) -> (StatusCode, String) {
    match Category::list(db.as_ref()).await {
        Ok(c) => (StatusCode::OK, serde_json::to_string(&c).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}
