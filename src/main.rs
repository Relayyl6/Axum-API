use axum::{
    extract::{
        Path, State
    },
    http::StatusCode,
    routing::{
        get, post
    },
    Json
};
use serde::{ Deserialize, Serialize };
use sqlx::{ postgres::PgPoolOptions, FromRow, PgPool };
use std::env;
use sqlx::Error;


#[derive(Deserialize)]
struct UserPayload {
    username: String,
    email: String,
}

#[derive(Serialize, FromRow)]
struct User {
    id: i32,
    username: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let app = axum::Router::new()
        .route("/", get(root))
        .route("/users", post(create_user).get(list_users))
        .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on port 8000");
    axum::serve(listener, app).await.unwrap();
}

//endpoint handler
async fn root() -> &'static str {
    "Welcome to the User Management API"
}

//list the available users in the database
async fn list_users(
    State(pool): State<PgPool>
) -> Result<Json<Vec<User>>, StatusCode> {
    sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// create user using th euser payload
async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<UserPayload>    
) -> Result<(StatusCode, Json<User>), StatusCode> {
    sqlx::query_as::<_, User>("INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *")
        .bind(payload.username)
        .bind(payload.email)
        .fetch_one(&pool).await
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|err| {
            if let Error::Database(db_err) = &err {
                if db_err.constraint() == Some("users_username_key") {
                    return StatusCode::CONFLICT; // 409
                }
            }
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

//get user by id
async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>
) -> Result<Json<User>, StatusCode> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool).await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UserPayload>
) -> Result<Json<User>, StatusCode> {
    sqlx::query_as::<_, User>("UPDATE users SET username = $1, email = $2 WHERE id = $3 RETURNING *")
        .bind(payload.username)
        .bind(payload.email)
        .bind(id)
        .fetch_one(&pool).await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>
) -> StatusCode {
    match sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool).await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    StatusCode::NO_CONTENT
                } else {
                    StatusCode::NOT_FOUND
                }
            },
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
}