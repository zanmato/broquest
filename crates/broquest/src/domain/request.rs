//! Core request and response data types

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::HttpMethod;

/// A key-value pair with an enabled flag, used for headers, query params, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl Default for KeyValuePair {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            enabled: true,
        }
    }
}

impl KeyValuePair {
    /// Check if two KeyValuePair vectors are equal (order-independent comparison).
    /// Filters out entries with empty keys and compares the remaining entries.
    pub fn vec_equals(left: &[KeyValuePair], right: &[KeyValuePair]) -> bool {
        // Filter out entries with empty keys (they don't represent real data)
        let left_filtered: Vec<_> = left.iter().filter(|p| !p.key.trim().is_empty()).collect();
        let right_filtered: Vec<_> = right.iter().filter(|p| !p.key.trim().is_empty()).collect();

        if left_filtered.len() != right_filtered.len() {
            return false;
        }

        // For each item in left, find a matching item in right
        for l in &left_filtered {
            if !right_filtered
                .iter()
                .any(|r| r.key == l.key && r.value == l.value && r.enabled == l.enabled)
            {
                return false;
            }
        }
        true
    }
}

/// Request data for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestData {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub path_params: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    pub headers: Vec<KeyValuePair>,
    pub body: String,
    pub pre_request_script: Option<String>,
    pub post_response_script: Option<String>,
}

impl Default for RequestData {
    fn default() -> Self {
        Self {
            name: "New Request".to_string(),
            method: HttpMethod::Get,
            url: String::new(),
            path_params: Vec::new(),
            query_params: Vec::new(),
            headers: Vec::new(),
            body: String::new(),
            pre_request_script: None,
            post_response_script: None,
        }
    }
}

/// Response data from HTTP requests
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResponseData {
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
    pub latency: Option<Duration>,
    pub size: Option<usize>,
    pub headers: Vec<KeyValuePair>,
    pub body: String,
    pub url: Option<String>,
}
