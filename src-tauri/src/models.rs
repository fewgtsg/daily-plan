use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub id: i64,
    pub date: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub quadrant: i32,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntryTask {
    pub id: i64,
    pub entry_id: i64,
    pub task_id: i64,
    pub linked_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub result_type: String,
    pub id: i64,
    pub date: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagDto {
    pub id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub usage_count: i64,
}
