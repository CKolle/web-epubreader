use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use utoipa::ToSchema;

extern crate rand_core;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[sqlx(rename = "password")]
    pub hashed_password: String,
}

impl User {
    pub async fn find_by_id(id: i32, pool: &Pool<Sqlite>) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(
        username: &str,
        pool: &Pool<Sqlite>,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user: Option<User> = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}

pub struct InsertableUser {
    pub username: String,
    pub hashed_password: String,
}

impl InsertableUser {
    async fn insert(self, pool: &Pool<Sqlite>) -> Result<User, sqlx::Error> {
        let Self {
            username,
            hashed_password,
        } = self;

        let result = sqlx::query(
            r#"
            INSERT INTO users ( username, password ) VALUES ( $1, $2 )
            RETURNING id
            "#,
        )
        .bind(&username)
        .bind(&hashed_password)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get("id");

        Ok(User {
            id,
            username,
            hashed_password,
        })
    }
}

pub fn hash_password(password: &str) -> String {
    let salt_string = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .unwrap()
        .to_string();

    password_hash
}

pub fn verify_password<'a>(hashed_password: &'a str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hashed_password).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

struct InsertableBearerToken {
    user_id: i32,
    token: String,
}

impl InsertableBearerToken {
    pub fn new(user_id: i32) -> Self {
        let token = uuid::Uuid::new_v4().to_string();
        Self { user_id, token }
    }

    pub async fn insert(self, pool: &Pool<Sqlite>) -> Result<BearerToken, sqlx::Error> {
        let Self { user_id, token } = self;

        let result = sqlx::query(
            r#"
            INSERT INTO bearer_tokens ( user_id, token ) VALUES ( $1, $2 )
            RETURNING id
            "#,
        )
        .bind(&user_id)
        .bind(&token)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get("id");

        let bearer_token = BearerToken { id, user_id, token };

        Ok(bearer_token)
    }
}

#[derive(sqlx::FromRow)]
pub struct BearerToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
}

impl BearerToken {
    pub async fn find_by_token(
        token: &str,
        pool: &Pool<Sqlite>,
    ) -> Result<Option<Self>, sqlx::Error> {
        let bearer_token: Option<BearerToken> = sqlx::query_as::<_, BearerToken>(
            r#"
            SELECT * FROM bearer_tokens WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(bearer_token)
    }
}

#[derive(Deserialize, ToSchema)]
#[schema(as = Login)]
pub struct Login {
    pub username: String,
    pub password: String,
}

impl Login {
    pub async fn login(self, pool: &Pool<Sqlite>) -> Result<BearerToken, LoginError> {
        let Self { username, password } = self;

        let user_result = User::find_by_username(&username, &pool).await;

        let user = match user_result {
            Ok(Some(user)) => user,
            Ok(None) => return Err(LoginError::UserNotFound),
            Err(error) => return Err(LoginError::DatabaseError(error)),
        };

        if !verify_password(&user.hashed_password, &password) {
            return Err(LoginError::PasswordIncorrect);
        }

        let token: BearerToken = sqlx::query_as::<_, BearerToken>(
            r#"
            SELECT * FROM bearer_tokens WHERE user_id = $1
            "#,
        )
        .bind(user.id)
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            error => LoginError::DatabaseError(error),
        })?;

        Ok(token)
    }
}

#[derive(Debug)]
pub enum LoginError {
    UserNotFound,
    PasswordIncorrect,
    DatabaseError(sqlx::Error),
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = Register)]
pub struct Register {
    pub username: String,
    pub password: String,
}

impl Register {
    pub async fn register(self, pool: &Pool<Sqlite>) -> Result<BearerToken, RegisterError> {
        let Self { username, password } = self;

        let user_result = User::find_by_username(&username, &pool).await;

        match user_result {
            Ok(Some(_)) => return Err(RegisterError::UsernameTaken),
            Ok(None) => (),
            Err(error) => return Err(RegisterError::DatabaseError(error)),
        };

        let hashed_password = hash_password(&password);

        let user = InsertableUser {
            username: username.clone(),
            hashed_password,
        }
        .insert(pool)
        .await
        .map_err(|error| match error {
            error => panic!("Unexpected error: {:?}", error),
        })?;

        let token = InsertableBearerToken::new(user.id)
            .insert(pool)
            .await
            .map_err(|error| match error {
                error => panic!("Unexpected error: {:?}", error),
            })?;

        Ok(token)
    }
}

#[derive(Debug)]
pub enum RegisterError {
    DatabaseError(sqlx::Error),
    UsernameTaken,
}
