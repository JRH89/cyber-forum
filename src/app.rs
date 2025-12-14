// src/app.rs
use crate::api::{self, Thread, NewThread, NewComment, User, Comment, Category};
// use crate::models::{User, Comment};
// use ratatui::widgets::ListState;
use uuid::Uuid;
use chrono;

#[derive(PartialEq)]
pub enum AppState {
    Login,
    Forum,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CurrentFocus {
    Username,
    Password,
    ThreadList,
    Categories,
    Conversation,
    NewThread,
    Reply,
    ThreadImage,
    ReplyImage,
}

pub struct App {
    pub state: AppState,
    pub focus: CurrentFocus,
    pub should_quit: bool,
    
    // Login state
    pub username_input: String,
    pub password_input: String,
    pub current_user: Option<User>,
    
    // Forum state
    pub threads: Vec<Thread>,
    pub categories: Vec<Category>,
    pub selected_thread: usize,
    pub selected_category: usize,
    pub selected_comment: usize,
    pub current_thread_id: Option<String>,
    pub comments: Vec<Comment>, // Store comments for the open thread
    
    // UI buffers for creating threads and replies
    pub new_thread_title: String,
    pub new_thread_content: String,
    pub new_thread_image_path: String,
    pub reply_content: String,
    pub reply_image_path: String,
    // Sub‑focus within NewThread mode (Title vs Content)
    pub new_thread_focus: CurrentFocus,
    
    // Auto-refresh timer
    pub last_refresh: std::time::Instant,
}

impl App {
    pub fn new() -> App {
        App {
            state: AppState::Login,
            focus: CurrentFocus::Username,
            should_quit: false,
            username_input: String::new(),
            password_input: String::new(),
            current_user: None,
            threads: Vec::new(),
            categories: Vec::new(),
            selected_thread: 0,
            selected_category: 0,
            selected_comment: 0,
            current_thread_id: None,
            comments: Vec::new(),
            new_thread_title: String::new(),
            new_thread_content: String::new(),
            new_thread_image_path: String::new(),
            reply_content: String::new(),
            reply_image_path: String::new(),
            new_thread_focus: CurrentFocus::Username, // reuse enum for sub‑focus (Title)
            last_refresh: std::time::Instant::now(),
        }
    }

    pub fn load_config(&mut self) {
        if let Ok(home) = std::env::var("HOME") {
            let config_path = std::path::Path::new(&home).join(".config/ternimal/config.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let (Some(u), Some(p)) = (json.get("username").and_then(|v| v.as_str()), json.get("password").and_then(|v| v.as_str())) {
                            self.username_input = u.to_string();
                            self.password_input = p.to_string();
                        }
                    }
                }
            }
        }
    }
    
    pub async fn login(&mut self) -> anyhow::Result<()> {
        // Check if username is available
        let username_available = api::check_username_available(&self.username_input).await?;
        
        if !username_available {
            return Err(anyhow::anyhow!("Username '{}' is already taken", self.username_input));
        }
        
        // Register the user
        let user = api::register_user(&self.username_input).await?;
        self.current_user = Some(user);
        self.state = AppState::Forum;
        self.focus = CurrentFocus::ThreadList;
        
        // Load threads with error handling
        if let Err(e) = self.load_threads().await {
            eprintln!("Failed to load threads: {}", e);
            // Continue anyway - user can try again
        }
        
        // Load categories
        if let Err(e) = self.load_categories().await {
            eprintln!("Failed to load categories: {}", e);
        }
        
        Ok(())
    }

    pub async fn load_categories(&mut self) -> anyhow::Result<()> {
        self.categories = api::list_categories().await?;
        Ok(())
    }

    pub async fn auto_refresh(&mut self) {
        // Auto-refresh every 10 seconds
        if self.last_refresh.elapsed().as_secs() >= 10 {
            if let Err(e) = self.load_threads().await {
                eprintln!("Auto-refresh failed: {}", e);
            } else {
                self.last_refresh = std::time::Instant::now();
            }
            
            // Also refresh comments if we have a thread open
            if let Some(thread_id) = self.current_thread_id.clone() {
                if let Err(e) = self.refresh_comments(&thread_id).await {
                    eprintln!("Failed to refresh comments: {}", e);
                }
            }
        }
    }

    pub async fn refresh_comments(&mut self, thread_id: &str) -> anyhow::Result<()> {
        self.comments = api::list_comments(thread_id).await?;
        Ok(())
    }

    pub async fn load_threads(&mut self) -> anyhow::Result<()> {
        let threads = api::list_threads().await?;
        self.threads = threads;
        Ok(())
    }

    pub async fn open_thread(&mut self, index: usize) -> anyhow::Result<()> {
        if let Some(thread) = self.threads.get(index) {
            self.current_thread_id = Some(thread.id.clone());
            self.comments = api::list_comments(&thread.id).await?;
            self.focus = CurrentFocus::Conversation;
            self.selected_comment = 0;
        }
        Ok(())
    }

    pub fn get_current_thread(&self) -> Option<&Thread> {
        self.current_thread_id.as_ref().and_then(|id| {
            self.threads.iter().find(|t| &t.id == id)
        })
    }

    pub async fn create_thread(&mut self, title: String, content: String) -> anyhow::Result<()> {
        if let Some(user) = &self.current_user {
            let image_url = if !self.new_thread_image_path.is_empty() {
                Some(api::create_data_url(&self.new_thread_image_path)?)
            } else {
                None
            };
            
            let category_id = if self.categories.is_empty() {
                None
            } else {
                Some(self.categories[self.selected_category].id.clone())
            };
            
            let new_thread = NewThread {
                title,
                author: user.username.clone(),
                content,
                image_url,
                category_id,
            };
            api::create_thread(new_thread).await?;
        }
        Ok(())
    }

    pub async fn create_reply(&mut self, content: String) -> anyhow::Result<()> {
        if let (Some(user), Some(thread_id)) = (&self.current_user, &self.current_thread_id) {
            let image_url = if !self.reply_image_path.is_empty() {
                Some(api::create_data_url(&self.reply_image_path)?)
            } else {
                None
            };
            
            let new_comment = NewComment {
                thread_id: thread_id.clone(),
                author: user.username.clone(),
                content,
                image_url,
            };
            api::create_comment(new_comment).await?;
            // Refresh comments
            self.comments = api::list_comments(thread_id).await?;
        }
        Ok(())
    }
}
