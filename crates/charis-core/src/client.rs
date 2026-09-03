use crate::error::{CharisError, Result};
use crate::models::{
    BoardCategory, BoardItem, PostPayload, PostResult, ThreadContent, ThreadItem,
};
use crate::parser::{
    decode_cp932, encode_cp932_url, parse_bbsmenu_html, parse_bbsmenu_json, parse_dat,
    parse_post_response, parse_subject_txt,
};
use reqwest::header::USER_AGENT;
use std::collections::HashMap;
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
    cookies: Arc<Mutex<HashMap<String, String>>>,
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
            cookies: Arc::new(Mutex::new(HashMap::new())),
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

    /// レス書き込み POST リクエスト
    pub async fn post_comment(&self, payload: &PostPayload) -> Result<PostResult> {
        self.ensure_rate_limit().await;

        let url = format!("https://{}.5ch.io/test/bbs.cgi", payload.server);
        let referer = format!(
            "https://{}.5ch.io/test/read.cgi/{}/{}/",
            payload.server, payload.board, payload.key
        );
        let origin = format!("https://{}.5ch.io", payload.server);

        // 送信パラメータの構築 (Shift_JIS URLエンコード)
        let now = chrono::Utc::now().timestamp();
        let time_str = payload
            .extra_params
            .get("time")
            .cloned()
            .unwrap_or_else(|| now.to_string());

        let submit_val = payload
            .extra_params
            .get("submit")
            .cloned()
            .unwrap_or_else(|| "書き込む".to_string());

        let mut body_params = vec![
            format!("bbs={}", payload.board),
            format!("key={}", payload.key),
            format!("time={}", time_str),
            format!("FROM={}", encode_cp932_url(&payload.name)),
            format!("mail={}", encode_cp932_url(&payload.mail)),
            format!(
                "MESSAGE={}",
                encode_cp932_url(&payload.body.replace('\n', "\r\n"))
            ),
            format!("submit={}", encode_cp932_url(&submit_val)),
        ];

        // extra_params にある追加パラメータ (bbs, key, time, FROM, mail, MESSAGE, submit 以外) を追記
        for (k, v) in &payload.extra_params {
            if !["bbs", "key", "time", "FROM", "mail", "MESSAGE", "submit"].contains(&k.as_str()) {
                body_params.push(format!("{}={}", k, encode_cp932_url(v)));
            }
        }

        let body_str = body_params.join("&");

        // 保持している Cookie を取得
        let cookie_header = {
            let cookies = self.cookies.lock().await;
            cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ")
        };

        let mut req = self
            .http_client
            .post(&url)
            .header(USER_AGENT, &self.user_agent)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .header(reqwest::header::REFERER, &referer)
            .header("Origin", &origin)
            .header(
                reqwest::header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header(reqwest::header::ACCEPT_LANGUAGE, "ja,en-US;q=0.9,en;q=0.8")
            .body(body_str);

        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, cookie_header);
        }

        let response = req.send().await?;

        // Set-Cookie ヘッダーの処理
        {
            let mut cookies_lock = self.cookies.lock().await;
            for val in response.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(s) = val.to_str() {
                    if let Some(cookie_pair) = s.split(';').next() {
                        let mut parts = cookie_pair.splitn(2, '=');
                        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                            cookies_lock.insert(k.trim().to_string(), v.trim().to_string());
                        }
                    }
                }
            }
        }

        let bytes = response.bytes().await?;
        let text = String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| decode_cp932(&bytes));

        let result = parse_post_response(&text);
        Ok(result)
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
