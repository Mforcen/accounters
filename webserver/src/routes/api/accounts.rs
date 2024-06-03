use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;

use accounters::models::account::Account;

pub async fn account_get(
    State(db): State<Arc<SqlitePool>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match Account::get_by_id(db.as_ref(), id).await {
        Ok(a) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            serde_json::to_string(&a).unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "plain/text")],
            format!("{e}"),
        ),
    }
}

#[derive(Deserialize)]
pub struct AccountRequestCreate {
    pub name: String,
}

pub async fn account_create(
    State(db): State<Arc<SqlitePool>>,
    Json(account): Json<AccountRequestCreate>,
) -> impl IntoResponse {
    match Account::new(db.as_ref(), &account.name).await {
        Ok(a) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            serde_json::to_string(&a).unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e}"),
        ),
    }
}

pub async fn account_list(State(db): State<Arc<SqlitePool>>) -> impl IntoResponse {
    match Account::list(db.as_ref()).await {
        Ok(a) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            serde_json::to_string(&a).unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e}"),
        ),
    }
}

pub async fn recategorize(
    State(db): State<Arc<SqlitePool>>,
    Path(account): Path<i32>,
) -> impl IntoResponse {
    let account = Account::get_by_id(db.as_ref(), account).await.unwrap();

    match account.recategorize_transactions(db.as_ref()).await {
        Ok(_) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/plain")],
            String::new(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e}"),
        ),
    }
}
