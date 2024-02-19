use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::SqlitePool;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use hyper::StatusCode;
use tera::Tera;

mod routes;
mod users;

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() {
    let db = accounters::create_db(DB_URL).await.unwrap();

    let mut tmpls = Tera::new("templates/*").unwrap();
    tmpls.autoescape_on(vec!["html"]);

    let state = AppState {
        db: Arc::new(db),
        tmpls: Arc::new(tmpls),
    };

    let app = Router::new()
        .nest(
            "/",
            Router::new()
                .route("/", get(routes::ui::index))
                .route("/accounts/id/:id", get(routes::ui::account))
                .route("/rules", get(routes::ui::rules::list))
                .route("/rules/new", get(routes::ui::rules::new_view))
                .route("/rules/new", post(routes::ui::rules::new_action))
                .nest(
                    "/static",
                    Router::new().route("/styles.css", get(routes::static_routes::styles)),
                ),
        )
        .nest(
            "/api/v1",
            Router::new()
                .route("/user", post(routes::api::create_user))
                .route("/login", post(routes::api::login))
                .route("/accounts", post(routes::api::accounts::account_create))
                .route("/accounts", get(routes::api::accounts::account_list))
                .route("/accounts/id/:id", get(routes::api::accounts::account_get))
                .route(
                    "/accounts/id/:id/transaction",
                    post(routes::api::transactions::create),
                )
                .route(
                    "/accounts/id/:id/transaction",
                    get(routes::api::transactions::list),
                )
                .route(
                    "/accounts/id/:id/update",
                    post(routes::api::accounts::snapshot_update),
                )
                .route(
                    "/accounts/id/:id/recategorize",
                    post(routes::api::accounts::recategorize),
                )
                .route("/categories", post(routes::api::categories::create))
                .route("/categories", get(routes::api::categories::list))
                .route("/rules", post(routes::api::rules::create))
                .route("/rules", get(routes::api::rules::list)),
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
    tmpls: Arc<Tera>,
}

impl FromRef<AppState> for Arc<SqlitePool> {
    fn from_ref(state: &AppState) -> Arc<SqlitePool> {
        state.db.clone()
    }
}

impl FromRef<AppState> for Arc<Tera> {
    fn from_ref(state: &AppState) -> Arc<Tera> {
        state.tmpls.clone()
    }
}

async fn index() -> (StatusCode, String) {
    (StatusCode::OK, String::from("Hello, World!"))
}
