// src/api.rs
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Base URL of the server
const BASE_URL: &str = "https://cyber-forum.onrender.com";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub author: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewThread {
    pub title: String,
    pub author: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub thread_id: String,
    pub author: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewComment {
    pub thread_id: String,
    pub author: String,
    pub content: String,
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
