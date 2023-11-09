use axum::{extract, http};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};
use sqlx::postgres::PgPool;

#[derive(Serialize, FromRow)]
pub struct Task {
    id: uuid::Uuid,
    title: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    fn new(title: String, description: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            title,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct CreateTask {
    title: String,
    description: String,
}

pub async fn health_check() -> http::StatusCode {
    http::StatusCode::OK
}

pub async fn create_task(
    extract::State(pool): extract::State<PgPool>,
    axum::Json(payload): axum::Json<CreateTask>,
) -> Result<(http::StatusCode, axum::Json<Task>), http::StatusCode> {
    let task = Task::new(payload.title, payload.description);

    let res = sqlx::query(
        r#"
        INSERT INTO tasks (id, title, description, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, title, description, created_at, updated_at
        "#,
    )
    .bind(&task.id)
    .bind(&task.title)
    .bind(&task.description)
    .bind(&task.created_at)
    .bind(&task.updated_at)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(task))),
        Err(_) => Err(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn read_tasks(
    extract::State(pool): extract::State<PgPool>,
) -> Result<axum::Json<Vec<Task>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Task>("SELECT * FROM tasks"
    ).fetch_all(&pool).await;

    match res {
        Ok(tasks) => Ok(axum::Json(tasks)),
        Err(_) => Err(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_task(
    extract::State(pool): extract::State<PgPool>,
    extract::Path(id): extract::Path<uuid::Uuid>,
    axum::Json(payload): axum::Json<CreateTask>,
) -> http::StatusCode {
    let now = chrono::Utc::now();

    let res = sqlx::query(
        r#"
        UPDATE tasks SET title = $1, description = $2, updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await
    .map(|res| match res.rows_affected() {
        0 => http::StatusCode::NOT_FOUND,
        _ => http::StatusCode::OK,
    });

    match res {
        Ok(status) => status,
        Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}