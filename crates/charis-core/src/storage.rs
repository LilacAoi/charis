use crate::error::Result;
use crate::models::{
    AppSettings, BoardItem, BookmarkThreadItem, CachedThreadItem, DatDroppedThreadItem,
    HistoryThreadItem, NGSettings, ThreadContent, ThreadItem,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct StorageManager {
    base_dir: PathBuf,
    lock: Arc<Mutex<()>>,
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
        Self {
            base_dir,
            lock: Arc::new(Mutex::new(())),
        }
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

    pub fn save_favorites(&self, list: &[BoardItem]) -> Result<()> {
        self.write_json(&self.favorites_path(), &list)
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

    pub fn save_bookmarks(&self, list: &[BookmarkThreadItem]) -> Result<()> {
        self.write_json(&self.bookmarks_path(), &list)
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
        let _ = self.clear_thread_cache();
        self.write_json(&self.history_path(), &Vec::<HistoryThreadItem>::new())
    }

    // --- DAT Dropped Threads ---
    fn dat_dropped_path(&self) -> PathBuf {
        self.base_dir.join("dat_dropped.json")
    }

    pub fn get_dat_dropped_threads(&self) -> Vec<DatDroppedThreadItem> {
        self.read_json(&self.dat_dropped_path()).unwrap_or_default()
    }

    pub fn add_dat_dropped_thread(&self, board: BoardItem, thread: ThreadItem) -> Result<()> {
        let mut list = self.get_dat_dropped_threads();
        let now = chrono::Utc::now().timestamp_millis();

        if let Some(pos) = list.iter().position(|item| {
            item.board.server == board.server
                && item.board.board == board.board
                && item.thread.id == thread.id
        }) {
            list[pos].archived_at = now;
            list[pos].thread = thread;
        } else {
            list.insert(
                0,
                DatDroppedThreadItem {
                    board,
                    thread,
                    archived_at: now,
                },
            );
        }

        self.write_json(&self.dat_dropped_path(), &list)
    }

    pub fn remove_dat_dropped_thread(&self, server: &str, board: &str, thread_id: &str) -> Result<()> {
        let list = self.get_dat_dropped_threads();
        let filtered: Vec<DatDroppedThreadItem> = list
            .into_iter()
            .filter(|item| {
                !(item.board.server == server
                    && item.board.board == board
                    && item.thread.id == thread_id)
            })
            .collect();
        self.write_json(&self.dat_dropped_path(), &filtered)
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
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut map = self.get_read_positions();
        map.insert(key.to_string(), res_number);
        self.write_json(&self.read_positions_path(), &map)
    }

    // --- Thread Cache / Offline Archive ---
    pub fn thread_cache_dir(&self) -> PathBuf {
        self.base_dir.join("cache").join("threads")
    }

    pub fn thread_cache_path(&self, server: &str, board: &str, key: &str) -> PathBuf {
        self.thread_cache_dir()
            .join(server)
            .join(board)
            .join(format!("{key}.json"))
    }

    pub fn save_thread_cache(
        &self,
        server: &str,
        board: &str,
        key: &str,
        content: &ThreadContent,
    ) -> Result<()> {
        let path = self.thread_cache_path(server, board, key);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let json_str = serde_json::to_string_pretty(content)?;
        fs::write(path, json_str)?;
        Ok(())
    }

    pub fn get_thread_cache(
        &self,
        server: &str,
        board: &str,
        key: &str,
    ) -> Option<ThreadContent> {
        let path = self.thread_cache_path(server, board, key);
        self.read_json(&path)
    }

    pub fn has_thread_cache(&self, server: &str, board: &str, key: &str) -> bool {
        self.thread_cache_path(server, board, key).exists()
    }

    pub fn delete_thread_cache(&self, server: &str, board: &str, key: &str) -> Result<()> {
        let path = self.thread_cache_path(server, board, key);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn clear_thread_cache(&self) -> Result<()> {
        let root = self.thread_cache_dir();
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        Ok(())
    }

    pub fn get_cached_threads(&self) -> Vec<CachedThreadItem> {
        let root = self.thread_cache_dir();
        if !root.exists() {
            return Vec::new();
        }

        let mut items = Vec::new();
        if let Ok(servers) = fs::read_dir(&root) {
            for s_entry in servers.flatten() {
                let s_path = s_entry.path();
                if !s_path.is_dir() {
                    continue;
                }
                let server_name = s_entry.file_name().to_string_lossy().to_string();

                if let Ok(boards) = fs::read_dir(&s_path) {
                    for b_entry in boards.flatten() {
                        let b_path = b_entry.path();
                        if !b_path.is_dir() {
                            continue;
                        }
                        let board_name = b_entry.file_name().to_string_lossy().to_string();

                        if let Ok(files) = fs::read_dir(&b_path) {
                            for f_entry in files.flatten() {
                                let f_path = f_entry.path();
                                if f_path.extension().and_then(|e| e.to_str()) != Some("json") {
                                    continue;
                                }
                                let stem = f_path
                                    .file_stem()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                let updated_at = f_entry
                                    .metadata()
                                    .ok()
                                    .and_then(|m| m.modified().ok())
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_millis() as i64)
                                    .unwrap_or(0);

                                if let Some(content) = self.read_json::<ThreadContent>(&f_path) {
                                    items.push(CachedThreadItem {
                                        server: server_name.clone(),
                                        board: board_name.clone(),
                                        key: stem,
                                        title: content.title,
                                        post_count: content.posts.len(),
                                        is_archived: content.is_archived,
                                        updated_at,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        items
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

        // test save_favorites reordering
        let reordered = vec![
            BoardItem {
                name: "板B".into(),
                url: "https://mevius.5ch.io/b/".into(),
                server: "mevius".into(),
                board: "b".into(),
            },
            BoardItem {
                name: "板A".into(),
                url: "https://mevius.5ch.io/a/".into(),
                server: "mevius".into(),
                board: "a".into(),
            },
        ];
        storage.save_favorites(&reordered).unwrap();
        let loaded = storage.get_favorites();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "板B");
        assert_eq!(loaded[1].name, "板A");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_storage_app_settings() {
        let temp_dir = std::env::temp_dir().join(format!("charis_settings_test_{}", std::process::id()));
        let storage = StorageManager::new(temp_dir.clone());

        let default_settings = storage.get_app_settings();
        assert_eq!(default_settings.ui_font_size, 13);
        assert_eq!(default_settings.post_font_size, 14);
        assert_eq!(default_settings.initial_scroll_position, "lastRead");

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
        storage.save_read_position("server_board_67890", 150).unwrap();
        storage.save_read_position("another_server_test_999", 7).unwrap();

        let loaded = storage.get_read_positions();
        assert_eq!(loaded.get("server_board_12345"), Some(&42));
        assert_eq!(loaded.get("server_board_67890"), Some(&150));
        assert_eq!(loaded.get("another_server_test_999"), Some(&7));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_storage_thread_cache() {
        let temp_dir = std::env::temp_dir().join(format!("charis_cache_test_{}", std::process::id()));
        let storage = StorageManager::new(temp_dir.clone());

        let content = ThreadContent {
            title: "テストスレッド".into(),
            posts: vec![],
            is_archived: true,
            from_cache: false,
        };

        assert!(!storage.has_thread_cache("agree", "operate", "1684064837"));
        storage.save_thread_cache("agree", "operate", "1684064837", &content).unwrap();
        assert!(storage.has_thread_cache("agree", "operate", "1684064837"));

        let cached = storage.get_thread_cache("agree", "operate", "1684064837").unwrap();
        assert_eq!(cached.title, "テストスレッド");
        assert!(cached.is_archived);

        let list = storage.get_cached_threads();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "テストスレッド");
        assert_eq!(list[0].key, "1684064837");
        assert!(list[0].is_archived);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_storage_dat_dropped() {
        let temp_dir = std::env::temp_dir().join(format!("charis_dat_drop_test_{}", std::process::id()));
        let storage = StorageManager::new(temp_dir.clone());

        let board = BoardItem {
            name: "運用情報".into(),
            url: "https://agree.5ch.io/operate/".into(),
            server: "agree".into(),
            board: "operate".into(),
        };
        let thread = ThreadItem {
            id: "1684064837".into(),
            title: "過去ログスレッド".into(),
            res_count: 1000,
            ikioi: 0,
        };

        assert!(storage.get_dat_dropped_threads().is_empty());
        storage.add_dat_dropped_thread(board.clone(), thread.clone()).unwrap();

        let list = storage.get_dat_dropped_threads();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].thread.title, "過去ログスレッド");

        storage.remove_dat_dropped_thread("agree", "operate", "1684064837").unwrap();
        assert!(storage.get_dat_dropped_threads().is_empty());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
