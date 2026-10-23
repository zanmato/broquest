//! The workspace state restored on the next launch: open tabs, the active
//! tab and which sidebar folders are expanded.
//!
//! Tabs point at what is saved on disk, never at unsaved edits. A tab whose
//! request, group or collection no longer exists is skipped on restore.

use serde::{Deserialize, Serialize};

/// Key of the session row in the `app_state` table.
pub const SESSION_KEY: &str = "session";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub tabs: Vec<SessionTab>,
    /// Index into `tabs` of the tab that was active.
    #[serde(default)]
    pub active_tab: usize,
    /// Ids of the expanded collections and groups in the sidebar.
    #[serde(default)]
    pub expanded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionTab {
    Request {
        collection_path: String,
        /// Path of the group, relative to the collection root.
        #[serde(default)]
        group_path: Option<String>,
        /// The request's saved name, which is how it is found on disk.
        name: String,
        /// The environment the tab had selected.
        #[serde(default)]
        environment: Option<String>,
    },
    Collection {
        collection_path: String,
    },
    Group {
        collection_path: String,
        group_name: String,
    },
    Settings,
}

impl Session {
    /// Parses a stored session. Anything unreadable starts a fresh session
    /// instead of failing the launch.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_else(|error| {
            tracing::warn!("Ignoring unreadable session state: {}", error);
            Self::default()
        })
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let session = Session {
            tabs: vec![
                SessionTab::Request {
                    collection_path: "/work/api".into(),
                    group_path: Some("users".into()),
                    name: "List users".into(),
                    environment: Some("Local".into()),
                },
                SessionTab::Collection {
                    collection_path: "/work/api".into(),
                },
                SessionTab::Group {
                    collection_path: "/work/api".into(),
                    group_name: "users".into(),
                },
                SessionTab::Settings,
            ],
            active_tab: 2,
            expanded: vec!["collection__work_api".into()],
        };

        assert_eq!(Session::from_json(&session.to_json()), session);
    }

    #[test]
    fn unreadable_state_starts_empty() {
        assert_eq!(Session::from_json("not json"), Session::default());
        assert_eq!(
            Session::from_json(r#"{"tabs":[{"kind":"unknown"}]}"#),
            Session::default()
        );
    }

    #[test]
    fn missing_fields_use_defaults() {
        let session = Session::from_json(
            r#"{"tabs":[{"kind":"request","collection_path":"/a","name":"Ping"}]}"#,
        );
        assert_eq!(
            session.tabs,
            vec![SessionTab::Request {
                collection_path: "/a".into(),
                group_path: None,
                name: "Ping".into(),
                environment: None,
            }]
        );
        assert_eq!(session.active_tab, 0);
        assert!(session.expanded.is_empty());
    }
}
