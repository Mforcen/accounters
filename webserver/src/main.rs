use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::SqlitePool;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use hyper::StatusCode;

mod routes;
mod users;

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() {
    let db = accounters::create_db(DB_URL).await.unwrap();

    let state = AppState { db: Arc::new(db) };

    let app = Router::new()
        .route("/", get(index))
        .nest(
            "/api/v1",
            Router::new()
                .route("/user", post(routes::create_user))
                .route("/login", post(routes::login))
                .route("/accounts", post(routes::accounts::account_create))
                .route("/accounts", get(routes::accounts::account_list))
                .route("/accounts/id/:id", get(routes::accounts::account_get))
                .route(
                    "/accounts/id/:id/transaction",
                    post(routes::transactions::create),
                )
                .route(
                    "/accounts/id/:id/transaction",
                    get(routes::transactions::list),
                )
                .route(
                    "/accounts/id/:id/update",
                    post(routes::accounts::snapshot_update),
                )
                .route(
                    "/accounts/id/:id/recategorize",
                    post(routes::accounts::recategorize),
                )
                .route("/categories", post(routes::categories::create))
                .route("/categories", get(routes::categories::list))
                .route("/rules", post(routes::rules::create))
                .route("/rules", get(routes::rules::list)),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    db: Arc<SqlitePool>,
}

impl FromRef<AppState> for Arc<SqlitePool> {
    fn from_ref(state: &AppState) -> Arc<SqlitePool> {
        state.db.clone()
    }
}

async fn index() -> (StatusCode, String) {
    (StatusCode::OK, String::from("Hello, World!"))
}
