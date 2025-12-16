// terminal_server.rs
use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::Arc;
use sqlx::{PgPool, Row};
use serde_json;
use anyhow::Result;

#[get("/terminal")]
pub async fn terminal_page() -> impl Responder {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Arch Forum Terminal</title>
    <style>
        body { background: #000; color: #0f0; font-family: monospace; padding: 20px; }
        .terminal { border: 1px solid #0f0; padding: 10px; height: 400px; overflow-y: auto; }
        input { background: #000; color: #0f0; border: 1px solid #0f0; font-family: monospace; width: 100%; }
    </style>
</head>
<body>
    <h1>Arch Forum SSH Terminal</h1>
    <div id="terminal" class="terminal">
        Welcome to Arch Forum Terminal<br>
        Commands: list, post, help, quit<br>
        $ 
    </div>
    <input type="text" id="input" onkeypress="handleInput(event)" placeholder="Enter command...">
    
    <script>
        function handleInput(event) {
            if (event.key === 'Enter') {
                const input = document.getElementById('input');
                const terminal = document.getElementById('terminal');
                const command = input.value.trim();
                
                terminal.innerHTML += command + '<br>';
                input.value = '';
                
                fetch('/terminal/cmd', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({cmd: command})
                })
                .then(r => r.text())
                .then(response => {
                    terminal.innerHTML += response + '<br>$ ';
                    terminal.scrollTop = terminal.scrollHeight;
                });
            }
        }
    </script>
</body>
</html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[post("/terminal/cmd")]
pub async fn handle_command(db: web::Data<Arc<PgPool>>, payload: web::Json<serde_json::Value>) -> impl Responder {
    let cmd = payload.get("cmd").and_then(|v| v.as_str()).unwrap_or("");
    
    match cmd {
        "list" => {
            if let Ok(threads) = get_threads_from_db(&db).await {
                let mut result = String::new();
                for (i, thread) in threads.iter().take(5).enumerate() {
                    result.push_str(&format!("[{}] {}\n", i + 1, thread.0));
                }
                result
            } else {
                "Error loading threads".to_string()
            }
        }
        "help" => "Commands: list, post <title>, help, quit".to_string(),
        "quit" => "Goodbye!".to_string(),
        cmd if cmd.starts_with("post ") => {
            let title = &cmd[5..];
            if create_thread_in_db(&db, title, "Posted from terminal", "terminal_user").await.unwrap_or(false) {
                format!("Thread '{}' created!", title)
            } else {
                "Error creating thread".to_string()
            }
        }
        _ => "Unknown command. Type 'help'".to_string(),
    }
}

async fn get_threads_from_db(pool: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query("SELECT title, author FROM threads ORDER BY created_at DESC LIMIT 10")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|row| Ok((
            row.get::<String, _>("title"),
            row.get::<String, _>("author")
        )))
        .collect()
}

async fn create_thread_in_db(pool: &PgPool, title: &str, content: &str, author: &str) -> Result<bool, sqlx::Error> {
    use uuid::Uuid;
    use chrono::Utc;
    
    let thread_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    // Create user if needed
    let user_id = Uuid::new_v4().to_string();
    let _ = sqlx::query("INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4) ON CONFLICT (username) DO NOTHING")
        .bind(&user_id)
        .bind(author)
        .bind("")
        .bind(&created_at)
        .execute(pool)
        .await;
    
    // Get actual user ID
    if let Ok(user_row) = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(author)
        .fetch_one(pool)
        .await {
        let actual_user_id: String = user_row.get("id");
        
        // Create thread
        Ok(sqlx::query("INSERT INTO threads (id, title, user_id, content, created_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(&thread_id)
            .bind(title)
            .bind(&actual_user_id)
            .bind(content)
            .bind(&created_at)
            .execute(pool)
            .await.is_ok())
    } else {
        Ok(false)
    }
}
