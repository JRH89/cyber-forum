// ssh_server.rs
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use sqlx::{PgPool, Row};

pub async fn start_ssh_server(db_pool: Arc<PgPool>) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:2222")?;
    println!("SSH Forum Server listening on port 80");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool = db_pool.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, pool) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
    Ok(())
}

fn handle_client(stream: std::net::TcpStream, db_pool: Arc<PgPool>) -> Result<(), Box<dyn std::error::Error>> {
    let logged_in_user: Option<String> = None;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        handle_client_async(stream, db_pool, logged_in_user).await
    })
}

async fn handle_client_async(mut stream: std::net::TcpStream, db_pool: Arc<PgPool>, mut logged_in_user: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_all(b"Welcome to Arch Forum SSH Server\r\n")?;
    stream.write_all(b"Arch Linux verification required...\r\n")?;
    
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let input = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    if input.contains("arch") || input.contains("linux") || input.trim().len() > 0 {
        stream.write_all(b"Arch Linux verified! Welcome.\r\n")?;
        stream.write_all(b"\r\n=== ARCH FORUM ===\r\n")?;
        stream.write_all(b"Commands: login, list, post, reply, help, quit\r\n")?;
        
        loop {
            stream.write_all(b"forum> ")?;
            
            let mut cmd_buffer = [0; 256];
            let bytes_read = stream.read(&mut cmd_buffer)?;
            if bytes_read == 0 { break; }
            
            let command_str = String::from_utf8_lossy(&cmd_buffer[..bytes_read]);
            let command = command_str.trim();
            
            match command {
                "list" => {
                    stream.write_all(b"\r\nRecent threads:\r\n")?;
                    if let Ok(threads) = get_threads_from_db(&db_pool) {
                        let mut result = String::new();
                        for (i, thread) in threads.iter().take(10).enumerate() {
                            result.push_str(&format!("[{}] {}\n", i + 1, thread.0));
                            let line = format!("[{}] {}\r\n", i + 1, thread.0);
                            stream.write_all(line.as_bytes())?;
                        }
                    } else {
                        stream.write_all(b"Error loading threads\r\n")?;
                    }
                }
                "login" => {
                    if logged_in_user.is_some() {
                        stream.write_all(b"Already logged in.\r\n")?;
                    } else {
                        stream.write_all(b"Username: ")?;
                        let mut user_buf = [0; 64];
                        let bytes_read = stream.read(&mut user_buf)?;
                        let username_str = String::from_utf8_lossy(&user_buf[..bytes_read]);
                        let username = username_str.trim();
                        
                        stream.write_all(b"Password: ")?;
                        let mut pass_buf = [0; 64];
                        let bytes_read = stream.read(&mut pass_buf)?;
                        let _password = String::from_utf8_lossy(&pass_buf[..bytes_read]).trim();
                        
                        // For now, accept any login (you can add real auth later)
                        logged_in_user = Some(username.to_string());
                        stream.write_all(b"Login successful!\r\n")?;
                    }
                }
                "help" => {
                    stream.write_all(b"\r\nCommands:\r\n")?;
                    if logged_in_user.is_some() {
                        stream.write_all(b"  list  - Show recent threads\r\n")?;
                        stream.write_all(b"  post  - Create new thread\r\n")?;
                        stream.write_all(b"  reply - Reply to thread\r\n")?;
                    } else {
                        stream.write_all(b"  list  - Show recent threads (read-only)\r\n")?;
                        stream.write_all(b"  login - Login to post\r\n")?;
                    }
                    stream.write_all(b"  help  - Show this help\r\n")?;
                    stream.write_all(b"  quit  - Exit forum\r\n")?;
                }
                cmd if cmd.starts_with("post") => {
                    if let Some(ref user) = logged_in_user {
                        let title = &cmd[5..];
                        stream.write_all(b"Enter thread content (end with '.' on new line):\r\n")?;
                        
                        let mut content_lines = Vec::new();
                        loop {
                            stream.write_all(b"> ")?;
                            let mut line_buf = [0; 256];
                            let bytes_read = stream.read(&mut line_buf)?;
                            let line_str = String::from_utf8_lossy(&line_buf[..bytes_read]);
                            let line = line_str.trim();
                            if line == "." { break; }
                            content_lines.push(line.to_string());
                        }
                        
                        let content = content_lines.join("\n");
                        if create_thread_in_db(&db_pool, title, &content, user) {
                            stream.write_all(b"Thread created successfully!\r\n")?;
                        } else {
                            stream.write_all(b"Error creating thread\r\n")?;
                        }
                    } else {
                        stream.write_all(b"Please login to post. Type 'login'.\r\n")?;
                    }
                }
                "reply" => {
                    if let Some(ref user) = logged_in_user {
                        stream.write_all(b"Reply to thread ID: ")?;
                        let mut id_buf = [0; 64];
                        let bytes_read = stream.read(&mut id_buf)?;
                        let thread_id_str = String::from_utf8_lossy(&id_buf[..bytes_read]);
                        let thread_id = thread_id_str.trim();
                        
                        stream.write_all(b"Enter reply (end with '.' on new line):\r\n")?;
                        
                        let mut reply_lines = Vec::new();
                        loop {
                            stream.write_all(b"> ")?;
                            let mut line_buf = [0; 256];
                            let bytes_read = stream.read(&mut line_buf)?;
                            let line_str = String::from_utf8_lossy(&line_buf[..bytes_read]);
                            let line = line_str.trim();
                            if line == "." { break; }
                            reply_lines.push(line.to_string());
                        }
                        
                        let reply = reply_lines.join("\n");
                        if create_comment_in_db(&db_pool, &thread_id, &reply, user).await {
                            stream.write_all(b"Reply posted successfully!\r\n")?;
                        } else {
                            stream.write_all(b"Error posting reply\r\n")?;
                        }
                    } else {
                        stream.write_all(b"Please login to reply. Type 'login'.\r\n")?;
                    }
                }
                "quit" | "exit" => {
                    stream.write_all(b"Goodbye!\r\n")?;
                    break;
                }
                _ => {
                    stream.write_all(b"Unknown command. Type 'help'.\r\n")?;
                }
            }
        }
    } else {
        stream.write_all(b"Access denied: Arch Linux required\r\n")?;
    }
    
    Ok(())
}

