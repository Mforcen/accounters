use std::net::{AddrParseError, SocketAddr};
use std::sync::Arc;

use axum::headers::ContentType;
use hyper::{header, StatusCode};
use sqlx::SqlitePool;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use tera::Tera;

use crate::{routes, static_values as templates};

#[derive(Debug)]
pub enum ServerError {
    Db(sqlx::Error),
    Tera(tera::Error),
    Axum(axum::Error),
    Hyper(hyper::Error),
    AddrParse(AddrParseError),
}

impl From<sqlx::Error> for ServerError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(value)
    }
}

impl From<tera::Error> for ServerError {
    fn from(value: tera::Error) -> Self {
        Self::Tera(value)
    }
}

impl From<axum::Error> for ServerError {
    fn from(value: axum::Error) -> Self {
        Self::Axum(value)
    }
}

impl From<hyper::Error> for ServerError {
    fn from(value: hyper::Error) -> Self {
        Self::Hyper(value)
    }
}

impl From<AddrParseError> for ServerError {
    fn from(value: AddrParseError) -> Self {
        Self::AddrParse(value)
    }
}

pub async fn start_server(bind: &str, db_url: &str) -> Result<(), ServerError> {
    let mut tmpls = Tera::default();
    tmpls
        .add_raw_template("base.html", templates::BASE)
        .unwrap();
    tmpls
        .add_raw_template("index.html", templates::INDEX)
        .unwrap();
    tmpls
        .add_raw_template("account_summary.html", templates::ACCOUNT_SUMMARY)
        .unwrap();
    tmpls
        .add_raw_template("account_txs.html", templates::ACCOUNT_TXS)
        .unwrap();
    tmpls
        .add_raw_template("account_add_txs.html", templates::ACCOUNT_ADD_TXS)
        .unwrap();
    tmpls
        .add_raw_template("categories_new.html", templates::CATEGORIES_NEW)
        .unwrap();
    tmpls
        .add_raw_template("classifiers.html", templates::CLASSIFIERS)
        .unwrap();
    tmpls
        .add_raw_template("rules_new.html", templates::RULES_NEW)
        .unwrap();
    tmpls
        .add_raw_template("rules_new_success.html", templates::RULES_NEW_SUCCESS)
        .unwrap();
    tmpls
        .add_raw_template("transaction.html", templates::TRANSACTION)
        .unwrap();
    let db = accounters::create_db(db_url).await?;

    let state = AppState {
        db: Arc::new(db),
        tmpls: Arc::new(tmpls),
    };

    let exec_id: u32 = rand::random();

    let app = Router::new()
        .nest(
            "/",
            Router::new()
                .route("/", get(routes::ui::index))
                .route("/accounts/id/:id", get(routes::ui::account::show))
                .route(
                    "/accounts/id/:id/transactions/add",
                    get(routes::ui::account::add_transactions_view)
                        .post(routes::ui::account::add_transactions_action),
                )
                .route(
                    "/accounts/id/:id/transactions",
                    get(routes::ui::account::list_transactions),
                )
                .route(
                    "/transaction/:id",
                    get(routes::ui::transaction::view).post(routes::ui::transaction::update),
                )
                .route(
                    "/classifiers",
                    get(routes::ui::classifier::view_classifiers),
                )
                .route(
                    "/classifiers/new_rule",
                    get(routes::ui::classifier::rules_new_view)
                        .post(routes::ui::classifier::rules_new_action),
                )
                .route(
                    "/classifiers/new_category",
                    get(routes::ui::classifier::category_new_view)
                        .post(routes::ui::classifier::category_new_action),
                )
                .nest(
                    "/static",
                    Router::new()
                        .route("/styles.css", get(routes::static_routes::styles))
                        .route("/csv.js", get(routes::static_routes::csv)),
                )
                .route(
                    "/execution",
                    get(move || async move { format!("{exec_id}") }),
                ),
        )
        .nest(
            "/api/v1",
            Router::new()
                .route(
                    "/healthcheck",
                    get(|| async {
                        (
                            StatusCode::OK,
                            [(header::CONTENT_TYPE, "text/html")],
                            String::from("<html><body><div>Hello world from the healthcheck!</div></body></html>"),
                        )
                    }),
                )
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
                    "/accounts/id/:id/recategorize",
                    post(routes::api::accounts::recategorize),
                )
                .route("/categories", post(routes::api::categories::create))
                .route("/categories", get(routes::api::categories::list))
                .route("/rules", post(routes::api::rules::create))
                .route("/rules", get(routes::api::rules::list)),
        )
        .with_state(state);

    let addr: SocketAddr = bind.parse()?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqlitePool>,
    pub tmpls: Arc<Tera>,
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
