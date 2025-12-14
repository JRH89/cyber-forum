// src/api.rs
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use base64::Engine;

// Base URL of the server
const BASE_URL: &str = "https://cyber-forum.onrender.com";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub author: String,
    pub content: String,
    pub image_url: Option<String>,
    pub category_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewThread {
    pub title: String,
    pub author: String,
    pub content: String,
    pub image_url: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub thread_id: String,
    pub author: String,
    pub content: String,
    pub image_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewComment {
    pub thread_id: String,
    pub author: String,
    pub content: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

fn client() -> Client {
    Client::new()
}

pub async fn list_threads() -> Result<Vec<Thread>> {
    let resp = client()
        .get(&format!("{}/threads", BASE_URL))
        .send()
        .await?;
    let threads = resp.json::<Vec<Thread>>().await?;
    Ok(threads)
}

pub async fn create_thread(new: NewThread) -> Result<()> {
    client()
        .post(&format!("{}/threads", BASE_URL))
        .json(&new)
        .send()
        .await?;
    Ok(())
}

pub async fn list_comments(thread_id: &str) -> Result<Vec<Comment>> {
    let resp = client()
        .get(&format!("{}/threads/{}/comments", BASE_URL, thread_id))
        .send()
        .await?;
    let comments = resp.json::<Vec<Comment>>().await?;
    Ok(comments)
}

pub async fn create_comment(new: NewComment) -> Result<()> {
    client()
        .post(&format!("{}/comments", BASE_URL))
        .json(&new)
        .send()
        .await?;
    Ok(())
}

pub fn create_data_url(image_path: &str) -> Result<String> {
    let image_data = std::fs::read(image_path)?;
    let mime_type = match std::path::Path::new(image_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png", // default
    };
    
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_data);
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

pub async fn list_categories() -> Result<Vec<Category>> {
    let resp = client()
        .get(&format!("{}/categories", BASE_URL))
        .send()
        .await?;
    let categories = resp.json::<Vec<Category>>().await?;
    Ok(categories)
}

pub async fn create_category(name: String, description: Option<String>) -> Result<()> {
    let payload = serde_json::json!({
        "name": name,
        "description": description
    });
    
    client()
        .post(&format!("{}/categories", BASE_URL))
        .json(&payload)
        .send()
        .await?;
    Ok(())
}