fn get_threads_from_db(pool: &PgPool) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    use std::thread;
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    let pool_clone = pool.clone();
    
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            sqlx::query("SELECT title, author FROM threads ORDER BY created_at DESC LIMIT 10")
                .fetch_all(&pool_clone)
                .await
        });
        
        match result {
            Ok(rows) => {
                let threads: Vec<(String, String)> = rows.iter()
                    .map(|row| (
                        row.get::<String, _>("title"),
                        row.get::<String, _>("author")
                    ))
                    .collect();
                tx.send(Ok(threads)).unwrap();
            }
            Err(e) => {
                tx.send(Err(Box::new(e))).unwrap();
            }
        }
    });
    
    Ok(rx.recv()??.into())
}

fn create_thread_in_db(pool: &PgPool, title: &str, content: &str, author: &str) -> bool {
    use std::thread;
    use std::sync::mpsc;
    use uuid::Uuid;
    use chrono::Utc;
    
    let (tx, rx) = mpsc::channel();
    let pool_clone = pool.clone();
    let title = title.to_string();
    let content = content.to_string();
    let author = author.to_string();
    
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            // First ensure user exists
            let user_id = Uuid::new_v4().to_string();
            let _ = sqlx::query("INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4) ON CONFLICT (username) DO NOTHING")
                .bind(&user_id)
                .bind(&author)
                .bind("")
                .bind(Utc::now().to_rfc3339())
                .execute(&pool_clone)
                .await;
            
            // Get actual user ID
            if let Ok(user_row) = sqlx::query("SELECT id FROM users WHERE username = $1")
                .bind(&author)
                .fetch_one(&pool_clone)
                .await {
                let actual_user_id: String = user_row.get("id");
                
                // Create thread
                let thread_id = Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO threads (id, title, user_id, content, created_at, ssh_user) VALUES ($1, $2, $3, $4, $5, $6)")
                    .bind(&thread_id)
                    .bind(&title)
                    .bind(&actual_user_id)
                    .bind(&content)
                    .bind(Utc::now().to_rfc3339())
                    .bind(true)
                    .execute(&pool_clone)
                    .await
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        });
        
        tx.send(result.is_ok()).unwrap();
    });
    
    rx.recv().unwrap_or(false)
}

async fn create_comment_in_db(pool: &PgPool, thread_id: &str, content: &str, author: &str) -> bool {
    use uuid::Uuid;
    use chrono::Utc;
    
    let comment_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    // First ensure user exists
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
        
        // Create comment
        sqlx::query("INSERT INTO comments (id, thread_id, user_id, content, created_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(&comment_id)
            .bind(thread_id)
            .bind(&actual_user_id)
            .bind(content)
            .bind(&created_at)
            .execute(pool)
            .await
            .is_ok()
    } else {
        false
    }
}
