use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use chrono::{offset::Utc, DateTime};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

use accounters::models::transaction::{Transaction, TxConflictResolutionMode};

#[derive(Deserialize)]
pub struct TransactionContent {
    description: String,
    timestamp: DateTime<Utc>,
    category: Option<String>,
    amount: i32,
    #[serde(default)]
    error_on_conflict: Option<bool>,
}

pub async fn create(
    State(db): State<Arc<SqlitePool>>,
    Path(account): Path<i32>,
    Json(txcnt): Json<TransactionContent>,
) -> (StatusCode, String) {
    let error_on_conflict = if txcnt.error_on_conflict.is_some_and(|x| x) {
        TxConflictResolutionMode::Error
    } else {
        TxConflictResolutionMode::Nothing
    };
    match Transaction::new(
        db.as_ref(),
        account,
        &txcnt.description,
        &txcnt.timestamp,
        None,
        txcnt.amount,
        error_on_conflict,
    )
    .await
    {
        Ok(tx) => (StatusCode::OK, serde_json::to_string(&tx).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}

#[derive(Deserialize)]
pub struct PaginationOptions {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn list(
    State(db): State<Arc<SqlitePool>>,
    Path(account): Path<i32>,
    Query(pagination): Query<PaginationOptions>,
) -> (StatusCode, String) {
    match Transaction::list(
        db.as_ref(),
        account,
        pagination.limit.unwrap_or(100),
        pagination.offset.unwrap_or(0),
        true,
    )
    .await
    {
        Ok(txs) => (StatusCode::OK, serde_json::to_string(&txs).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}
