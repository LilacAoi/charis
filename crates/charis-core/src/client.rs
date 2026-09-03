use crate::error::{CharisError, Result};
use crate::models::{BoardCategory, BoardItem, ThreadContent, ThreadItem};
use crate::parser::{decode_cp932, parse_bbsmenu_html, parse_bbsmenu_json, parse_dat, parse_subject_txt};
use reqwest::header::USER_AGENT;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const DEFAULT_USER_AGENT: &str = "Monazilla/1.00 (charis/0.1.0)";
const DEFAULT_RATE_LIMIT_DELAY: Duration = Duration::from_millis(1000);

#[derive(Clone)]
pub struct FiveChannelClient {
    http_client: reqwest::Client,
    user_agent: String,
    last_request_time: Arc<Mutex<Option<Instant>>>,
    rate_limit_delay: Duration,
}

impl Default for FiveChannelClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FiveChannelClient {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            http_client,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            last_request_time: Arc::new(Mutex::new(None)),
            rate_limit_delay: DEFAULT_RATE_LIMIT_DELAY,
        }
    }

    /// レートリミット制御（連続リクエスト時に最低1秒間隔を空ける）
    async fn ensure_rate_limit(&self) {
        let mut last = self.last_request_time.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < self.rate_limit_delay {
                tokio::time::sleep(self.rate_limit_delay - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    /// 板一覧（BBSMenu）の取得
    pub async fn get_bbsmenu(&self) -> Result<Vec<BoardCategory>> {
        self.ensure_rate_limit().await;

        let urls = [
            "https://menu.5ch.io/bbsmenu.json",
            "https://menu.5ch.net/bbsmenu.json",
        ];

        for url in urls {
            let res = self
                .http_client
                .get(url)
                .header(USER_AGENT, &self.user_agent)
                .send()
                .await;

            if let Ok(response) = res {
                if response.status().is_success() {
                    if let Ok(bytes) = response.bytes().await {
                        // UTF-8 または CP932 としてデコードを試みる
                        let text = String::from_utf8(bytes.to_vec())
                            .unwrap_or_else(|_| decode_cp932(&bytes));

                        if let Ok(categories) = parse_bbsmenu_json(&text) {
                            if !categories.is_empty() {
                                return Ok(categories);
                            }
                        }
                    }
                }
            }
        }

        // JSON取得失敗時は HTML 版をフォールバックとして試行
        self.get_bbsmenu_html_fallback().await
    }

    /// BBSMenu HTML フォールバック取得
    async fn get_bbsmenu_html_fallback(&self) -> Result<Vec<BoardCategory>> {
        self.ensure_rate_limit().await;

        let url = "https://menu.5ch.io/bbsmenu.html";
        let res = self
            .http_client
            .get(url)
            .header(USER_AGENT, &self.user_agent)
            .send()
            .await;

        if let Ok(response) = res {
            if response.status().is_success() {
                if let Ok(bytes) = response.bytes().await {
                    let html = decode_cp932(&bytes);
                    if let Ok(categories) = parse_bbsmenu_html(&html) {
                        if !categories.is_empty() {
                            return Ok(categories);
                        }
                    }
                }
            }
        }

        // ネットワークやパースが全て失敗した場合のデフォルトおすすめ板
        Ok(vec![BoardCategory {
            name: "おすすめ".to_string(),
            boards: vec![
                BoardItem {
                    name: "ニュース速報(VIP)".to_string(),
                    url: "https://mi.5ch.io/news4vip/".to_string(),
                    server: "mi".to_string(),
                    board: "news4vip".to_string(),
                },
                BoardItem {
                    name: "なんでも実況G".to_string(),
                    url: "https://nova.5ch.io/livegalileo/".to_string(),
                    server: "nova".to_string(),
                    board: "livegalileo".to_string(),
                },
                BoardItem {
                    name: "プログラム".to_string(),
                    url: "https://mevius.5ch.io/tech/".to_string(),
                    server: "mevius".to_string(),
                    board: "tech".to_string(),
                },
            ],
        }])
    }

    /// スレッド一覧（subject.txt）の取得
    pub async fn get_thread_list(&self, server: &str, board: &str) -> Result<Vec<ThreadItem>> {
        self.ensure_rate_limit().await;

        let domains = ["5ch.io", "5ch.net"];
        for domain in domains {
            let url = format!("https://{server}.{domain}/{board}/subject.txt");
            let res = self
                .http_client
                .get(&url)
                .header(USER_AGENT, &self.user_agent)
                .send()
                .await;

            if let Ok(response) = res {
                if response.status().is_success() {
                    if let Ok(bytes) = response.bytes().await {
                        let text = decode_cp932(&bytes);
                        let now = chrono::Utc::now().timestamp();
                        let threads = parse_subject_txt(&text, now);
                        if !threads.is_empty() {
                            return Ok(threads);
                        }
                    }
                }
            }
        }

        Err(CharisError::Parse(format!(
            "Failed to fetch thread list for {server}/{board}"
        )))
    }

    /// スレッド本文（DAT）の取得
    pub async fn get_thread_posts(
        &self,
        server: &str,
        board: &str,
        key: &str,
    ) -> Result<ThreadContent> {
        self.ensure_rate_limit().await;

        let domains = ["5ch.io", "5ch.net"];
        for domain in domains {
            let url = format!("https://{server}.{domain}/{board}/dat/{key}.dat");
            let res = self
                .http_client
                .get(&url)
                .header(USER_AGENT, &self.user_agent)
                .send()
                .await;

            if let Ok(response) = res {
                if response.status().is_success() {
                    if let Ok(bytes) = response.bytes().await {
                        let text = decode_cp932(&bytes);
                        let content = parse_dat(&text);
                        if !content.posts.is_empty() {
                            return Ok(content);
                        }
                    }
                }
            }
        }

        Err(CharisError::Parse(format!(
            "Failed to fetch dat for {server}/{board}/{key}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_online_fetch_bbsmenu() {
        let client = FiveChannelClient::new();
        let menu = client.get_bbsmenu().await.expect("Failed to fetch BBSMenu");
        assert!(!menu.is_empty(), "BBSMenu should not be empty");
        println!("Fetched {} categories", menu.len());
        for cat in menu.iter().take(3) {
            println!("Category: {} ({} boards)", cat.name, cat.boards.len());
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_online_fetch_thread_list() {
        let client = FiveChannelClient::new();
        let threads = client.get_thread_list("mevius", "tech").await.expect("Failed to fetch threads");
        assert!(!threads.is_empty(), "Threads should not be empty");
        println!("Fetched {} threads from mevius/tech", threads.len());
        println!("Top thread: {} (res: {}, ikioi: {})", threads[0].title, threads[0].res_count, threads[0].ikioi);
    }
}

    #[tokio::test]
    #[ignore]
    async fn test_online_fetch_thread_posts() {
        let client = FiveChannelClient::new();
        let threads = client.get_thread_list("mevius", "tech").await.expect("Failed to fetch threads");
        if let Some(target) = threads.first() {
            let content = client.get_thread_posts("mevius", "tech", &target.id).await.expect("Failed to fetch dat");
            println!("Thread title: {}", content.title);
            println!("Fetched {} posts", content.posts.len());
            if let Some(first_post) = content.posts.first() {
                println!(">>1 Name: {}, Date: {}, ID: {}", first_post.name, first_post.date, first_post.id);
            }
        }
    }
