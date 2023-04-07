use anyhow::Context;
use axum::Extension;
use axum::Json;
use axum::Router;
use axum::Server;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx_crud::SqlxCrud;
use sqlx_crud::Crud;

use std::net::SocketAddr;

#[derive(FromRow, Deserialize, Serialize, SqlxCrud)]
struct Task {
    pub id: i64,
    pub task: String,
}

impl Default for Task {
    fn default() -> Self {
        Self { id: 0, task: "".to_string() }
    }
}

async fn task(Path(task_id): Path<i32>, Extension(pool): Extension<SqlitePool>) ->  impl IntoResponse {
    let task = sqlx::query_as!(Task, r#"
        SELECT tasks.id, tasks.task
        FROM tasks
        WHERE tasks.id = ?"#, task_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    (StatusCode::OK, Json(task))
}

async fn task_sqlx_crud(Path(task_id): Path<i64>, Extension(pool): Extension<SqlitePool>) ->  impl IntoResponse {
    match Task::by_id(&pool, task_id).await {
        Ok(Some(task)) => (StatusCode::OK, Json(task)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn tasks(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let tasks = sqlx::query_as!(Task,
        r#"SELECT tasks.id, tasks.task
        FROM tasks"#)
        .fetch_all(&pool)
        .await
        .unwrap();
    (StatusCode::OK, Json(tasks))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://rust.db")
        .await
        .context("could not connect to database")?;

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/tasks", routing::get(tasks))
        .route("/tasks/:id", routing::get(task))
        .route("/tasks/:id/crud", routing::get(task_sqlx_crud))
        .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
