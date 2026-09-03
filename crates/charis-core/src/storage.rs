use crate::error::Result;
use crate::models::{
    AppSettings, BoardItem, BookmarkThreadItem, HistoryThreadItem, NGSettings, ThreadItem,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct StorageManager {
    base_dir: PathBuf,
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new_default()
    }
}

impl StorageManager {
    pub fn new_default() -> Self {
        let base_dir = if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(config_home).join("charis")
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config").join("charis")
        } else {
            PathBuf::from(".charis")
        };

        Self::new(base_dir)
    }

    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn ensure_dir(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)?;
        }
        Ok(())
    }

    fn favorites_path(&self) -> PathBuf {
        self.base_dir.join("favorites.json")
    }

    fn bookmarks_path(&self) -> PathBuf {
        self.base_dir.join("bookmarks.json")
    }

    fn history_path(&self) -> PathBuf {
        self.base_dir.join("history.json")
    }

    fn ng_settings_path(&self) -> PathBuf {
        self.base_dir.join("ng_settings.json")
    }

    fn settings_path(&self) -> PathBuf {
        self.base_dir.join("settings.json")
    }

    fn read_positions_path(&self) -> PathBuf {
        self.base_dir.join("read_positions.json")
    }

    // --- Favorites ---
    pub fn get_favorites(&self) -> Vec<BoardItem> {
        self.read_json(&self.favorites_path()).unwrap_or_default()
    }

    pub fn add_favorite(&self, board: BoardItem) -> Result<()> {
        let mut list = self.get_favorites();
        if !list.iter().any(|b| b.server == board.server && b.board == board.board) {
            list.push(board);
            self.write_json(&self.favorites_path(), &list)?;
        }
        Ok(())
    }

    pub fn remove_favorite(&self, server: &str, board: &str) -> Result<()> {
        let list = self.get_favorites();
        let filtered: Vec<BoardItem> = list
            .into_iter()
            .filter(|b| !(b.server == server && b.board == board))
            .collect();
        self.write_json(&self.favorites_path(), &filtered)
    }

    pub fn is_favorite(&self, server: &str, board: &str) -> bool {
        self.get_favorites()
            .iter()
            .any(|b| b.server == server && b.board == board)
    }

    // --- Bookmarks ---
    pub fn get_bookmarks(&self) -> Vec<BookmarkThreadItem> {
        self.read_json(&self.bookmarks_path()).unwrap_or_default()
    }

    pub fn add_bookmark(&self, board: BoardItem, thread: ThreadItem) -> Result<()> {
        let mut list = self.get_bookmarks();
        let now = chrono::Utc::now().timestamp_millis();

        if let Some(pos) = list.iter().position(|item| {
            item.board.server == board.server
                && item.board.board == board.board
                && item.thread.id == thread.id
        }) {
            list[pos].bookmarked_at = now;
        } else {
            list.insert(
                0,
                BookmarkThreadItem {
                    board,
                    thread,
                    bookmarked_at: now,
                },
            );
        }

        self.write_json(&self.bookmarks_path(), &list)
    }

    pub fn remove_bookmark(&self, server: &str, board: &str, thread_id: &str) -> Result<()> {
        let list = self.get_bookmarks();
        let filtered: Vec<BookmarkThreadItem> = list
            .into_iter()
            .filter(|item| {
                !(item.board.server == server
                    && item.board.board == board
                    && item.thread.id == thread_id)
            })
            .collect();
        self.write_json(&self.bookmarks_path(), &filtered)
    }

    pub fn is_bookmarked(&self, server: &str, board: &str, thread_id: &str) -> bool {
        self.get_bookmarks().iter().any(|item| {
            item.board.server == server
                && item.board.board == board
                && item.thread.id == thread_id
        })
    }

    // --- History ---
    pub fn get_history(&self) -> Vec<HistoryThreadItem> {
        self.read_json(&self.history_path()).unwrap_or_default()
    }

    pub fn add_history(&self, board: BoardItem, thread: ThreadItem) -> Result<()> {
        let list = self.get_history();
        let now = chrono::Utc::now().timestamp_millis();

        let mut filtered: Vec<HistoryThreadItem> = list
            .into_iter()
            .filter(|item| {
                !(item.board.server == board.server
                    && item.board.board == board.board
                    && item.thread.id == thread.id)
            })
            .collect();

        filtered.insert(
            0,
            HistoryThreadItem {
                board,
                thread,
                visited_at: now,
            },
        );

        // 最新50件に制限
        if filtered.len() > 50 {
            filtered.truncate(50);
        }

        self.write_json(&self.history_path(), &filtered)
    }

    pub fn clear_history(&self) -> Result<()> {
        self.write_json(&self.history_path(), &Vec::<HistoryThreadItem>::new())
    }

    // --- NG Settings ---
    pub fn get_ng_settings(&self) -> NGSettings {
        self.read_json(&self.ng_settings_path())
            .unwrap_or_default()
    }

    pub fn save_ng_settings(&self, settings: &NGSettings) -> Result<()> {
        self.write_json(&self.ng_settings_path(), settings)
    }

    // --- App Settings ---
    pub fn get_app_settings(&self) -> AppSettings {
        self.read_json(&self.settings_path()).unwrap_or_default()
    }

    pub fn save_app_settings(&self, settings: &AppSettings) -> Result<()> {
        self.write_json(&self.settings_path(), settings)
    }

    // --- Read Positions (Bookmarks/Scroll positions) ---
    pub fn get_read_positions(&self) -> HashMap<String, u32> {
        self.read_json(&self.read_positions_path())
            .unwrap_or_default()
    }

    pub fn save_read_position(&self, key: &str, res_number: u32) -> Result<()> {
        let mut map = self.get_read_positions();
        map.insert(key.to_string(), res_number);
        self.write_json(&self.read_positions_path(), &map)
    }

    // Helper functions
    fn read_json<T: serde::de::DeserializeOwned>(&self, path: &PathBuf) -> Option<T> {
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn write_json<T: serde::Serialize>(&self, path: &PathBuf, value: &T) -> Result<()> {
        self.ensure_dir()?;
        let json_str = serde_json::to_string_pretty(value)?;
        fs::write(path, json_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_favorites() {
        let temp_dir = std::env::temp_dir().join(format!("charis_test_{}", std::process::id()));
        let storage = StorageManager::new(temp_dir.clone());

        let board = BoardItem {
            name: "プログラム".into(),
            url: "https://mevius.5ch.io/tech/".into(),
            server: "mevius".into(),
            board: "tech".into(),
        };

        assert!(!storage.is_favorite("mevius", "tech"));
        storage.add_favorite(board.clone()).unwrap();
        assert!(storage.is_favorite("mevius", "tech"));

        let list = storage.get_favorites();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "プログラム");

        storage.remove_favorite("mevius", "tech").unwrap();
        assert!(!storage.is_favorite("mevius", "tech"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_storage_app_settings() {
        let temp_dir = std::env::temp_dir().join(format!("charis_settings_test_{}", std::process::id()));
        let storage = StorageManager::new(temp_dir.clone());

        let default_settings = storage.get_app_settings();
        assert_eq!(default_settings.ui_font_size, 13);
        assert_eq!(default_settings.post_font_size, 14);

        let mut updated = default_settings.clone();
        updated.ui_font_size = 16;
        updated.post_font_family = "MS PMincho".into();
        storage.save_app_settings(&updated).unwrap();

        let loaded = storage.get_app_settings();
        assert_eq!(loaded.ui_font_size, 16);
        assert_eq!(loaded.post_font_family, "MS PMincho");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_storage_read_positions() {
        let temp_dir = std::env::temp_dir().join(format!("charis_read_pos_test_{}", std::process::id()));
        let storage = StorageManager::new(temp_dir.clone());

        let positions = storage.get_read_positions();
        assert!(positions.is_empty());

        storage.save_read_position("server_board_12345", 42).unwrap();
        let loaded = storage.get_read_positions();
        assert_eq!(loaded.get("server_board_12345"), Some(&42));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
