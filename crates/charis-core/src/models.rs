use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardItem {
    pub name: String,
    pub url: String,
    pub server: String,
    pub board: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardCategory {
    pub name: String,
    pub boards: Vec<BoardItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadItem {
    pub id: String,
    pub title: String,
    pub res_count: u32,
    pub ikioi: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostItem {
    pub number: u32,
    pub name: String,
    pub mail: String,
    pub date: String,
    pub id: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadContent {
    pub title: String,
    pub posts: Vec<PostItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkThreadItem {
    pub board: BoardItem,
    pub thread: ThreadItem,
    pub bookmarked_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryThreadItem {
    pub board: BoardItem,
    pub thread: ThreadItem,
    pub visited_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NGMode {
    #[default]
    Abone,
    Transparent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NGSettings {
    #[serde(default)]
    pub ng_words: Vec<String>,
    #[serde(default)]
    pub ng_ids: Vec<String>,
    #[serde(default)]
    pub ng_mode: NGMode,
    #[serde(default = "default_chain_abone")]
    pub chain_abone: bool,
}

fn default_chain_abone() -> bool {
    true
}

impl Default for NGSettings {
    fn default() -> Self {
        Self {
            ng_words: Vec::new(),
            ng_ids: Vec::new(),
            ng_mode: NGMode::default(),
            chain_abone: true,
        }
    }
}
