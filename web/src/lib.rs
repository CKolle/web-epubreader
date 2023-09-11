use axum::extract::FromRef;
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt,
};
use database::users::BearerToken;
use endepunkter::auth::AuthError;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;

pub mod endepunkter;

use axum::{Router, Server};

pub async fn serve(socket_addr: std::net::SocketAddr, app: Router) -> Result<(), hyper::Error> {
    Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
}

#[derive(Serialize)]
pub struct GenericSuccess {
    success: String,
}

pub struct ValidatedUser {
    user_id: i32,
    username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for ValidatedUser
where
    SqlitePool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = SqlitePool::from_ref(state);

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token = bearer.token().to_string();
        let token_object = BearerToken::find_by_token(&token, &pool)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        match token_object {
            Some(token_object) => Ok(ValidatedUser {
                user_id: token_object.user_id,
                username: "".to_string(),
            }),
            None => Err(AuthError::InvalidToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
