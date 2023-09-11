use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::{Pool, Row, Sqlite};
use utoipa::ToSchema;
use crate::assets::Asset;

use crate::assets;

#[derive(Deserialize, ToSchema)]
#[schema(as = Library)]
pub struct InsertableLibrary {
    pub path: String,
    pub name: String,
}

// Todo: Implement check for duplicate path and invalid path

impl InsertableLibrary {
    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<Library, sqlx::Error> {
        let Self { path, name } = self;

        let result = sqlx::query(
            r#"
            INSERT INTO libraries (path, name)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(&path)
        .bind(&name)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get("id");

        let root_collection = InsertableCollection {
            library_id: id,
            path: path.clone(),
            name: "root".to_string(),
        };

        root_collection.insert(pool).await.unwrap();

        Ok(Library { id, path, name })
    }
}

#[derive(sqlx::FromRow)]
pub struct Library {
    pub id: i32,
    pub path: String,
    pub name: String,
}

impl Library {
    pub async fn get_library(id: i32, pool: &Pool<Sqlite>) -> Result<Library, sqlx::Error> {
        let library: Library = sqlx::query_as::<_, Library>(
            r#"
            SELECT * FROM libraries WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(library)
    }

    pub async fn get_by_path(
        path: &str,
        pool: &Pool<Sqlite>,
    ) -> Result<Option<Library>, sqlx::Error> {
        let library: Option<Library> = sqlx::query_as::<_, Library>(
            r#"
            SELECT * FROM libraries WHERE path = $1
            "#,
        )
        .bind(path)
        .fetch_optional(pool)
        .await?;

        Ok(library)
    }

    pub async fn get_libraries(pool: &Pool<Sqlite>) -> Result<Vec<Library>, sqlx::Error> {
        let libraries: Vec<Library> = sqlx::query_as::<_, Library>(
            r#"
            SELECT * FROM libraries
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(libraries)
    }

    pub async fn delete_library(id: i32, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM libraries WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_self(self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM libraries WHERE id = $1
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_library_contents(id: i32, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM books WHERE library_id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await
        .unwrap();

        // Cannot delete root collection
        sqlx::query(
            r#"
            DELETE FROM collections WHERE name != "root" AND library_id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct Book {
    pub id: i32,
    pub asset_id: String,
    pub name: String,
    pub library_id: i32,
    pub collection_id: i32,
    pub primary_cover: Option<String>,
}

impl Book {
    pub async fn get_book(id: i32, pool: &Pool<Sqlite>) -> Result<Book, sqlx::Error> {
        let book: Book = sqlx::query_as::<_, Book>(
            r#"
            SELECT * FROM books WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(book)
    }

    pub async fn get_books(pool: &Pool<Sqlite>) -> Result<Vec<Book>, sqlx::Error> {
        let books: Vec<Book> = sqlx::query_as::<_, Book>(
            r#"
            SELECT * FROM books
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(books)
    }

    pub async fn get_books_by_library(
        library_id: i32,
        pool: &Pool<Sqlite>,
    ) -> Result<Vec<Book>, sqlx::Error> {
        let books: Vec<Book> = sqlx::query_as::<_, Book>(
            r#"
            SELECT * FROM books WHERE library_id = $1
            "#,
        )
        .bind(library_id)
        .fetch_all(pool)
        .await?;

        Ok(books)
    }

    pub async fn delete_self(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM books WHERE id = $1
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct InsertableBook {
    pub path: String,
    pub name: String,
    pub library_id: i32,
    pub collection_id: Option<i32>,
    pub primary_cover: Option<String>,
}

impl InsertableBook {
    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<Book, sqlx::Error> {
        let Self {
            path,
            name,
            library_id,
            collection_id,
            primary_cover,
        } = self;

        // One indicates the root collection
        let collection_id = match collection_id {
            Some(id) => id,
            None => 1,
        };

        // Need to create the asset first

        let insertable_asset = assets::InsertableAsset {
            local_path: path,
            file_extension: Some("application/epub+zip".to_string()),
        };

        let asset_id = insertable_asset.insert(pool).await.unwrap().id;

        let result = sqlx::query(
            r#"
            INSERT INTO books (asset_id, name, library_id, collection_id, primary_cover)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(&asset_id)
        .bind(&name)
        .bind(&library_id)
        .bind(&collection_id)
        .bind(&primary_cover)
        .fetch_one(pool)
        .await
        .unwrap();

        let id: i32 = result.get("id");

        Ok(Book {
            id,
            asset_id,
            name,
            library_id,
            collection_id,
            primary_cover: primary_cover,
        })
    }

    pub fn add_cover(&mut self, asset_id: String) {
        self.primary_cover = Some(asset_id);
    }
}

#[derive(sqlx::FromRow)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub library_id: i32,
    pub path: String,
}

impl Collection {
    pub async fn get_by_libary(
        library_id: i32,
        pool: &Pool<Sqlite>,
    ) -> Result<Vec<Collection>, sqlx::Error> {
        let collections: Vec<Collection> = sqlx::query_as::<_, Collection>(
            r#"
            SELECT * FROM collections WHERE library_id = $1
            "#,
        )
        .bind(library_id)
        .fetch_all(pool)
        .await?;

        Ok(collections)
    }

    pub async fn delete_self(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM collections WHERE id = $1
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct InsertableCollection {
    pub name: String,
    pub library_id: i32,
    pub path: String,
}

impl InsertableCollection {
    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<Collection, sqlx::Error> {
        let Self {
            name,
            library_id,
            path,
        } = self;

        let result = sqlx::query(
            r#"
            INSERT INTO collections (name, library_id, path)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(&name)
        .bind(&library_id)
        .bind(&path)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get("id");

        Ok(Collection {
            id,
            name,
            library_id,
            path,
        })
    }
}


pub struct InsertableBookProgress {
    pub book_id: i32,
    pub user_id: i32,
    pub page: i32,
    pub page_progress: f32,
}

impl InsertableBookProgress {
    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<BookProgress, sqlx::Error> {
        let Self {
            book_id,
            user_id,
            page,
            page_progress,
        } = self;

        let result = sqlx::query(
            r#"
            INSERT INTO book_progress (book_id, user_id, page, page_progress)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(&book_id)
        .bind(&user_id)
        .bind(&page)
        .bind(&page_progress)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get("id");

        Ok(BookProgress {
            id,
            book_id,
            user_id,
            page,
            page_progress,
        })
    }
}

#[derive(sqlx::FromRow)]
pub struct BookProgress {
    pub id: i32,
    pub book_id: i32,
    pub user_id: i32,
    pub page: i32,
    pub page_progress: f32,
}
