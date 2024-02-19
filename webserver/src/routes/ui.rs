use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;
use accounters::models::{Account, Transaction};

pub mod rules;

pub async fn index(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
) -> impl IntoResponse {
    let mut ctx = Context::new();

    let accounts = Account::list(db.as_ref(), uid.user_id).await.unwrap();
    ctx.insert("accounts", &accounts);

    match tmpls.render("index.html", &ctx) {
        Ok(out) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/html;charset=utf-8")],
            out,
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain")],
            format!("{e}"),
        ),
    }
}

#[derive(Deserialize)]
pub struct AccountViewParams {
    movements: Option<i32>,
}

pub async fn account(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Path(account_id): Path<i32>,
    Query(AccountViewParams { movements }): Query<AccountViewParams>,
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

    let txs = match Transaction::list(
        db.as_ref(),
        account.get_id(),
        movements.unwrap_or(10),
        0,
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
    ctx.insert("n_txs", &txs.len());

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("accounts.html", &ctx).unwrap(),
    )
}
