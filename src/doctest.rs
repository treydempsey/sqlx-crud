use sqlx::FromRow;
use sqlx::SqlitePool;
use sqlx_crud_macros::SqlxCrud;

#[derive(Debug, FromRow, SqlxCrud)]
pub struct User {
   pub user_id: i32,
   pub name: String,
}

pub async fn setup() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("a database connection");

    sqlx::query("CREATE TABLE users (user_id INTEGER NOT NULL, name TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("successful");

    sqlx::query("INSERT INTO users (user_id, name) VALUES(?, ?)")
        .bind(1)
        .bind("test")
        .execute(&pool)
        .await
        .expect("successful");

    pool
}
