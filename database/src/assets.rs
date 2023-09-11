use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::{Pool, Row, Sqlite};
use utoipa::ToSchema;
use uuid::Uuid;

pub struct InsertableAsset {
    pub local_path: String,
    pub file_extension: Option<String>,
}
impl InsertableAsset {
    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<Asset, sqlx::Error> {
        let Self {
            local_path,
            file_extension,
        } = self;

        // Assets will be exposed in the url so the id should be ungessable
        let uuid = Uuid::new_v4().to_string();

        let result = sqlx::query(
            r#"
            INSERT INTO assets (id, local_path, file_extension)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(&uuid)
        .bind(&local_path)
        .bind(&file_extension)
        .fetch_one(pool)
        .await?;

        let id = result.get("id");

        Ok(Asset {
            id,
            local_path,
            file_extension,
        })
    }
}

#[derive(sqlx::FromRow)]
pub struct Asset {
    pub id: String,
    pub local_path: String,
    pub file_extension: Option<String>,
}

impl Asset {
    pub async fn into_book_cover(
        &self,
        book_id: i32,
        pool: &Pool<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO book_covers (book_id, asset_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(&book_id)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_self(self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM assets WHERE id = $1
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_asset(id: &str, pool: &Pool<Sqlite>) -> Result<Option<Asset>, sqlx::Error> {
        let asset: Option<Asset> = sqlx::query_as::<_, Asset>(
            r#"
            SELECT * FROM assets WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(asset)
    }

    // Technically, this is a book cover, but a book cover is an asset. And the book_cover table is a lookup table.
    pub async fn get_by_book_id(book_id: i32, pool: &Pool<Sqlite>) -> Result<Asset, sqlx::Error> {
        let asset: Asset = sqlx::query_as::<_, Asset>(
            r#"
            SELECT assets.* FROM assets
            INNER JOIN book_covers ON book_covers.asset_id = assets.id
            WHERE book_covers.book_id = $1
            "#,
        )
        .bind(book_id)
        .fetch_one(pool)
        .await?;

        Ok(asset)
    }

    pub async fn get_cover_assets_book(
        book_id: i32,
        pool: &Pool<Sqlite>,
    ) -> Result<Vec<Asset>, sqlx::Error> {
        let assets: Vec<Asset> = sqlx::query_as::<_, Asset>(
            r#"
            SELECT assets.* FROM assets
            INNER JOIN book_covers ON book_covers.asset_id = assets.id
            WHERE book_covers.book_id = $1
            "#,
        )
        .bind(book_id)
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    pub async fn get_cover_assets_library(
        library_id: i32,
        pool: &Pool<Sqlite>,
    ) -> Result<Vec<Asset>, sqlx::Error> {
        let assets: Vec<Asset> = sqlx::query_as::<_, Asset>(
            r#"
            SELECT assets.* FROM assets
            INNER JOIN book_covers ON book_covers.asset_id = assets.id
            INNER JOIN books ON books.id = book_covers.book_id
            WHERE library_id = $1
            "#,
        )
        .bind(library_id)
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    pub async fn delete_asset(id: &str, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM assets WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
