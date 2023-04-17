use anyhow::Context;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing;
use axum::Extension;
use axum::Json;
use axum::Router;
use axum::Server;
use serde::Deserialize;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::FromRow;
use sqlx_crud::Crud;
use sqlx_crud::SqlxCrud;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(FromRow, Deserialize, Serialize, SqlxCrud)]
struct Task {
    pub id: i64,
    pub task: String,
}

async fn tasks(Extension(pool): Extension<SqlitePool>) -> Response {
    match Task::all(&pool).await {
        Ok(tasks) => (StatusCode::OK, Json(tasks)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn new_task(Extension(pool): Extension<SqlitePool>, Json(new_task): Json<Task>) -> Response {
    match new_task.create(&pool).await {
        Ok(r) => (StatusCode::OK, Json(r)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn task(Path(task_id): Path<i64>, Extension(pool): Extension<SqlitePool>) -> Response {
    match Task::by_id(&pool, task_id).await {
        Ok(Some(task)) => (StatusCode::OK, Json(task)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn update_task(
    Path(task_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    Json(mut task): Json<Task>,
) -> Response {
    if let Ok(Some(_)) = Task::by_id(&pool, task_id).await {
        task.id = task_id;
        match task.update(&pool).await {
            Ok(r) => (StatusCode::OK, Json(r)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    } else {
        (StatusCode::NOT_ACCEPTABLE).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://rust.db")
        .await
        .context("could not connect to database")?;

    let app = Router::new()
        .route("/tasks", routing::get(tasks))
        .route("/tasks", routing::post(new_task))
        .route("/tasks/:id", routing::put(update_task))
        .route("/tasks/:id", routing::get(task))
        .layer(Extension(pool));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
