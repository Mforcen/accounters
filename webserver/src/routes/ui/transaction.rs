use std::sync::Arc;

use accounters::models::{categories::Category, transaction::Transaction};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Form,
};
use chrono::{DateTime, Utc};
use hyper::{header, StatusCode};
use serde::{Deserialize, Deserializer};
use sqlx::SqlitePool;
use tera::Tera;

pub async fn view(
    db: State<Arc<SqlitePool>>,
    tmpl: State<Arc<Tera>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let tx = Transaction::get_by_id(db.as_ref(), id).await.unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("tx_id", &id);
    ctx.insert("tx", &tx);

    let categories = Category::list(db.as_ref()).await.unwrap();
    ctx.insert("categories", &categories);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpl.render("transaction.html", &ctx).unwrap(),
    )
}

fn deserialize_optional<'de, D>(data: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let str = String::deserialize(data)?;
    if str.is_empty() {
        Ok(None)
    } else {
        Ok(Some(str.parse().unwrap()))
    }
}

#[derive(Deserialize, Debug)]
pub struct TxUpdateRequest {
    description: String,
    date: DateTime<Utc>,
    amount: f32,
    #[serde(deserialize_with = "deserialize_optional")]
    category: Option<i32>,
}

pub async fn update(
    db: State<Arc<SqlitePool>>,
    Path(id): Path<i32>,
    Form(req): Form<TxUpdateRequest>,
) -> impl IntoResponse {
    let ret_str = format!("/transaction/{id}");
    let mut tx = match Transaction::get_by_id(db.as_ref(), id).await {
        Ok(tx) => tx,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                [(header::LOCATION, ret_str)],
                format!("{e:?}"),
            );
        }
    };

    let amount = (req.amount * 100.0).round() as i32;

    if tx.get_amount() != amount {
        tx.set_amount(db.as_ref(), amount).await.unwrap();
    }

    if tx.get_description() != req.description {
        tx.set_description(db.as_ref(), &req.description)
            .await
            .unwrap();
    }

    if tx.get_category() != req.category {
        tx.set_category(db.as_ref(), req.category).await.unwrap();
    }

    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, ret_str)],
        String::new(),
    )
}
