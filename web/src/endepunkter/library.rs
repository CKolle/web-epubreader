use axum::debug_handler;
use axum::extract::{FromRef, State};
use axum::Json;
use axum::{
    async_trait,
    extract::{FromRequestParts, Path, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    RequestPartsExt,
};
use database::library::{InsertableLibrary, Library};
use database::users::BearerToken;
use scanner::LibraryScanner;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use utoipa::ToSchema;

use crate::endepunkter::auth::AuthError;

#[utoipa::path(
    post,
    path = "/api/v1/library",
    request_body = Library,
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn add_library(
    State(pool): State<SqlitePool>,
    _: ValidatedUser,
    Json(library): Json<InsertableLibrary>,
) -> Result<Json<LibraryBody>, LibraryError> {
    let library_result = library.insert(&pool).await;

    match library_result {
        Ok(library) => Ok(Json(LibraryBody {
            id: library.id,
            path: library.path,
            name: library.name,
        })),
        Err(_) => Err(LibraryError::InternalError),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/library",
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn get_libraries(
    State(pool): State<SqlitePool>,
    _: ValidatedUser,
) -> Result<Json<Vec<LibraryBody>>, LibraryError> {
    let libraries = Library::get_libraries(&pool).await;

    match libraries {
        Ok(libraries) => {
            let libraries = libraries
                .into_iter()
                .map(|library| LibraryBody {
                    id: library.id,
                    path: library.path,
                    name: library.name,
                })
                .collect();

            Ok(Json(libraries))
        }
        Err(_) => Err(LibraryError::InternalError),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/library/{id}",
    params(("id" = i32, Path, description = "Library id")),
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn delete_library(
    State(pool): State<SqlitePool>,
    _: ValidatedUser,
    Path(id): Path<i32>,
) -> Result<Json<GenericSuccess>, LibraryError> {
    let library = Library::delete_library(id, &pool).await;

    match library {
        Ok(_) => Ok(Json(GenericSuccess {
            success: "Library deleted".to_string(),
        })),
        Err(_) => Err(LibraryError::InternalError),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/library/scan",
    request_body = LibraryScanOptions,
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn scan_library(
    State(pool): State<SqlitePool>,
    _: ValidatedUser,
    Json(options): Json<LibraryScanOptions>,
) -> Result<Json<GenericSuccess>, LibraryError> {
    let result = Library::get_library(options.library_id, &pool).await;

    let library = match result {
        Ok(library) => library,
        Err(_) => return Err(LibraryError::InternalError),
    };

    tokio::spawn(async move {
        println!("Scanning library {}", &library.path);
        println!("Using new scanner");
        match scanner::scanner::clean_up(&library.path, &pool).await {
            Ok(_) => println!("Done cleaning up library {}", &library.path),
            Err(_) => println!("Error cleaning up library {}", &library.path),
        }
        match scanner::scanner::force_scan(&library.path, options.library_id, &pool).await {
            Ok(_) => println!("Done scanning library {}", &library.path),
            Err(_) => println!("Error scanning library can't continue"),
        }
    });

    // tokio::spawn(async move {
    //     println!("Scanning library {}", library.path);
    //     let mut scanner = LibraryScanner::new(library.path.clone());
    //     match scanner.scan().await {
    //         Ok(_) => println!("Done scanning library {}", library.path),
    //         Err(_) => {
    //             println!("Error scanning library can't continue");
    //             return;
    //         }
    //     };

    //     match scanner.store_to_db(library.id, &pool).await {
    //         Ok(_) => println!("Done storing library {}", library.path),
    //         Err(_) => println!("Error storing library {}", library.path),
    //     };
    // });

    Ok(Json(GenericSuccess {
        success: "Library scan started".to_string(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct LibraryScanOptions {
    library_id: i32,
}

pub enum LibraryError {
    InvalidPath,
    InternalError,
}

impl IntoResponse for LibraryError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            LibraryError::InvalidPath => (StatusCode::BAD_REQUEST, "Invalid path"),
            LibraryError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(Serialize)]

pub struct LibraryBody {
    id: i32,
    path: String,
    name: String,
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
