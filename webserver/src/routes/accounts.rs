use std::sync::Arc;

use axum::extract::{Json, Path, State};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::users::UserToken;
use accounters::models::Account;

pub async fn account_get(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Path(id): Path<i32>,
) -> (StatusCode, String) {
    match Account::get_by_id(db.as_ref(), id).await {
        Ok(a) => {
            if a.get_user() == uid.user_id {
                (StatusCode::OK, serde_json::to_string(&a).unwrap())
            } else {
                (StatusCode::UNAUTHORIZED, String::new())
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}

#[derive(Deserialize)]
pub struct AccountRequestCreate {
    pub name: String,
}

pub async fn account_create(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Json(account): Json<AccountRequestCreate>,
) -> (StatusCode, String) {
    match Account::new(db.as_ref(), uid.user_id, &account.name).await {
        Ok(a) => (StatusCode::OK, serde_json::to_string(&a).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}

pub async fn account_list(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
) -> (StatusCode, String) {
    match Account::list(db.as_ref(), uid.user_id).await {
        Ok(a) => (StatusCode::OK, serde_json::to_string(&a).unwrap()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}

pub async fn snapshot_update(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Path(account): Path<i32>,
) -> (StatusCode, String) {
    let account = Account::get_by_id(db.as_ref(), account).await.unwrap();
    if account.get_user() != uid.user_id {
        return (StatusCode::UNAUTHORIZED, String::new());
    }

    match account.recalculate_snapshots(db.as_ref(), None).await {
        Ok(_) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}

pub async fn recategorize(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Path(account): Path<i32>,
) -> (StatusCode, String) {
    let account = Account::get_by_id(db.as_ref(), account).await.unwrap();
    if account.get_user() != uid.user_id {
        return (StatusCode::UNAUTHORIZED, String::new());
    }

    match account
        .recategorize_transactions(db.as_ref(), None, None)
        .await
    {
        Ok(_) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")),
    }
}
