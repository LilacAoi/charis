use crate::error::{CharisError, Result};
use crate::models::{BoardCategory, BoardItem, PostItem, ThreadContent, ThreadItem};
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;

static BOARD_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://([^.]+)\.5ch\.(?:io|net)/([^/]+)/?").unwrap()
});

static SUBJECT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d+)\.dat<>(.*?)\s*\((\d+)\)$").unwrap()
});

static ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"ID:([^\s]+)").unwrap()
});

static HTML_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<[^>]*>").unwrap()
});

static BBSMENU_CAT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)<B>(.*?)</B>").unwrap()
});

static BBSMENU_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<A\s+HREF=(https?://([^.]+)\.5ch\.(?:io|net)/([^/]+)/?)>(.*?)</A>"#).unwrap()
});

/// Shift_JIS (CP932) のバイト列を UTF-8 文字列にデコード
pub fn decode_cp932(bytes: &[u8]) -> String {
    let (cow, _encoding, malformed) = encoding_rs::SHIFT_JIS.decode(bytes);
    if malformed {
        eprintln!("[warn] Shift_JIS decode encountered malformed sequences");
    }
    cow.into_owned()
}

// BBSMenu JSON の構造定義
#[derive(Deserialize)]
struct RawBBSMenuJson {
    menu_list: Option<Vec<RawCategory>>,
}

#[derive(Deserialize)]
struct RawCategory {
    category_name: Option<String>,
    category_content: Option<Vec<RawBoardContent>>,
}

#[derive(Deserialize)]
struct RawBoardContent {
    board_name: Option<String>,
    url: Option<String>,
    directory_name: Option<String>,
}

/// BBSMenu JSON 形式のパース
pub fn parse_bbsmenu_json(json_str: &str) -> Result<Vec<BoardCategory>> {
    let menu: RawBBSMenuJson = serde_json::from_str(json_str)?;
    let menu_list = menu.menu_list.unwrap_or_default();

    let mut categories = Vec::new();

    for cat in menu_list {
        let cat_name = match cat.category_name {
            Some(n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => continue,
        };

        let raw_boards = cat.category_content.unwrap_or_default();
        let mut boards = Vec::new();

        for b in raw_boards {
            let (board_name, url) = match (b.board_name, b.url) {
                (Some(name), Some(url)) if !name.trim().is_empty() && !url.trim().is_empty() => {
                    (name.trim().to_string(), url.trim().to_string())
                }
                _ => continue,
            };

            if let Some(caps) = BOARD_URL_REGEX.captures(&url) {
                let server = caps[1].to_string();
                let board = b.directory_name.unwrap_or_else(|| caps[2].to_string());

                boards.push(BoardItem {
                    name: board_name,
                    url,
                    server,
                    board,
                });
            }
        }

        if !boards.is_empty() {
            categories.push(BoardCategory {
                name: cat_name,
                boards,
            });
        }
    }

    if categories.is_empty() {
        return Err(CharisError::Parse("No categories found in BBSMenu JSON".to_string()));
    }

    Ok(categories)
}

/// BBSMenu HTML 形式のフォールバックパース
pub fn parse_bbsmenu_html(html_str: &str) -> Result<Vec<BoardCategory>> {
    let mut categories = Vec::new();
    let mut current_category: Option<BoardCategory> = None;

    for line in html_str.lines() {
        if let Some(caps) = BBSMENU_CAT_REGEX.captures(line) {
            let cat_name = caps[1].trim();
            if !cat_name.is_empty() && !cat_name.contains("特別") && !cat_name.contains("チャット") {
                if let Some(prev) = current_category.take() {
                    if !prev.boards.is_empty() {
                        categories.push(prev);
                    }
                }
                current_category = Some(BoardCategory {
                    name: cat_name.to_string(),
                    boards: Vec::new(),
                });
            }
            continue;
        }

        if let Some(caps) = BBSMENU_LINK_REGEX.captures(line) {
            if let Some(ref mut cur) = current_category {
                cur.boards.push(BoardItem {
                    url: caps[1].to_string(),
                    server: caps[2].to_string(),
                    board: caps[3].to_string(),
                    name: caps[4].trim().to_string(),
                });
            }
        }
    }

    if let Some(prev) = current_category {
        if !prev.boards.is_empty() {
            categories.push(prev);
        }
    }

    Ok(categories)
}

/// subject.txt のパース
pub fn parse_subject_txt(text: &str, now_unix: i64) -> Vec<ThreadItem> {
    let mut threads = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(caps) = SUBJECT_REGEX.captures(trimmed) {
            let key = caps[1].to_string();
            let title = caps[2].trim().to_string();
            let res_count = caps[3].parse::<u32>().unwrap_or(0);

            let created_time = key.parse::<i64>().unwrap_or(now_unix);
            let elapsed_days = ((now_unix - created_time) as f64 / 86400.0).max(0.01);
            let ikioi = (res_count as f64 / elapsed_days).round() as u32;

            threads.push(ThreadItem {
                id: key,
                title,
                res_count,
                ikioi,
            });
        }
    }

    // デフォルトで勢い降順にソート
    threads.sort_by(|a, b| b.ikioi.cmp(&a.ikioi));
    threads
}

