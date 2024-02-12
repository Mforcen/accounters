use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::authorization::Bearer,
    headers::Authorization,
    http::request::Parts,
    response::{IntoResponse, Redirect},
    RequestPartsExt, TypedHeader,
};

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
    S: Send + Sync,
{
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|e| panic!("Could not get cookies: {e}"))
            .unwrap();
        let ut = UserToken {
            user_id: auth.0 .0.token().parse().unwrap(),
        };
        Ok(ut)
    }
}

