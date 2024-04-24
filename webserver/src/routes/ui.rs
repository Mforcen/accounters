use std::{borrow::BorrowMut, collections::HashMap, sync::Arc};

use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Serialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;
use accounters::models::{account::Account, categories::Category, transaction::Transaction};

pub mod account;
pub mod categories;
pub mod classifier;
pub mod rules;
pub mod transaction;

#[derive(Serialize)]
struct AccountRender {
    id: i32,
    description: String,
    accumulated: f32,
}

impl AccountRender {
    async fn from_account(pool: &SqlitePool, acc: Account) -> Self {
        let last_acc = Transaction::list(pool, acc.get_id(), 1, 0, false)
            .await
            .map_or(0.0, |x| {
                x.get(0)
                    .map_or(0.0, |x| (x.get_accumulated() as f32) / 100.0)
            });
        Self {
            id: acc.get_id(),
            description: acc.get_account_name().to_string(),
            accumulated: last_acc,
        }
    }
}

fn hm_sort(hm: HashMap<i32, i64>, collapse: usize) -> Vec<(i32, i64)> {
    let mut res: Vec<(i32, i64)> = hm.into_iter().collect();
    res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    if res.len() > collapse {
        let rest = res
            .split_off(collapse)
            .iter()
            .fold(0i64, |acc, item| acc + item.1);
        let last = res.last_mut().unwrap();
        *last = (-1, last.1 + rest);
    }
    res
}

pub async fn index(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
) -> impl IntoResponse {
    let mut ctx = Context::new();

    let accounts = Account::list(db.as_ref(), uid.user_id).await.unwrap();
    let mut acc_render = Vec::new();

    for acc in accounts.into_iter() {
        acc_render.push(AccountRender::from_account(db.as_ref(), acc).await);
    }

    ctx.insert("accounts", &acc_render);

    let last_month = Transaction::list_by_date(
        db.as_ref(),
        uid.user_id,
        Some(Utc::now() - chrono::Duration::days(30)),
        Some(Utc::now()),
        None,
        false,
    )
    .await
    .unwrap();

    let mut categories: HashMap<i32, String> = Category::list(db.as_ref())
        .await
        .unwrap()
        .iter()
        .map(|x| (x.category_id, x.name.clone()))
        .collect();
    categories.insert(0, String::from("Unclassified"));
    ctx.insert("categories", &categories);

    let mut income: HashMap<i32, i64> = HashMap::new();
    let mut expenses: HashMap<i32, i64> = HashMap::new();

    for tx in last_month.iter() {
        if tx.get_amount() > 0 {
            let acc = income
                .entry(tx.get_category().unwrap_or(0))
                .or_default()
                .borrow_mut();
            *acc = *acc + tx.get_amount() as i64;
        } else {
            let acc = expenses
                .entry(tx.get_category().unwrap_or(0))
                .or_default()
                .borrow_mut();
            *acc = *acc - tx.get_amount() as i64;
        }
    }

    let income = hm_sort(income, 5);
    let expenses = hm_sort(expenses, 5);
    ctx.insert("income", &income);
    ctx.insert("expenses", &expenses);

    let mut colors = Vec::new();
    colors.extend_from_slice(&(["85FF33", "60F000", "46AF00", "2C6D00", "1A4200"][..income.len()]));

    colors
        .extend_from_slice(&(["FF3333", "C50000", "830000", "570000", "420000"][..expenses.len()]));

    ctx.insert("colors", &colors);

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
            format!("{e:?}"),
        ),
    }
}
