#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use charis_core::{
    AppSettings, BoardCategory, BoardItem, BookmarkThreadItem, CachedThreadItem,
    DatDroppedThreadItem, FiveChannelClient, HistoryThreadItem, NGSettings, PostPayload,
    PostResult, StorageManager, ThreadContent, ThreadItem,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

struct AppState {
    client: FiveChannelClient,
    storage: StorageManager,
}

#[tauri::command]
async fn get_bbsmenu(state: State<'_, Arc<AppState>>) -> Result<Vec<BoardCategory>, String> {
    println!("[charis-backend] get_bbsmenu called");
    let res = state.client.get_bbsmenu().await.map_err(|e| {
        eprintln!("[charis-backend] get_bbsmenu error: {e}");
        e.to_string()
    });
    if let Ok(ref categories) = res {
        println!("[charis-backend] get_bbsmenu success: {} categories found", categories.len());
    }
    res
}

#[tauri::command]
async fn get_thread_list(
    state: State<'_, Arc<AppState>>,
    server: String,
    board: String,
) -> Result<Vec<ThreadItem>, String> {
    println!("[charis-backend] get_thread_list called for {server}/{board}");
    let res = state
        .client
        .get_thread_list(&server, &board)
        .await
        .map_err(|e| {
            eprintln!("[charis-backend] get_thread_list error: {e}");
            e.to_string()
        });
    if let Ok(ref threads) = res {
        println!("[charis-backend] get_thread_list success: {} threads found", threads.len());
    }
    res
}

#[tauri::command]
async fn get_thread_posts(
    state: State<'_, Arc<AppState>>,
    server: String,
    board: String,
    key: String,
) -> Result<ThreadContent, String> {
    println!("[charis-backend] get_thread_posts called for {server}/{board}/{key}");

    // 1. リモートからの取得試行 (稼働中DAT -> oyster自動フォールバック)
    match state.client.get_thread_posts(&server, &board, &key).await {
        Ok(content) => {
            println!(
                "[charis-backend] get_thread_posts remote success: \"{}\" with {} posts (is_archived: {})",
                content.title,
                content.posts.len(),
                content.is_archived
            );
            // 取得成功したらローカルキャッシュに保存
            if let Err(e) = state.storage.save_thread_cache(&server, &board, &key, &content) {
                eprintln!("[charis-backend] failed to save thread cache: {e}");
            }
            Ok(content)
        }
        Err(remote_err) => {
            // 2. リモート失敗時、ローカルキャッシュからの復元を試行
            if let Some(mut cached) = state.storage.get_thread_cache(&server, &board, &key) {
                println!(
                    "[charis-backend] get_thread_posts loaded from cache: \"{}\" with {} posts (is_archived: {})",
                    cached.title,
                    cached.posts.len(),
                    cached.is_archived
                );
                cached.from_cache = true;
                Ok(cached)
            } else {
                eprintln!("[charis-backend] get_thread_posts remote error and no cache: {remote_err}");
                Err(remote_err.to_string())
            }
        }
    }
}

#[tauri::command]
fn get_cached_threads(state: State<'_, Arc<AppState>>) -> Vec<CachedThreadItem> {
    state.storage.get_cached_threads()
}