/// DAT ファイルのパース
pub fn parse_dat(text: &str) -> ThreadContent {
    let mut posts = Vec::new();
    let mut thread_title = String::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split("<>").collect();
        if parts.len() >= 4 {
            let name = parts[0];
            let mail = parts[1];
            let date_and_id = parts[2];
            let body = parts[3];
            let title = parts.get(4).copied().unwrap_or("");

            if index == 0 && !title.trim().is_empty() {
                thread_title = title.trim().to_string();
            }

            let id = if let Some(caps) = ID_REGEX.captures(date_and_id) {
                caps[1].to_string()
            } else {
                String::new()
            };

            let clean_date = ID_REGEX.replace_all(date_and_id, "").trim().to_string();
            let clean_name = HTML_TAG_REGEX.replace_all(name, "").trim().to_string();

            posts.push(PostItem {
                number: (index + 1) as u32,
                name: clean_name,
                mail: mail.trim().to_string(),
                date: clean_date,
                id,
                body: body.trim().to_string(),
            });
        }
    }

    ThreadContent {
        title: thread_title,
        posts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subject_txt() {
        let sample = "1700000000.dat<>【Rust】charis 開発スレッド Part1 (120)\n1700050000.dat<>ニュース速報テスト (50)\n";
        let now = 1700086400; // 約1日後
        let threads = parse_subject_txt(sample, now);

        assert_eq!(threads.len(), 2);
        assert_eq!(threads[0].id, "1700000000");
        assert_eq!(threads[0].title, "【Rust】charis 開発スレッド Part1");
        assert_eq!(threads[0].res_count, 120);
        assert!(threads[0].ikioi > 0);
    }

    #[test]
    fn test_parse_dat() {
        let sample = "名無しさん<>sage<>2024/01/01(月) 12:00:00.00 ID:abc12345<>こんにちは<br>テスト本文<>スレッドのタイトル\n名無しさん<><>2024/01/01(月) 12:01:00.00 ID:xyz98765<>>>1 乙<>\n";
        let content = parse_dat(sample);

        assert_eq!(content.title, "スレッドのタイトル");
        assert_eq!(content.posts.len(), 2);

        assert_eq!(content.posts[0].number, 1);
        assert_eq!(content.posts[0].name, "名無しさん");
        assert_eq!(content.posts[0].mail, "sage");
        assert_eq!(content.posts[0].id, "abc12345");
        assert_eq!(content.posts[0].body, "こんにちは<br>テスト本文");

        assert_eq!(content.posts[1].number, 2);
        assert_eq!(content.posts[1].id, "xyz98765");
        assert_eq!(content.posts[1].body, ">>1 乙");
    }

    #[test]
    fn test_decode_cp932() {
        // "テスト" in CP932: [0x83, 0x65, 0x83, 0x58, 0x83, 0x67]
        let bytes = vec![0x83, 0x65, 0x83, 0x58, 0x83, 0x67];
        let decoded = decode_cp932(&bytes);
        assert_eq!(decoded, "テスト");
    }
}
