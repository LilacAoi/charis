use crate::error::Result;
use crate::models::{BoardItem, BookmarkThreadItem, HistoryThreadItem, NGSettings, ThreadItem};
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
}
