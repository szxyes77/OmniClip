use pinyin::ToPinyin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;

use crate::database::models::ClipboardRecord;
use crate::encryption::EncryptionManager;

#[derive(Debug, Clone)]
struct IndexedEntry {
    id: String,
    content: String,
    content_lower: String,
    pinyin_full: String,
    pinyin_initials: String,
    last_used_at: Option<String>,
}

pub struct SearchEngine {
    index: Arc<RwLock<HashMap<String, IndexedEntry>>>,
    app_handle: AppHandle,
    encryption: Arc<std::sync::Mutex<EncryptionManager>>,
}

impl SearchEngine {
    pub fn new(app_handle: AppHandle, encryption: Arc<std::sync::Mutex<EncryptionManager>>) -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
            encryption,
        }
    }

    pub async fn build_index(&self, records: Vec<ClipboardRecord>) {
        let mut index = self.index.write().await;
        index.clear();

        let encryption = self.encryption.lock().unwrap();

        for record in records {
            let decrypted_content = if record.record_type == "text" {
                encryption.decrypt_string(&record.content).unwrap_or_default()
            } else if record.record_type == "image" {
                record.title.clone()
            } else if record.record_type == "files" {
                encryption.decrypt_string(&record.content)
                    .ok()
                    .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                    .map(|paths| paths.join(" "))
                    .unwrap_or_else(|| record.title.clone())
            } else {
                record.title.clone()
            };

            let content_lower = decrypted_content.to_lowercase();
            let pinyin_full = text_to_pinyin(&decrypted_content);
            let pinyin_initials = text_to_pinyin_initials(&decrypted_content);

            index.insert(
                record.id.clone(),
                IndexedEntry {
                    id: record.id,
                    content: decrypted_content,
                    content_lower,
                    pinyin_full,
                    pinyin_initials,
                    last_used_at: record.last_used_at,
                },
            );
        }
    }

    pub async fn add_to_index(&self, record: ClipboardRecord) {
        let encryption = self.encryption.lock().unwrap();

        let decrypted_content = if record.record_type == "text" {
            encryption.decrypt_string(&record.content).unwrap_or_default()
        } else if record.record_type == "image" {
            record.title.clone()
        } else if record.record_type == "files" {
            encryption.decrypt_string(&record.content)
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                .map(|paths| paths.join(" "))
                .unwrap_or_else(|| record.title.clone())
        } else {
            record.title.clone()
        };

        let content_lower = decrypted_content.to_lowercase();
        let pinyin_full = text_to_pinyin(&decrypted_content);
        let pinyin_initials = text_to_pinyin_initials(&decrypted_content);

        let mut index = self.index.write().await;
        index.insert(
            record.id.clone(),
            IndexedEntry {
                id: record.id,
                content: decrypted_content,
                content_lower,
                pinyin_full,
                pinyin_initials,
                last_used_at: record.last_used_at,
            },
        );
    }

    pub async fn remove_from_index(&self, id: &str) {
        let mut index = self.index.write().await;
        index.remove(id);
    }

    pub async fn search(&self, query: &str, limit: u32) -> Vec<ClipboardRecord> {
        if query.trim().is_empty() {
            return vec![];
        }

        let index = self.index.read().await;
        let query_lower = query.to_lowercase();
        let query_pinyin = text_to_pinyin(query);
        let query_initials = text_to_pinyin_initials(query);

        let mut scored_entries: Vec<(String, f64)> = Vec::new();

        for (id, entry) in index.iter() {
            let mut max_score = 0.0;

            if entry.content_lower == query_lower {
                max_score = 100.0;
            } else if entry.content_lower.starts_with(&query_lower) {
                max_score = 80.0;
            } else if entry.content_lower.contains(&query_lower) {
                let content_len = entry.content_lower.len() as f64;
                let query_len = query_lower.len() as f64;
                let ratio = query_len / content_len;
                max_score = 60.0 + (ratio * 10.0);
            }

            if max_score < 60.0 {
                if !query_pinyin.is_empty() && entry.pinyin_full.contains(&query_pinyin) {
                    max_score = 50.0;
                }
            }

            if max_score < 50.0 {
                if !query_initials.is_empty() && entry.pinyin_initials.contains(&query_initials) {
                    max_score = 40.0;
                }
            }

            if max_score > 0.0 {
                let recency_score = calculate_recency_score(&entry.last_used_at);
                let final_score = max_score * 0.7 + recency_score * 0.3;
                scored_entries.push((id.clone(), final_score));
            }
        }

        scored_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let limit = limit as usize;
        let top_ids: Vec<&String> = scored_entries
            .iter()
            .take(limit)
            .map(|(id, _)| id)
            .collect();

        let state = self.app_handle.state::<crate::AppState>();
        let db = state.db.lock().map_err(|e| e.to_string()).unwrap();

        let mut results = Vec::new();
        for id in top_ids {
            if let Ok(record) = db.get_record_by_id(id) {
                results.push(record);
            }
        }

        results
    }
}

fn text_to_pinyin(text: &str) -> String {
    text.to_pinyin()
        .filter_map(|word| word.map(|p| p.plain()))
        .collect::<Vec<&str>>()
        .join(" ")
}

fn text_to_pinyin_initials(text: &str) -> String {
    text.to_pinyin()
        .filter_map(|word| word.map(|p| p.first_letter().to_string()))
        .collect::<Vec<String>>()
        .join("")
}

fn calculate_recency_score(last_used_at: &Option<String>) -> f64 {
    if let Some(date_str) = last_used_at {
        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(date);
            let days = duration.num_days() as f64;
            if days <= 0.0 {
                return 100.0;
            }
            let score = 100.0 / (1.0 + days * 0.1);
            return score.max(0.0).min(100.0);
        }
    }
    50.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_to_pinyin() {
        let result = text_to_pinyin("你好");
        assert!(result.contains("ni"));
        assert!(result.contains("hao"));
    }

    #[test]
    fn test_text_to_pinyin_mixed() {
        let result = text_to_pinyin("Hello 世界");
        assert!(result.contains("shi"));
        assert!(result.contains("jie"));
    }

    #[test]
    fn test_text_to_pinyin_initials() {
        let result = text_to_pinyin_initials("你好世界");
        assert_eq!(result, "nhsj");
    }

    #[test]
    fn test_text_to_pinyin_initials_single_char() {
        let result = text_to_pinyin_initials("中");
        assert_eq!(result, "z");
    }

    #[test]
    fn test_calculate_recency_score_now() {
        let now = chrono::Utc::now().to_rfc3339();
        let score = calculate_recency_score(&Some(now));
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_recency_score_none() {
        let score = calculate_recency_score(&None);
        assert_eq!(score, 50.0);
    }

    #[test]
    fn test_calculate_recency_score_yesterday() {
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let score = calculate_recency_score(&Some(yesterday));
        assert!(score > 0.0 && score < 100.0);
    }

    #[test]
    fn test_calculate_recency_score_old() {
        let old = (chrono::Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let score = calculate_recency_score(&Some(old));
        assert!(score < 10.0);
    }

    #[test]
    fn test_search_empty_query_returns_empty() {
        let entries: HashMap<String, IndexedEntry> = HashMap::new();
        assert!(entries.is_empty());
    }
}
