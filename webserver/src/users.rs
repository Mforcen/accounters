use std::sync::Arc;

use accounters::models::users::User;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    headers::authorization::{Basic, Bearer},
    headers::Authorization,
    http::request::Parts,
    response::{IntoResponse, Redirect},
    RequestPartsExt, TypedHeader,
};
use hyper::StatusCode;

use crate::AppState;

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> axum::response::Response {
        Redirect::temporary("/login").into_response()
    }
}

pub struct UserToken {
    pub user_id: i32,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserToken
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, [(&'static str, &'static str); 1]);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(auth) => Ok(UserToken {
                user_id: auth.0 .0.token().parse().unwrap(),
            }),
            Err(_) => match parts.extract::<TypedHeader<Authorization<Basic>>>().await {
                Ok(auth) => {
                    let state = AppState::from_ref(state);
                    let user = User::get_user(state.db.as_ref(), auth.username())
                        .await
                        .unwrap();
                    if user.check_pass(auth.password()) {
                        Ok(UserToken {
                            user_id: user.get_id(),
                        })
                    } else {
                        Err((StatusCode::UNAUTHORIZED, [("", "")]))
                    }
                }
                Err(_) => Err((
                    StatusCode::UNAUTHORIZED,
                    [("WWW-Authenticate", "Basic realm=\"Access\"")],
                )),
            },
        }
    }
}
