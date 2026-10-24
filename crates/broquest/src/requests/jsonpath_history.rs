//! Recent JSONPath response filters, newest first, kept between launches.

use gpui::{App, Global};

use crate::app_database::AppDatabase;

/// The `app_state` key the filters are saved under, as a JSON array.
const STATE_KEY: &str = "jsonpath_history";

/// How many filters are kept.
const MAX_ENTRIES: usize = 20;

/// Filters that show what JSONPath can do, offered before there is history.
pub const EXAMPLES: &[(&str, &str)] = &[
    ("$..id", "Every id, at any depth"),
    ("$.items[*].name", "The name of each item"),
    ("$.items[0]", "The first item"),
    ("$.items[?@.price < 10]", "Items cheaper than 10"),
];

#[derive(Default)]
pub struct JsonPathHistory {
    entries: Vec<String>,
}

impl Global for JsonPathHistory {}

impl JsonPathHistory {
    /// Read the saved filters. Called once at startup, after the database.
    pub fn load(cx: &mut App) {
        let db = AppDatabase::global(cx).clone();
        let entries = match smol::block_on(async move { db.load_state(STATE_KEY).await }) {
            Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
            Ok(None) => Vec::new(),
            Err(error) => {
                tracing::warn!("Failed to load the JSONPath filter history: {error}");
                Vec::new()
            }
        };
        cx.set_global(Self { entries });
    }

    /// The recent filters, newest first.
    pub fn entries(cx: &App) -> Vec<String> {
        cx.try_global::<Self>()
            .map(|history| history.entries.clone())
            .unwrap_or_default()
    }

    /// Remember `query` as the newest filter.
    pub fn record(query: &str, cx: &mut App) {
        let history = cx.default_global::<Self>();
        if !push(&mut history.entries, query) {
            return;
        }
        Self::save(cx);
    }

    /// Forget every filter.
    pub fn clear(cx: &mut App) {
        cx.default_global::<Self>().entries.clear();
        Self::save(cx);
    }

    fn save(cx: &mut App) {
        let Some(db) = cx.try_global::<AppDatabase>().cloned() else {
            return;
        };
        let json = serde_json::to_string(&cx.global::<Self>().entries).unwrap_or_default();
        cx.background_executor()
            .spawn(async move {
                if let Err(error) = db.save_state(STATE_KEY, &json).await {
                    tracing::warn!("Failed to save the JSONPath filter history: {error}");
                }
            })
            .detach();
    }
}

/// Put `query` first in `entries`, once, keeping at most [`MAX_ENTRIES`].
/// Returns whether the entries changed.
fn push(entries: &mut Vec<String>, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() || entries.first().is_some_and(|first| first == query) {
        return false;
    }
    entries.retain(|entry| entry != query);
    entries.insert(0, query.to_string());
    entries.truncate(MAX_ENTRIES);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonpath_rust::JsonPath as _;

    #[test]
    fn newest_first_without_repeats() {
        let mut entries = Vec::new();
        assert!(push(&mut entries, "$.a"));
        assert!(push(&mut entries, " $.b "));
        assert!(!push(&mut entries, "$.b"), "already first");
        assert!(!push(&mut entries, "  "));
        assert!(push(&mut entries, "$.a"));
        assert_eq!(entries, ["$.a", "$.b"]);
    }

    #[test]
    fn keeps_the_newest_twenty() {
        let mut entries = Vec::new();
        for n in 0..25 {
            push(&mut entries, &format!("$[{n}]"));
        }
        assert_eq!(entries.len(), MAX_ENTRIES);
        assert_eq!(entries[0], "$[24]");
        assert_eq!(entries[MAX_ENTRIES - 1], "$[5]");
    }

    #[test]
    fn examples_are_valid_jsonpath() {
        let json = serde_json::json!({
            "id": 1,
            "items": [{"id": 2, "name": "a", "price": 5}, {"id": 3, "name": "b", "price": 20}]
        });
        let count = |query: &str| json.query(query).map(|found| found.len());
        assert_eq!(count(EXAMPLES[0].0), Ok(3));
        assert_eq!(count(EXAMPLES[1].0), Ok(2));
        assert_eq!(count(EXAMPLES[2].0), Ok(1));
        assert_eq!(count(EXAMPLES[3].0), Ok(1));
    }
}
