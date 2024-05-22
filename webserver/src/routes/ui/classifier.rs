use std::sync::Arc;

use accounters::models::{categories::Category, rules::Rule};
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use hyper::{
    header::{CONTENT_TYPE, LOCATION},
    StatusCode,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};

use crate::users::UserToken;

pub async fn view_classifiers(
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

    let categories = match Category::list(db.as_ref()).await {
        Ok(categories) => categories,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, "text/plain;charset=utf-8")],
                format!("{e}"),
            )
        }
    };

    let mut ctx = Context::new();

    ctx.insert("rules", &rules);
    ctx.insert("categories", &categories);

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpls.render("classifiers.html", &ctx).unwrap(),
    )
}

pub async fn rules_new_view(
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

pub async fn rules_new_action(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Form(params): Form<NewRuleParams>,
) -> impl IntoResponse {
    match Rule::new(db.as_ref(), uid.user_id, params.regex, params.category).await {
        Ok(_) => (
            StatusCode::MOVED_PERMANENTLY,
            [(LOCATION, "/classifiers")],
            String::new(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain;charset=utf-8")],
            format!("{e}"),
        ),
    }
}

pub async fn category_new_view(State(tmpl): State<Arc<Tera>>, uid: UserToken) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/html;charset=utf-8")],
        tmpl.render("categories_new.html", &Context::new()).unwrap(),
    )
}

#[derive(Deserialize)]
pub struct CategoryNewRuleParams {
    pub name: String,
    pub description: String,
}

pub async fn category_new_action(
    State(db): State<Arc<SqlitePool>>,
    uid: UserToken,
    Form(params): Form<CategoryNewRuleParams>,
) -> impl IntoResponse {
    match Category::new(db.as_ref(), &params.name, &params.description).await {
        Ok(_) => (
            StatusCode::MOVED_PERMANENTLY,
            [(LOCATION, "/classifiers")],
            String::new(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain;charset=utf-8")],
            format!("{e}"),
        ),
    }
}
