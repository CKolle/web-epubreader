use sqlx::{Pool, Sqlite};
extern crate argon2;

pub mod assets;
pub mod folders;
pub mod library;
pub mod users;

pub async fn create_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&database_url)
        .await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePool;
    use sqlx::Row;

    #[tokio::test]
    async fn test_db() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let res = sqlx::query("SELECT 1 + 1 as val")
            .fetch_one(&pool)
            .await
            .unwrap();
        let sum: i32 = res.get("val");
        assert_eq!(sum, 2);
    }

    #[tokio::test]
    async fn test_register() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Apply migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let register = users::Register {
            username: "test".into(),
            password: "test".into(),
        };

        let token = register.register(&pool).await.unwrap();

        assert_eq!(token.user_id, 1);
        println!("{}", token.token);
    }

    #[tokio::test]
    async fn test_login() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Apply migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let register = users::Register {
            username: "test".into(),
            password: "test".into(),
        };

        let regist_token = register.register(&pool).await.unwrap();

        let login = users::Login {
            username: "test".into(),
            password: "test".into(),
        };

        let login_token = login.login(&pool).await.unwrap();

        assert_eq!(regist_token.token, login_token.token);
        assert_eq!(regist_token.user_id, login_token.user_id);
    }

    #[tokio::test]
    async fn test_existing_user() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Apply migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let register = users::Register {
            username: "alice".into(),
            password: "password".into(),
        };

        register.register(&pool).await.unwrap();

        let duplicate = users::Register {
            username: "alice".into(),
            password: "password".into(),
        };

        let duplicate_token = duplicate.register(&pool).await;

        assert!(duplicate_token.is_err());
    }

    #[tokio::test]
    async fn test_new_folder() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Apply migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let insertable_folder = folders::InsertableFolder {
            nickname: "folder".into(),
            path: "hello/hello".into(),
            watch: true,
            drop_type: folders::DropType::Source,
        };

        let insert_result = insertable_folder.insert(&pool).await.unwrap();

        assert_eq!(insert_result.nickname, "folder");
        assert_eq!(insert_result.path, "hello/hello");
        assert_eq!(insert_result.id, 1);
        assert_eq!(insert_result.watch, true);
        assert_eq!(insert_result.drop_type, folders::DropType::Source);
        pool.close().await;
    }
}
