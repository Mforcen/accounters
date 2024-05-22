use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{Date, DateTime, Duration, DurationRound, TimeZone, Utc};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;
use accounters::models::{account::Account, categories::Category, transaction::Transaction};

#[derive(Deserialize)]
pub struct AccountViewParams {
    from: Option<String>,
    to: Option<String>,
}

fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let mut iter = s.split('-');
    let year = iter.next()?.parse::<i32>().ok()?;
    let month = iter.next()?.parse::<u32>().ok()?;
    let day = iter.next()?.parse::<u32>().ok()?;
    Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).single()
}

pub async fn show(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Path(account_id): Path<i32>,
    Query(AccountViewParams { from, to }): Query<AccountViewParams>,
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

    let from = from
        .and_then(|x| parse_date(&x))
        .unwrap_or(Utc::now().duration_trunc(Duration::days(1)).unwrap() - Duration::days(30));
    let to = to
        .and_then(|x| parse_date(&x))
        .unwrap_or(Utc::now().duration_trunc(Duration::days(1)).unwrap());

    ctx.insert("date_from", &from);
    ctx.insert("date_to", &to);

    let tx_agg = Transaction::group_by_date(db.as_ref(), account_id, Some(from), Some(to), false)
        .await
        .unwrap();

    ctx.insert("tx_agg", &tx_agg);

    let categories: HashMap<i32, String> = Category::list(db.as_ref())
        .await
        .unwrap()
        .iter()
        .map(|x| (x.category_id, x.name.clone()))
        .collect();
    ctx.insert("categories", &categories);

    let txs = match Transaction::list(db.as_ref(), account.get_id(), 10, 0, false).await {
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
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("account_summary.html", &ctx).unwrap(),
    )
}

#[derive(Deserialize)]
pub struct AccountTxListParams {
    entries: Option<i32>,
    page: Option<i32>,
}

pub async fn list_transactions(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Path(account_id): Path<i32>,
    Query(AccountTxListParams { entries, page }): Query<AccountTxListParams>,
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
        tmpls.render("account_txs.html", &ctx).unwrap(),
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
        )
        .await
        {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}"));
        }
    }
    (StatusCode::OK, String::new())
}
