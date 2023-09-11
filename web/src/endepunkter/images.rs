use crate::ValidatedUser;
use axum::body::StreamBody;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::{debug_handler, ServiceExt};
use database::assets::Asset;
use hyper::header;
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;
use tokio_util::io::{ReaderStream, StreamReader};

const TEMPORARY_COVER_DIR: &str = "./covers";

#[utoipa::path(
    get,
    path = "/api/v1/images/covers/{asset_id}",
    params(
        ("asset_id" = String, Path, description = "The asset_id for the image"),
    ),
    responses(
        (status = 200, content_type = "image/jpeg")
    )
)]
#[debug_handler]
pub async fn get_cover(
    State(pool): State<SqlitePool>,
    Path(asset_id): Path<String>,
) -> impl IntoResponse {
    let cover_asset_option = match Asset::get_asset(&asset_id, &pool).await {
        Ok(asset) => asset,
        Err(_) => return Err(ImageError::InternalError),
    };

    let cover_asset = match cover_asset_option {
        Some(asset) => asset,
        None => return Err(ImageError::ImageNotFound),
    };

    let cover_path = cover_asset.local_path;
    let extension = match cover_asset.file_extension {
        Some(extension) => extension,
        None => return Err(ImageError::InternalError),
    };

    let file = match tokio::fs::File::open(cover_path).await {
        Ok(file) => file,
        Err(Error) => {
            println!("Failed to open cover: {}", Error);
            return Err(ImageError::FailedToOpen);
        }
    };

    let stream = ReaderStream::new(file);

    let body = StreamBody::new(stream).into_response();

    let headers = [
        (header::CONTENT_TYPE, extension.as_str()),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}.jpg\"", asset_id),
        ),
    ];

    Ok((headers, body).into_response())
}
pub enum ImageError {
    InternalError,
    FailedToOpen,
    ImageNotFound,
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ImageError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
            ImageError::FailedToOpen => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open"),
            ImageError::ImageNotFound => (StatusCode::NOT_FOUND, "Image not found"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
