mod handlers;
use axum::routing::{get, post, put, delete, Router};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let app = Router::new()
        .route("/", get(handlers::health_check))
        .route("/tasks", post(handlers::create_task))
        .route("/tasks", get(handlers::read_tasks))
        .route("/tasks/:id", put(handlers::update_task))
        .route("/tasks/:id", delete(handlers::delete_task))
        .with_state(pool);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}