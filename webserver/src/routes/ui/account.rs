use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;
use accounters::models::{
    account::Account,
    transaction::{Transaction, TxConflictResolutionMode},
};

#[derive(Deserialize)]
pub struct AccountViewParams {
    entries: Option<i32>,
    page: Option<i32>,
}

pub async fn list(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Path(account_id): Path<i32>,
    Query(AccountViewParams { entries, page }): Query<AccountViewParams>,
) -> impl IntoResponse {
    let mut ctx = Context::new();

    let account = match Account::get_by_id(db.as_ref(), account_id).await {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, "text/plain")],
                format!("{e}"),
            );
        }
    };

    if account.get_user() != uid.user_id {
        return (
            StatusCode::UNAUTHORIZED,
            [(CONTENT_TYPE, "text/plain")],
            String::from("You cannot access this resource"),
        );
    }

    let n_entries = entries.unwrap_or(10).max(10);
    let page = page.unwrap_or(0).max(0);

    let txs = match Transaction::list(
        db.as_ref(),
        account.get_id(),
        n_entries,
        n_entries * page,
        false,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, "text/plain")],
                format!("Error at loading transactions: {e}"),
            );
        }
    };

    ctx.insert("account", &account);
    ctx.insert("transactions", &txs);
    ctx.insert("prev_page", &((page - 1).max(0)));
    ctx.insert("curr_page", &page);
    ctx.insert("next_page", &(page + 1));
    ctx.insert("n_entries", &(n_entries));

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("accounts.html", &ctx).unwrap(),
    )
}

pub async fn add_transactions_view(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Path(account_id): Path<i32>,
) -> impl IntoResponse {
    let mut ctxt = Context::new();
    ctxt.insert("account_id", &account_id);

    let account = match Account::get_by_id(db.as_ref(), account_id).await {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, "text/plain")],
                format!("{e:?}"),
            );
        }
    };

    if account.get_user() != uid.user_id {
        return (
            StatusCode::UNAUTHORIZED,
            [(CONTENT_TYPE, "text/plain")],
            String::from("You cannot access this resource"),
        );
    }

    ctxt.insert("account", &account);

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("accounts_add_txs.html", &ctxt).unwrap(),
    )
}

#[derive(Deserialize, Debug)]
pub struct CreateTransactionRequest {
    date: DateTime<Utc>,
    description: String,
    amount: f32,
}

pub async fn add_transactions_action(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Path(account_id): Path<i32>,
    Json(body): Json<Vec<CreateTransactionRequest>>,
) -> impl IntoResponse {
    // TODO missing user id check
    for tx in body.iter() {
        if let Err(e) = Transaction::new(
            db.as_ref(),
            account_id,
            &tx.description,
            &tx.date,
            None,
            (tx.amount * 100.0).round() as i32,
            TxConflictResolutionMode::Nothing,
        )
        .await
        {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}"));
        }
    }
    (StatusCode::OK, String::new())
}
