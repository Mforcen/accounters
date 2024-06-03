use std::fs;

use axum::response::IntoResponse;
use hyper::{header::CONTENT_TYPE, StatusCode};

use crate::static_values;

pub async fn styles() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/css")],
        static_values::STYLES,
    )
}

pub async fn csv() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/javascript")],
        static_values::CSV,
    )
}
