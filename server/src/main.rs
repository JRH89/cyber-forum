// server/src/main.rs
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use sqlx::{PgPool, query, query_as};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Thread {
    id: String,
    title: String,
    author: String,
    content: String,
    image_url: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NewThread {
    title: String,
    author: String,
    content: String,
    image_url: Option<String>,
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

type Db = PgPool;

#[derive(Debug, Serialize, Deserialize, Clone)]
enum WsMessage {
    NewThread(Thread),
    NewComment { thread_id: String, comment: Comment },
}

type Broadcaster = Arc<RwLock<Vec<actix_ws::Session>>>;

#[get("/threads")]
async fn list_threads(db: web::Data<Db>) -> impl Responder {
    let rows = sqlx::query_as::<_, Thread>(
        r#"SELECT t.id, t.title, u.username as author, t.content, t.image_url, t.created_at
           FROM threads t JOIN users u ON t.user_id = u.id
           ORDER BY t.created_at DESC"#
    )
    .fetch_all(&**db)
    .await
    .unwrap_or_else(|_| vec![]);
    
    HttpResponse::Ok().json(rows)
}
#[post("/threads")]
async fn create_thread(db: web::Data<Db>, broadcaster: web::Data<Broadcaster>, payload: web::Json<NewThread>) -> impl Responder {
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
        r#"INSERT INTO threads (id, title, user_id, content, image_url, created_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#
    )
    .bind(id.clone())
    .bind(payload.title.clone())
    .bind(user_id)
    .bind(payload.content.clone())
    .bind(payload.image_url.clone())
    .bind(created_at.clone())
    .execute(&**db)
    .await;
    
    // Broadcast new thread to WebSocket clients
    let new_thread = Thread {
        id: id.clone(),
        title: payload.title.clone(),
        author: payload.author.clone(),
        content: payload.content.clone(),
        image_url: payload.image_url.clone(),
        created_at: created_at.clone(),
    };
    
    let msg = WsMessage::NewThread(new_thread);
    if let Ok(msg_json) = serde_json::to_string(&msg) {
        let sessions = broadcaster.read().await;
        for session in sessions.iter() {
            let _ = session.text(msg_json.clone()).await;
        }
    }
    
    HttpResponse::Created().finish()
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

#[get("/ws")]
async fn websocket_index(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut session = actix_ws::Session::new(req, stream)?;
    let broadcaster = broadcaster.clone();
    
    // Add this session to the broadcaster list
    broadcaster.write().await.push(session.clone());
    
    // Keep the connection alive
    actix_ws::handle(&mut session, |msg| async move {
        match msg {
            Message::Ping(_) => Ok(Message::Pong(vec![])),
            _ => Ok(Message::Close(None)),
        }
    }).await?;
    
    // Remove session from broadcaster when disconnected
    broadcaster.write().await.retain(|s| s.id() != session.id());
    
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to Postgres");
    
    // Create broadcaster for WebSocket connections
    let broadcaster: Broadcaster = Arc::new(RwLock::new(Vec::new()));

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
        r#"CREATE TABLE IF NOT EXISTS threads (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                user_id TEXT NOT NULL,
                content TEXT NOT NULL,
                image_url TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
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
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(broadcaster.clone()))
            .service(list_threads)
            .service(create_thread)
            .service(list_comments)
            .service(create_comment)
            .service(websocket_index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
