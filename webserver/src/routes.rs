use std::sync::Arc;

use axum::extract::{Json, State};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

use accounters::models::users::User;

pub mod accounts;
pub mod categories;
pub mod rules;
pub mod transactions;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    user: String,
    pass: String,
}

pub async fn create_user(
    State(db): State<Arc<SqlitePool>>,
    Json(user_info): Json<CreateUserRequest>,
) -> (StatusCode, String) {
    let exec = User::create_user(db.as_ref(), &user_info.user, &user_info.pass).await;
    match exec {
        Ok(e) => (StatusCode::OK, format!("{}", e.get_id())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}

pub async fn login(
    State(db): State<Arc<SqlitePool>>,
    Json(user_info): Json<CreateUserRequest>,
) -> (StatusCode, String) {
    let user = User::get_user(db.as_ref(), &user_info.user).await.unwrap();

    if user.check_pass(&user_info.pass) {
        (StatusCode::OK, format!("{}", user.get_id()))
    } else {
        (StatusCode::UNAUTHORIZED, String::new())
    }
}