#[tauri::command]
fn delete_thread_cache(
    state: State<'_, Arc<AppState>>,
    server: String,
    board: String,
    key: String,
) -> Result<(), String> {
    state
        .storage
        .delete_thread_cache(&server, &board, &key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_thread_cache(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state
        .storage
        .clear_thread_cache()
        .map_err(|e| e.to_string())
}


#[tauri::command]
fn get_favorites(state: State<'_, Arc<AppState>>) -> Vec<BoardItem> {
    state.storage.get_favorites()
}

#[tauri::command]
fn add_favorite(state: State<'_, Arc<AppState>>, board: BoardItem) -> Result<(), String> {
    state.storage.add_favorite(board).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_favorite(
    state: State<'_, Arc<AppState>>,
    server: String,
    board: String,
) -> Result<(), String> {
    state
        .storage
        .remove_favorite(&server, &board)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_favorites(
    state: State<'_, Arc<AppState>>,
    list: Vec<BoardItem>,
) -> Result<(), String> {
    state
        .storage
        .save_favorites(&list)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_bookmarks(state: State<'_, Arc<AppState>>) -> Vec<BookmarkThreadItem> {
    state.storage.get_bookmarks()
}

#[tauri::command]
fn add_bookmark(
    state: State<'_, Arc<AppState>>,
    board: BoardItem,
    thread: ThreadItem,
) -> Result<(), String> {
    state
        .storage
        .add_bookmark(board, thread)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_bookmark(
    state: State<'_, Arc<AppState>>,
    server: String,
    board: String,
    thread_id: String,
) -> Result<(), String> {
    state
        .storage
        .remove_bookmark(&server, &board, &thread_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_bookmarks(
    state: State<'_, Arc<AppState>>,
    list: Vec<BookmarkThreadItem>,
) -> Result<(), String> {
    state
        .storage
        .save_bookmarks(&list)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_dat_dropped_threads(state: State<'_, Arc<AppState>>) -> Vec<DatDroppedThreadItem> {
    state.storage.get_dat_dropped_threads()
}

#[tauri::command]
fn add_dat_dropped_thread(
    state: State<'_, Arc<AppState>>,
    board: BoardItem,
    thread: ThreadItem,
) -> Result<(), String> {
    state
        .storage
        .add_dat_dropped_thread(board, thread)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_dat_dropped_thread(
    state: State<'_, Arc<AppState>>,
    server: String,
    board: String,
    thread_id: String,
) -> Result<(), String> {
    state
        .storage
        .remove_dat_dropped_thread(&server, &board, &thread_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history(state: State<'_, Arc<AppState>>) -> Vec<HistoryThreadItem> {
    state.storage.get_history()
}

#[tauri::command]
fn add_history(
    state: State<'_, Arc<AppState>>,
    board: BoardItem,
    thread: ThreadItem,
) -> Result<(), String> {
    state
        .storage
        .add_history(board, thread)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.storage.clear_history().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ng_settings(state: State<'_, Arc<AppState>>) -> NGSettings {
    state.storage.get_ng_settings()
}

#[tauri::command]
fn save_ng_settings(state: State<'_, Arc<AppState>>, settings: NGSettings) -> Result<(), String> {
    state
        .storage
        .save_ng_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.storage.get_app_settings()
}

#[tauri::command]
fn save_app_settings(state: State<'_, Arc<AppState>>, settings: AppSettings) -> Result<(), String> {
    state
        .storage
        .save_app_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_read_positions(state: State<'_, Arc<AppState>>) -> HashMap<String, u32> {
    state.storage.get_read_positions()
}

#[tauri::command]
fn save_read_position(
    state: State<'_, Arc<AppState>>,
    key: String,
    res_number: u32,
) -> Result<(), String> {
    state
        .storage
        .save_read_position(&key, res_number)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL protocol".to_string());
    }
    println!("[charis-backend] Opening external URL: {url}");

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to run xdg-open: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to run open: {e}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to run start: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
async fn post_comment(
    state: State<'_, Arc<AppState>>,
    payload: PostPayload,
) -> Result<PostResult, String> {
    println!(
        "[charis-backend] post_comment called for {}/{} (key: {})",
        payload.server, payload.board, payload.key
    );
    let res = state.client.post_comment(&payload).await.map_err(|e| {
        eprintln!("[charis-backend] post_comment error: {e}");
        e.to_string()
    });
    if let Ok(ref result) = res {
        println!(
            "[charis-backend] post_comment result: {:?} - {}",
            result.status, result.message
        );
    }
    res
}

fn main() {
    #[cfg(target_os = "linux")]
    unsafe {
        // Wayland + WebKitGTK / NVIDIA / Explicit Sync 環境でのクラッシュ
        // (wp_linux_drm_syncobj_surface_v1 "Missing acquire timeline" / Error 71) 回避
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    let client = FiveChannelClient::new();
    let storage = StorageManager::new_default();
    let state = Arc::new(AppState { client, storage });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_bbsmenu,
            get_thread_list,
            get_thread_posts,
            get_cached_threads,
            delete_thread_cache,
            clear_thread_cache,
            get_favorites,
            add_favorite,
            remove_favorite,
            save_favorites,
            get_bookmarks,
            add_bookmark,
            remove_bookmark,
            save_bookmarks,
            get_dat_dropped_threads,
            add_dat_dropped_thread,
            remove_dat_dropped_thread,
            get_history,
            add_history,
            clear_history,
            get_ng_settings,
            save_ng_settings,
            get_app_settings,
            save_app_settings,
            get_read_positions,
            save_read_position,
            post_comment,
            open_external_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


