use std::fs;

use axum::response::IntoResponse;
use hyper::{header::CONTENT_TYPE, StatusCode};

pub async fn styles() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/css")],
        fs::read_to_string("static/styles.css").unwrap(),
    )
}

pub async fn csv() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/javascript")],
        fs::read_to_string("static/csv.js").unwrap(),
    )
}
