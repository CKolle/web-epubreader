use axum::debug_handler;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use database::users::Login;
use database::users::LoginError;
use database::users::Register;
use database::users::RegisterError;
use hyper::StatusCode;
use serde::Serialize;
use serde_json::json;
use sqlx::sqlite::SqlitePool;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = Register,
    responses(
        (status = 200, content_type = "application/json")
    )
)]
#[debug_handler]
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(register): Json<Register>,
) -> Result<Json<AuthBody>, AuthError> {
    let user_result = register.register(&pool).await;

    match user_result {
        Ok(token) => Ok(Json(AuthBody {
            bearer_token: token.token,
        })),
        Err(RegisterError::UsernameTaken) => Err(AuthError::UsernameTaken),
        Err(RegisterError::DatabaseError(_)) => Err(AuthError::WrongCredentials),
    }
}

pub enum AuthError {
    WrongCredentials,
    UsernameTaken,
    InvalidToken,
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::UsernameTaken => (StatusCode::CONFLICT, "Username taken"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct AuthBody {
    bearer_token: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = Login,
    responses(
        (status = 200, content_type = "application/json")
    )
)]
#[debug_handler]
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(login): Json<Login>,
) -> Result<Json<AuthBody>, AuthError> {
    let login_result = login.login(&pool).await;

    match login_result {
        Ok(token) => Ok(Json(AuthBody {
            bearer_token: token.token,
        })),
        Err(LoginError::UserNotFound) => Err(AuthError::WrongCredentials),
        Err(LoginError::PasswordIncorrect) => Err(AuthError::WrongCredentials),
        Err(LoginError::DatabaseError(_)) => Err(AuthError::InternalError),
    }
}
