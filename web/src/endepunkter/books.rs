use std::os::fd::FromRawFd;

use crate::ValidatedUser;
use axum::body::{Full, StreamBody};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Json, Response};
use database::assets::Asset;
use database::library::{Book, InsertableBookProgress};
use epub::doc::EpubDoc;
use hyper::header;
use hyper::StatusCode;
use scanner::epub_sandbox::{Epub, EpubError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::SqlitePool;

use tokio::fs::File;

#[utoipa::path(
    get,
    path = "/api/v1/book",
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn get_books(
    State(pool): State<SqlitePool>,
    _: ValidatedUser,
) -> Result<Json<Vec<BookBody>>, BookError> {
    let books: Vec<Book> = match Book::get_books(&pool).await {
        Ok(books) => books,
        Err(_) => return Err(BookError::InternalError),
    };

    let books = books
        .into_iter()
        .map(|book| BookBody {
            id: book.id,
            title: book.name,
            book_asset: book.asset_id,
            primary_cover: book.primary_cover,
        })
        .collect();

    Ok(Json(books))
}

#[utoipa::path(
    get,
    path = "/api/v1/books/{book_id}",
    params(
        ("book_id" = i32, Path, description = "The id of the book to get the cover for"),
    ),
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn get_book(
    State(pool): State<SqlitePool>,
    _: ValidatedUser,
    Path(book_id): Path<i32>,
) -> Result<Json<BookBody>, BookError> {
    let book = match Book::get_book(book_id, &pool).await {
        Ok(book) => book,
        Err(_) => return Err(BookError::InternalError),
    };

    Ok(Json(BookBody {
        id: book.id,
        title: book.name,
        book_asset: book.asset_id,
        primary_cover: book.primary_cover,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/books/{book_id}/sync",
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn sync_book(
    State(pool): State<SqlitePool>,
    user: ValidatedUser,
    Path(book_id): Path<i32>,
    Json(book_progress): Json<InsertableBookProgress>,
) -> Result<Json<BookSyncResult>, BookError> {

    match Book::get_book(book_id, &pool).await {
        Ok(book) => book,
        Err(_) => return Err(BookError::InternalError),
    };

    match book_progress.insert(&pool).await {
        Ok(book_progess) => book_progess,
        Err(_) => return Err(BookError::InternalError),
    };

    Ok(Json(BookSyncResult {
        status: "Book synced".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/book/{asset_id}/page/{page_num}",
    params(
        ("asset_id" = i32, Path, description = "The id of the book to get the cover for"),
        ("page_num" = uize, Path, description = "The id of the page to get"),
    ),
    responses(
        (status = 200, content_type = "application/json")
    )
)]
pub async fn get_book_page(
    State(pool): State<SqlitePool>,
    Path((asset_id, page_num)): Path<(String, usize)>,
) -> Result<impl IntoResponse, BookError> {
    let book_asset = Asset::get_asset(&asset_id, &pool)
        .await?
        .ok_or(BookError::InvalidPath)?;

    let book_path = std::path::PathBuf::from(book_asset.local_path);
    let mut epub = Epub::new(&book_path)?;
    let page = epub
        .get_page(page_num, &asset_id)
        .ok_or(BookError::InvalidPath)?;
    let html = page.0;

    let headers = [(header::CONTENT_TYPE, "text/html")];

    Ok((headers, html).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/book/{asset_id}/resource/*path",
    params(
        ("asset_id" = i32, Path, description = "The asset_id of the book to get the cover for"),
        ("path" = String, Path, description = "The path of the resource to get"),
    ),
    responses(
        (status = 200, content_type = "application/json")
    )
)]
#[debug_handler]
pub async fn get_book_resource(
    Path((asset_id, path)): Path<(String, String)>,

    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, BookError> {
    // Need to use a query for the bearer token
    // Because it will be used in background-image css, and that doesn't support headers

    let book_asset = Asset::get_asset(&asset_id, &pool)
        .await?
        .ok_or(BookError::InvalidPath)?;

    let book_path = std::path::PathBuf::from(book_asset.local_path);
    let mut epub = Epub::new(&book_path)?;

    let res_path = std::path::PathBuf::from(&path);
    let resource = epub
        .get_res_by_path(&res_path)
        .ok_or_else(|| BookError::InvalidPath)?
        .into();
    let body = Full::new(resource);

    if path.ends_with(".css") {
        let headers = [(header::CONTENT_TYPE, "text/css")];
        return Ok((headers, body));
    }

    if path.contains(".html") {
        let headers = [(header::CONTENT_TYPE, "text/html")];
        return Ok((headers, body));
    }

    let headers = [(header::CONTENT_TYPE, "image/jpg")];

    Ok((headers, body))
}

#[derive(Deserialize)]
pub struct ResourceQuery {
    path: String,
}

#[derive(Serialize)]
pub struct BookBody {
    id: i32,
    title: String,
    book_asset: String,
    primary_cover: Option<String>,
}

#[derive(Deserialize)]
pub struct BookSync {
    pub page_num: usize,
    pub page_percent: f32, 
}

#[derive(Serialize)]
pub struct BookSyncResult {
    pub status: String,
}

pub enum BookError {
    InternalError,
    InvalidPath,
    BadFile,
}

impl From<sqlx::Error> for BookError {
    fn from(_: sqlx::Error) -> Self {
        BookError::InternalError
    }
}

impl From<EpubError> for BookError {
    fn from(_: EpubError) -> Self {
        BookError::BadFile
    }
}

impl IntoResponse for BookError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            BookError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
            BookError::InvalidPath => (StatusCode::BAD_REQUEST, "Invalid path"),
            BookError::BadFile => (StatusCode::BAD_REQUEST, "Bad file"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
