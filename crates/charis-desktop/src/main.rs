#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use charis_core::{
    BoardCategory, BoardItem, BookmarkThreadItem, FiveChannelClient, HistoryThreadItem,
    NGSettings, StorageManager, ThreadContent, ThreadItem,
};
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
    let res = state
        .client
        .get_thread_posts(&server, &board, &key)
        .await
        .map_err(|e| {
            eprintln!("[charis-backend] get_thread_posts error: {e}");
            e.to_string()
        });
    if let Ok(ref content) = res {
        println!(
            "[charis-backend] get_thread_posts success: \"{}\" with {} posts",
            content.title,
            content.posts.len()
        );
    }
    res
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

fn main() {
    let client = FiveChannelClient::new();
    let storage = StorageManager::new_default();
    let state = Arc::new(AppState { client, storage });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_bbsmenu,
            get_thread_list,
            get_thread_posts,
            get_favorites,
            add_favorite,
            remove_favorite,
            get_bookmarks,
            add_bookmark,
            remove_bookmark,
            get_history,
            add_history,
            clear_history,
            get_ng_settings,
            save_ng_settings,
            open_external_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

