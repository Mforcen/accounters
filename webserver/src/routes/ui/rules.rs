use std::sync::Arc;

use accounters::models::{categories::Category, rules::Rule};
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use hyper::{header::CONTENT_TYPE, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;

pub async fn list(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
) -> impl IntoResponse {
    let rules = match Rule::list_by_user(db.as_ref(), uid.user_id).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, "text/plain")],
                format!("{e:?}"),
            );
        }
    };

    let mut ctx = Context::new();

    ctx.insert("rules", &rules);

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("rules_list.html", &ctx).unwrap(),
    )
}

pub async fn new_view(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
) -> impl IntoResponse {
    let categories = Category::list(db.as_ref()).await.unwrap();
    let mut ctx = Context::new();
    ctx.insert("categories", &categories);
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("rules_new.html", &ctx).unwrap(),
    )
}

#[derive(Deserialize)]
pub struct NewRuleParams {
    pub description: String,
    pub regex: String,
    pub category: i32,
}

pub async fn new_action(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Form(params): Form<NewRuleParams>,
) -> impl IntoResponse {
    match Rule::new(db.as_ref(), uid.user_id, params.regex, params.category).await {
        Ok(_) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/html;charset=utf-8")],
            tmpls
                .render("rules_new_success.html", &Context::new())
                .unwrap(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain;charset=utf-8")],
            format!("{e}"),
        ),
    }
}
