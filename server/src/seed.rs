use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

pub async fn seed_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("Seeding database with sample content...");
    
    // Clear existing data
    println!("Clearing existing threads and comments...");
    sqlx::query("DELETE FROM comments")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM threads")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await?;
    
    // Create sample users
    let users = vec![
        ("arch_user", "Arch Linux Enthusiast"),
        ("linux_admin", "System Administrator"),
        ("terminal_ninja", "CLI Expert"),
        ("rust_dev", "Rust Programmer"),
    ];
    
    for (username, bio) in users {
        let user_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4)")
            .bind(&user_id)
            .bind(username)
            .bind("hashed_password")
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await?;
    }
    
    // Get user IDs
    let arch_user_id: String = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind("arch_user")
        .fetch_one(pool)
        .await?
        .get("id");
    
    let linux_admin_id: String = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind("linux_admin")
        .fetch_one(pool)
        .await?
        .get("id");
    
    let terminal_ninja_id: String = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind("terminal_ninja")
        .fetch_one(pool)
        .await?
        .get("id");
    
    let rust_dev_id: String = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind("rust_dev")
        .fetch_one(pool)
        .await?
        .get("id");
    
    // Create sample threads
    let threads = vec![
        ("Welcome to TERNIMAL!", "This is the official forum for the TERNIMAL terminal forum client. Feel free to discuss features, report bugs, or share your terminal setups!", &arch_user_id),
        ("Best terminal emulators?", "What's your favorite terminal emulator? I've been using Alacritty lately but curious what others prefer.", &linux_admin_id),
        ("Rust in terminal apps", "Building terminal apps with Rust is amazing! The performance and safety are unmatched. What terminal apps have you built?", &rust_dev_id),
        ("Productivity tips", "Share your best terminal productivity tips! I'll start: tmux + vim + fzf is my holy trinity.", &terminal_ninja_id),
        ("Arch vs other distros", "Why did you choose Arch Linux? Was it the AUR, the rolling release, or something else?", &arch_user_id),
    ];
    
    for (title, content, user_id) in threads {
        let thread_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO threads (id, title, user_id, content, created_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(&thread_id)
            .bind(title)
            .bind(user_id)
            .bind(content)
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await?;
        
        // Add some comments to threads
        if title == "Best terminal emulators?" {
            add_comment(pool, &thread_id, "I'm still using gnome-terminal. It's simple and works well.", &arch_user_id).await?;
            add_comment(pool, &thread_id, "Try Kitty! It's fast and has great GPU acceleration.", &terminal_ninja_id).await?;
            add_comment(pool, &thread_id, "WezTerm is my favorite - cross platform and highly configurable.", &rust_dev_id).await?;
        }
        
        if title == "Rust in terminal apps" {
            add_comment(pool, &thread_id, "I built a file manager in Rust! The compile times are worth it.", &rust_dev_id).await?;
            add_comment(pool, &thread_id, "How's the binary size compared to C?", &linux_admin_id).await?;
        }
    }
    
    println!("Database seeded successfully!");
    println!("Created users: arch_user, linux_admin, terminal_ninja, rust_dev");
    println!("Created 5 threads with sample comments");
    
    Ok(())
}

async fn add_comment(pool: &PgPool, thread_id: &str, content: &str, user_id: &str) -> Result<(), sqlx::Error> {
    let comment_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO comments (id, thread_id, user_id, content, created_at) VALUES ($1, $2, $3, $4, $5)")
        .bind(&comment_id)
        .bind(thread_id)
        .bind(user_id)
        .bind(content)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
    Ok(())
}
