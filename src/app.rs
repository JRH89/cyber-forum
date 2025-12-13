// src/app.rs
use crate::api::{self, Thread, NewThread, NewComment, User, Comment};
// use crate::models::{User, Comment};
// use ratatui::widgets::ListState;

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
    Conversation,
    NewThread,
    Reply,
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
    pub selected_thread: usize,
    pub selected_comment: usize,
    pub current_thread_id: Option<String>,
    pub comments: Vec<Comment>, // Store comments for the open thread
    
    // UI buffers for creating threads and replies
    pub new_thread_title: String,
    pub new_thread_content: String,
    pub reply_content: String,
    // Sub‑focus within NewThread mode (Title vs Content)
    pub new_thread_focus: CurrentFocus,
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
            selected_thread: 0,
            selected_comment: 0,
            current_thread_id: None,
            comments: Vec::new(),
            new_thread_title: String::new(),
            new_thread_content: String::new(),
            reply_content: String::new(),
            new_thread_focus: CurrentFocus::Username, // reuse enum for sub‑focus (Title)
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
        // For now, we just assume login is successful if fields are non-empty
        // In a real app, we'd hit an auth endpoint.
        // We'll create a dummy user object.
        self.current_user = Some(User {
            id: "local-session".to_string(),
            username: self.username_input.clone(),
            password_hash: "".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        self.state = AppState::Forum;
        self.focus = CurrentFocus::ThreadList;
        self.load_threads().await?;
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
            let new_thread = NewThread {
                title,
                author: user.username.clone(),
                content,
            };
            api::create_thread(new_thread).await?;
        }
        Ok(())
    }

    pub async fn create_reply(&mut self, content: String) -> anyhow::Result<()> {
        if let (Some(user), Some(thread_id)) = (&self.current_user, &self.current_thread_id) {
            let new_comment = NewComment {
                thread_id: thread_id.clone(),
                author: user.username.clone(),
                content,
            };
            api::create_comment(new_comment).await?;
            // Refresh comments
            self.comments = api::list_comments(thread_id).await?;
        }
        Ok(())
    }
}
