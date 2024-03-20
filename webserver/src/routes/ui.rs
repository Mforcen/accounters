use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use hyper::{header::CONTENT_TYPE, StatusCode};
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;
use accounters::models::{Account, Transaction};

pub mod account;
pub mod categories;
pub mod classifier;
pub mod rules;

pub async fn index(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
) -> impl IntoResponse {
    let mut ctx = Context::new();

    let accounts = Account::list(db.as_ref(), uid.user_id).await.unwrap();
    ctx.insert("accounts", &accounts);

    let transactions = Transaction::list_by_user(db.as_ref(), uid.user_id, 10, 0, false)
        .await
        .unwrap();
    ctx.insert("transactions", &transactions);

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
