use crate::models::{NGSettings, PostItem};
#[cfg(test)]
use crate::models::NGMode;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static ANCHOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:&gt;|[>＞]){1,2}(\d+)(?:-\d+)?").unwrap()
});

static ANCHOR_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<a\s+[^>]*?href=["'][^"']*?/(\d+)(?:-\d+)?["'][^>]*>"#).unwrap()
});

/// 本文からアンカー先のレス番号一覧を抽出
pub fn extract_anchor_targets(body: &str) -> Vec<u32> {
    let mut targets = HashSet::new();

    for caps in ANCHOR_TAG_REGEX.captures_iter(body) {
        if let Ok(num) = caps[1].parse::<u32>() {
            if num > 0 {
                targets.insert(num);
            }
        }
    }

    for caps in ANCHOR_REGEX.captures_iter(body) {
        if let Ok(num) = caps[1].parse::<u32>() {
            if num > 0 {
                targets.insert(num);
            }
        }
    }

    let mut result: Vec<u32> = targets.into_iter().collect();
    result.sort();
    result
}

/// スレッド全体の被アンカー（どのレスがどのレスから参照されているか）マップを構築
pub fn build_reply_map(posts: &[PostItem]) -> HashMap<u32, Vec<u32>> {
    let mut reply_map: HashMap<u32, Vec<u32>> = HashMap::new();

    for post in posts {
        let targets = extract_anchor_targets(&post.body);
        for target in targets {
            if target != post.number {
                reply_map.entry(target).or_default().push(post.number);
            }
        }
    }

    // ソートして重複除去
    for list in reply_map.values_mut() {
        list.sort();
        list.dedup();
    }

    reply_map
}

/// NG判定および連鎖あぼーんの計算を行い、NGとなったレス番号の Set を返す
pub fn calculate_ng_posts(posts: &[PostItem], settings: &NGSettings) -> HashSet<u32> {
    let mut ng_posts = HashSet::new();

    // 1. 直接該当するレスの抽出
    for post in posts {
        let mut is_ng = false;

        // ID判定
        if !post.id.is_empty() && settings.ng_ids.iter().any(|id| id == &post.id) {
            is_ng = true;
        }

        // ワード判定
        if !is_ng && !settings.ng_words.is_empty() {
            for word in &settings.ng_words {
                if !word.is_empty()
                    && (post.body.contains(word) || post.name.contains(word))
                {
                    is_ng = true;
                    break;
                }
            }
        }

        if is_ng {
            ng_posts.insert(post.number);
        }
    }

    // 2. 連鎖あぼーんの伝播（再帰的Fix-point計算）
    if settings.chain_abone && !ng_posts.is_empty() {
        let mut changed = true;
        while changed {
            changed = false;
            for post in posts {
                if ng_posts.contains(&post.number) {
                    continue;
                }

                let targets = extract_anchor_targets(&post.body);
                for target in targets {
                    if ng_posts.contains(&target) {
                        ng_posts.insert(post.number);
                        changed = true;
                        break;
                    }
                }
            }
        }
    }

    ng_posts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_anchor_targets() {
        let body = ">>1 乙です。&gt;&gt;5 も参考になります。>10-15 も見た";
        let targets = extract_anchor_targets(body);
        assert_eq!(targets, vec![1, 5, 10]);
    }

    #[test]
    fn test_chain_abone() {
        let posts = vec![
            PostItem {
                number: 1,
                name: "スレ主".into(),
                mail: "".into(),
                date: "".into(),
                id: "good_id".into(),
                body: "本スレです".into(),
            },
            PostItem {
                number: 2,
                name: "荒らし".into(),
                mail: "".into(),
                date: "".into(),
                id: "troll_id".into(),
                body: "グロ注意".into(),
            },
            PostItem {
                number: 3,
                name: "一般人".into(),
                mail: "".into(),
                date: "".into(),
                id: "normal_id".into(),
                body: ">>2 通報しました".into(),
            },
            PostItem {
                number: 4,
                name: "一般人2".into(),
                mail: "".into(),
                date: "".into(),
                id: "normal_id2".into(),
                body: ">>3 乙".into(),
            },
        ];

        let settings = NGSettings {
            ng_words: vec![],
            ng_ids: vec!["troll_id".into()],
            ng_mode: NGMode::Abone,
            chain_abone: true,
        };

        let ng_set = calculate_ng_posts(&posts, &settings);
        assert!(ng_set.contains(&2)); // 直接NG
        assert!(ng_set.contains(&3)); // >>2 への返信なので連鎖NG
        assert!(ng_set.contains(&4)); // >>3 への返信なので連鎖NG
        assert!(!ng_set.contains(&1));
    }
}
