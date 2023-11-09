use axum::{extract, http};
use serde::{Serialize, Deserialize};
use sqlx::{self};
use sqlx::encode::IsNull;
use sqlx::postgres::PgPool;

#[derive(Serialize)]
pub struct Task {
    id: uuid::Uuid,
    title: String,
    description: String,
    status: Status,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
enum Status {
    Todo,
    Doing,
    Done,
}
impl sqlx::Type<sqlx::Postgres> for Status {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // Specify the SQL type information for Status
        sqlx::postgres::PgTypeInfo::with_name("status")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for Status {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> IsNull {
        // Implement how to encode Status to Postgres
        match self {
            Status::Todo => buf.extend(b"todo"),
            Status::Doing => buf.extend(b"doing"),
            Status::Done => buf.extend(b"done"),
        }
        IsNull::No
    }
}


impl Task {
    fn new(title: String, description: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            title,
            description,
            status: Status::Todo,
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
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, title, description, status, created_at, updated_at
        "#,
    )
    .bind(&task.id)
    .bind(&task.title)
    .bind(&task.description)
    .bind(&task.status)
    .bind(&task.created_at)
    .bind(&task.updated_at)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(task))),
        Err(_) => Err(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}