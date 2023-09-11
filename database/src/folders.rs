use sqlx;
use sqlx::{Pool, Row, Sqlite};
use std::cmp::PartialEq;

pub struct InsertableFolder {
    pub nickname: String,
    pub path: String,
    pub watch: bool,
    pub drop_type: DropType,
}

impl InsertableFolder {
    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<Folder, sqlx::Error> {
        let Self {
            nickname: folder_nickname,
            path: folder_path,
            watch,
            drop_type,
        } = self;

        let result = sqlx::query(
            r#"
            INSERT INTO folders (nickname, path, watch, drop_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(&folder_nickname)
        .bind(&folder_path)
        .bind(watch)
        .bind(&drop_type)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get("id");

        Ok(Folder {
            id,
            nickname: folder_nickname,
            path: folder_path,
            watch,
            drop_type,
        })
    }
}

#[derive(sqlx::Type, PartialEq, Debug)]
pub enum DropType {
    Source,
    Destination,
    None,
}

#[derive(Debug)]
pub struct Folder {
    pub id: i32,
    pub nickname: String,
    pub path: String,
    pub watch: bool,
    pub drop_type: DropType,
}
