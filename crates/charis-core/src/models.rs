use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_ui_font_family")]
    pub ui_font_family: String,
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: u32,
    #[serde(default = "default_post_font_family")]
    pub post_font_family: String,
    #[serde(default = "default_post_font_size")]
    pub post_font_size: u32,
    #[serde(default = "default_post_line_height")]
    pub post_line_height: f64,
    #[serde(default = "default_blur_images")]
    pub default_blur_images: bool,
    #[serde(default = "default_scroll_amount")]
    pub scroll_amount: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_initial_scroll_position")]
    pub initial_scroll_position: String,
    #[serde(default = "default_name")]
    pub default_name: String,
    #[serde(default = "default_mail")]
    pub default_mail: String,
}

fn default_ui_font_family() -> String {
    "'M PLUS 1', 'M PLUS 1p', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif".into()
}

fn default_ui_font_size() -> u32 {
    13
}

fn default_post_font_family() -> String {
    "".into()
}

fn default_post_font_size() -> u32 {
    14
}

fn default_post_line_height() -> f64 {
    1.65
}

fn default_blur_images() -> bool {
    true
}

fn default_scroll_amount() -> u32 {
    120
}

fn default_theme() -> String {
    "dark".into()
}

fn default_initial_scroll_position() -> String {
    "top".into()
}

fn default_name() -> String {
    "".into()
}

fn default_mail() -> String {
    "sage".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_font_family: default_ui_font_family(),
            ui_font_size: default_ui_font_size(),
            post_font_family: default_post_font_family(),
            post_font_size: default_post_font_size(),
            post_line_height: default_post_line_height(),
            default_blur_images: default_blur_images(),
            scroll_amount: default_scroll_amount(),
            theme: default_theme(),
            initial_scroll_position: default_initial_scroll_position(),
            default_name: default_name(),
            default_mail: default_mail(),
        }
    }
}

/// レス書き込みペイロード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostPayload {
    pub server: String,
    pub board: String,
    pub key: String,
    pub name: String,
    pub mail: String,
    pub body: String,
    #[serde(default)]
    pub extra_params: HashMap<String, String>,
}

/// 書き込みレスポンスステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PostResponseStatus {
    Success,
    NeedConfirm,
    Error,
}

/// 書き込み処理結果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostResult {
    pub status: PostResponseStatus,
    pub message: String,
    #[serde(default)]
    pub extra_params: HashMap<String, String>,
}

