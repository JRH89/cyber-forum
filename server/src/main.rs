// server/src/main.rs
mod ssh_server;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use chrono::Utc;
use sqlx::{PgPool, query, query_as};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures_util::StreamExt;

#[get("/ws")]
async fn websocket_index(req: actix_web::HttpRequest,
                         stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let res = actix_ws::handle(req, stream);
    let (response, session, mut msg_stream) = res?;
    
    // WebSocket session loop
    while let Some(msg_result) = msg_stream.next().await {
        match msg_result {
            Ok(Message::Ping(_)) => {
                let _ = session.pong(&[]).await;
            }
            Ok(Message::Text(text)) => {
                // Echo back or handle commands
                let _ = session.text(format!("Echo: {}", text)).await;
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }
    
    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to Postgres");
    
    // Start SSH server in background
    let ssh_pool = pool.clone();
    let ssh_handle = tokio::spawn(async {
        if let Err(e) = ssh_server::start_ssh_server(Arc::new(ssh_pool)).await {
            eprintln!("SSH server error: {}", e);
        }
    });
    
    // Run simple migrations to ensure tables exist (executed once at startup)
    let _ = sqlx::query(
        r#"INSERT INTO users (id, username, password_hash, created_at)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (username) DO NOTHING"#
    )
    .bind(Uuid::new_v4().to_string())
    .bind("admin")
    .bind("")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await;
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(broadcaster.clone()))
            .service(list_threads)
            .service(create_thread)
            .service(list_categories)
            .service(create_category)
            .service(check_username)
            .service(register_user)
            .service(list_comments)
            .service(create_comment)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

type Db = PgPool;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Thread {
    id: String,
    title: String,
    author: String,
    content: String,
    image_url: Option<String>,
    category_id: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Category {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NewThread {
    title: String,
    author: String,
    content: String,
    image_url: Option<String>,
    category_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Comment {
    id: String,
    thread_id: String,
    author: String,
    content: String,
    image_url: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NewComment {
    thread_id: String,
    author: String,
    content: String,
    image_url: Option<String>,
}

#[get("/threads")]
async fn list_threads(db: web::Data<Db>) -> impl Responder {
    let rows = sqlx::query_as::<_, Thread>(
        r#"SELECT t.id, t.title, u.username as author, t.content, t.image_url, t.category_id, t.created_at
           FROM threads t JOIN users u ON t.user_id = u.id
           ORDER BY t.created_at DESC"#
    )
    .fetch_all(&**db)
    .await
    .unwrap_or_else(|_| vec![]);
    
    HttpResponse::Ok().json(rows)
}

#[post("/threads")]
async fn create_thread(db: web::Data<Db>, payload: web::Json<NewThread>) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    // Insert user if not exists (ON CONFLICT DO NOTHING)
    let _ = sqlx::query(
        r#"INSERT INTO users (id, username, password_hash, created_at)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (username) DO NOTHING"#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(payload.author.clone())
    .bind("")
    .bind(created_at.clone())
    .execute(&**db)
    .await;
    // Retrieve user id
    let user_row: (String,) = sqlx::query_as(
        r#"SELECT id FROM users WHERE username = $1"#
    )
    .bind(payload.author.clone())
    .fetch_one(&**db)
    .await
    .unwrap();
    let user_id = user_row.0;
    // Insert thread
    let _ = sqlx::query(
        r#"INSERT INTO threads (id, title, user_id, content, image_url, category_id, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#
    )
    .bind(id.clone())
    .bind(payload.title.clone())
    .bind(user_id)
    .bind(payload.content.clone())
    .bind(payload.image_url.clone())
    .bind(payload.category_id.clone())
    .bind(created_at.clone())
    .execute(&**db)
    .await;
    
    HttpResponse::Created().finish()
}

#[get("/categories")]
async fn list_categories(db: web::Data<Db>) -> impl Responder {
    let rows = sqlx::query_as::<_, Category>(
        r#"SELECT id, name, description, created_at
           FROM categories ORDER BY name ASC"#
    )
    .fetch_all(&**db)
    .await
    .unwrap_or_else(|_| vec![]);
    
    HttpResponse::Ok().json(rows)
}

#[post("/categories")]
async fn create_category(db: web::Data<Db>, payload: web::Json<serde_json::Value>) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("General");
    let description = payload.get("description").and_then(|v| v.as_str());
    
    let _ = sqlx::query(
        r#"INSERT INTO categories (id, name, description, created_at)
           VALUES ($1, $2, $3, $4)"#
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(created_at)
    .execute(&**db)
    .await;
    
    HttpResponse::Created().finish()
}

#[get("/auth/check-username/{username}")]
async fn check_username(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    let username = path.into_inner();
    let exists = sqlx::query(
        r#"SELECT COUNT(*) as count FROM users WHERE username = $1"#
    )
    .bind(&username)
    .fetch_one(&**db)
    .await
    .map(|row: (i64,)| row.0 > 0)
    .unwrap_or(false);
    
    HttpResponse::Ok().json(serde_json::json!({
        "available": !exists,
        "username": username
    }))
}

#[post("/auth/register")]
async fn register_user(db: web::Data<Db>, payload: web::Json<serde_json::Value>) -> impl Responder {
    let username = payload.get("username").and_then(|v| v.as_str()).unwrap_or("");
    
    // Check if username already exists
    let exists = sqlx::query(
        r#"SELECT COUNT(*) as count FROM users WHERE username = $1"#
    )
    .bind(username)
    .fetch_one(&**db)
    .await
    .map(|row: (i64,)| row.0 > 0)
    .unwrap_or(false);
    
    if exists {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Username already taken"
        }));
    }
    
    // Create new user
    let user_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    let _ = sqlx::query(
        r#"INSERT INTO users (id, username, password_hash, created_at)
           VALUES ($1, $2, $3, $4)"#
    )
    .bind(&user_id)
    .bind(username)
    .bind("") // No password
    .bind(&created_at)
    .execute(&**db)
    .await;
    
    HttpResponse::Created().json(serde_json::json!({
        "id": user_id,
        "username": username,
        "created_at": created_at
    }))
}

#[get("/threads/{id}/comments")]
async fn list_comments(db: web::Data<Db>, path: web::Path<String>) -> impl Responder {
    let thread_id = path.into_inner();
    let rows = sqlx::query_as::<_, Comment>(
        r#"SELECT c.id, $1 as thread_id, u.username as author, c.content, c.image_url, c.created_at
           FROM comments c JOIN users u ON c.user_id = u.id
           WHERE c.thread_id = $2 ORDER BY c.created_at ASC"#
    )
    .bind(thread_id.clone())
    .bind(thread_id)
    .fetch_all(&**db)
    .await
    .unwrap_or_else(|_| vec![]);
    HttpResponse::Ok().json(rows)
}

#[post("/comments")]
async fn create_comment(db: web::Data<Db>, payload: web::Json<NewComment>) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    // Ensure user exists (ON CONFLICT DO NOTHING)
    let _ = sqlx::query(
        r#"INSERT INTO users (id, username, password_hash, created_at)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (username) DO NOTHING"#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(payload.author.clone())
    .bind("")
    .bind(created_at.clone())
    .execute(&**db)
    .await;
    // Retrieve user id
    let user_row: (String,) = sqlx::query_as(
        r#"SELECT id FROM users WHERE username = $1"#
    )
    .bind(payload.author.clone())
    .fetch_one(&**db)
    .await
    .unwrap();
    let user_id = user_row.0;
    // Insert comment
    let _ = sqlx::query(
        r#"INSERT INTO comments (id, thread_id, user_id, content, image_url, created_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#
    )
    .bind(id)
    .bind(payload.thread_id.clone())
    .bind(user_id)
    .bind(payload.content.clone())
    .bind(payload.image_url.clone())
    .bind(created_at)
    .execute(&**db)
    .await;
    HttpResponse::Created().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to Postgres");
    
    // Start SSH server in background
    let ssh_pool = pool.clone();
    let ssh_handle = tokio::spawn(async {
        if let Err(e) = ssh_server::start_ssh_server(Arc::new(ssh_pool)).await {
            eprintln!("SSH server error: {}", e);
        }
    });
    
    // Run simple migrations to ensure tables exist (executed once at startup)
    let _ = sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            );"#
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            );"#
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS threads (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                user_id TEXT NOT NULL,
                content TEXT NOT NULL,
                image_url TEXT,
                category_id TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)
            );"#
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                content TEXT NOT NULL,
                image_url TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (thread_id) REFERENCES threads(id),
                FOREIGN KEY (user_id) REFERENCES users(id)
            );"#
    )
    .execute(&pool)
    .await;
    
    // Add image_url columns to existing tables (for backwards compatibility)
    let _ = sqlx::query("ALTER TABLE threads ADD COLUMN IF NOT EXISTS image_url TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE comments ADD COLUMN IF NOT EXISTS image_url TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE threads ADD COLUMN IF NOT EXISTS category_id TEXT")
        .execute(&pool)
        .await;
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(list_threads)
            .service(create_thread)
            .service(list_comments)
            .service(create_comment)
            .service(list_categories)
            .service(create_category)
            .service(check_username)
            .service(register_user)
            .service(websocket_index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
