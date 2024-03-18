use std::sync::Arc;

use accounters::models::categories::Category;
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
    State(tmpl): State<Arc<Tera>>,
    uid: UserToken,
) -> impl IntoResponse {
    match Category::list(db.as_ref()).await {
        Ok(categories) => {
            let mut ctx = Context::new();
            ctx.insert("categories", &categories);
            (
                StatusCode::OK,
                [(CONTENT_TYPE, "text/html;charset=utf-8")],
                tmpl.render("categories_list.html", &ctx).unwrap(),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain;charset=utf-8")],
            format!("{e}"),
        ),
    }
}

pub async fn new_view(State(tmpl): State<Arc<Tera>>, uid: UserToken) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpl.render("categories_new.html", &Context::new()).unwrap(),
    )
}

#[derive(Deserialize)]
pub struct NewRuleParams {
    pub name: String,
    pub description: String,
}

pub async fn new_action(
    State(db): State<Arc<SqlitePool>>,
    State(tmpls): State<Arc<Tera>>,
    uid: UserToken,
    Form(params): Form<NewRuleParams>,
) -> impl IntoResponse {
    match Category::new(db.as_ref(), &params.name, &params.description).await {
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
