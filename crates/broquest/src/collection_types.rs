use crate::request_editor::{HttpMethod, KeyValuePair};
use serde::{Deserialize, Serialize};

/// TOML structure for collection.toml files
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CollectionToml {
    pub collection: CollectionMeta,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "environment")]
    pub environments: Vec<EnvironmentToml>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CollectionMeta {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    pub description: String,
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// TOML structure for request .toml files
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestToml {
    pub meta: RequestMeta,
    pub http: RequestHttp,
    pub script: Option<RequestScript>,
    pub headers: Option<Vec<HeaderToml>>,
    pub query: Option<Vec<QueryToml>>,
    pub body: Option<RequestBodyToml>,
    pub params: Option<RequestParams>,
}

/// TOML structure for request body
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestBodyToml {
    pub json: Option<String>,
    pub text: Option<String>,
    pub form: Option<std::collections::HashMap<String, String>>,
    pub graphql: Option<GraphQLBody>,
    pub xml: Option<String>,
}

/// TOML structure for GraphQL body
#[derive(Debug, Deserialize, Serialize)]
pub struct GraphQLBody {
    pub query: Option<String>,
    pub variables: Option<serde_json::Value>,
}

/// TOML structure for request parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestParams {
    pub path: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub request_type: String,
    pub seq: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestHttp {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
    pub auth: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestScript {
    #[serde(rename = "pre-request")]
    pub pre_request: Option<String>,
    #[serde(rename = "post-response")]
    pub post_response: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HeaderToml {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryToml {
    pub key: String,
    pub value: String,
}

/// TOML structure for environment files
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EnvironmentToml {
    pub name: String,
    #[serde(default)]
    pub variables: std::collections::HashMap<String, EnvironmentVariable>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EnvironmentVariable {
    pub value: String,
    pub secret: bool,
    #[serde(default, skip_serializing_if = "crate::collection_types::is_true")]
    pub temporary: bool,
}

impl EnvironmentVariable {
    /// Read a credential value from secure storage using GPUI's credential system
    /// Format: broquest://collection_name/environment_name/variable_name
    pub fn read_credential(
        collection_name: &str,
        environment_name: &str,
        variable_name: &str,
        cx: &gpui::App,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let credential_path = format!(
            "broquest://{}/{}/{}",
            collection_name, environment_name, variable_name
        );

        let read_task = cx.read_credentials(&credential_path);
        smol::block_on(async {
            match read_task.await? {
                Some((_, secret_bytes)) => Ok(Some(String::from_utf8(secret_bytes)?)),
                None => Ok(None),
            }
        })
    }

    /// Write a credential value to secure storage using GPUI's credential system
    /// Format: broquest://collection_name/environment_name/variable_name
    pub fn write_credential(
        collection_name: &str,
        environment_name: &str,
        variable_name: &str,
        value: &str,
        cx: &gpui::App,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let credential_path = format!(
            "broquest://{}/{}/{}",
            collection_name, environment_name, variable_name
        );
        let write_task = cx.write_credentials(&credential_path, variable_name, value.as_bytes());
        cx.spawn(async move |_| {
            if let Err(e) = write_task.await {
                tracing::error!("Failed to write credential '{}': {}", credential_path, e);
            } else {
                tracing::info!("Successfully wrote credential '{}'", credential_path);
            }
        })
        .detach();
        Ok(())
    }

    /// Delete a credential from secure storage
    pub fn delete_credential(
        collection_name: &str,
        environment_name: &str,
        variable_name: &str,
        cx: &mut gpui::App,
    ) {
        let credential_path = format!(
            "broquest://{}/{}/{}",
            collection_name, environment_name, variable_name
        );

        let delete_task = cx.delete_credentials(&credential_path);
        cx.spawn(async move |_| {
            if let Err(e) = delete_task.await {
                tracing::error!("Failed to delete credential '{}': {}", credential_path, e);
            } else {
                tracing::info!("Successfully deleted credential '{}'", credential_path);
            }
        })
        .detach();
    }
}

/// Helper function to skip serializing temporary=true variables
pub fn is_true(value: &bool) -> bool {
    *value
}

/// Convert TOML request to internal RequestData
impl From<RequestToml> for crate::request_editor::RequestData {
    fn from(toml: RequestToml) -> Self {
        let method = match toml.http.method.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            _ => HttpMethod::Get,
        };

        // Parse headers from TOML
        let headers = toml
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|h| KeyValuePair {
                key: h.key,
                value: h.value,
                enabled: true,
            })
            .collect();

        // Parse query params from TOML
        let query_params = toml
            .query
            .unwrap_or_default()
            .into_iter()
            .map(|q| KeyValuePair {
                key: q.key,
                value: q.value,
                enabled: true,
            })
            .collect();

        // Parse path params from TOML [params.path] section
        let path_params = if let Some(params) = toml.params {
            if let Some(path_params) = params.path {
                path_params
                    .into_iter()
                    .map(|(key, value)| KeyValuePair {
                        key,
                        value,
                        enabled: true,
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Extract script data
        let (pre_request_script, post_response_script) = if let Some(script) = toml.script {
            (script.pre_request, script.post_response)
        } else {
            (None, None)
        };

        // Extract body content from [body] section based on http.body type
        let body = match toml.http.body.as_deref() {
            Some("none") => String::new(), // Skip body entirely when http.body is "none"
            Some(body_type) => {
                if let Some(body_section) = toml.body {
                    match body_type {
                        "json" => body_section.json.unwrap_or_default(),
                        "text" => body_section.text.unwrap_or_default(),
                        "xml" => body_section.xml.unwrap_or_default(),
                        "form" => {
                            // Convert form data to URL-encoded string
                            body_section
                                .form
                                .map(|form_map| {
                                    form_map
                                        .iter()
                                        .map(|(k, v)| {
                                            format!(
                                                "{}={}",
                                                urlencoding::encode(k),
                                                urlencoding::encode(v)
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join("&")
                                })
                                .unwrap_or_default()
                        }
                        "graphql" => {
                            // Convert GraphQL to JSON string
                            body_section.graphql
                                .map(|g| serde_json::json!({
                                    "query": g.query.unwrap_or_default(),
                                    "variables": g.variables.unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
                                }).to_string())
                                .unwrap_or_default()
                        }
                        _ => String::new(),
                    }
                } else {
                    // If no body section but body_type is not "none", use the raw http.body value (for backward compatibility)
                    toml.http.body.unwrap_or_default()
                }
            }
            None => String::new(),
        };

        crate::request_editor::RequestData {
            name: toml.meta.name,
            method,
            url: toml.http.url,
            path_params,
            body,
            headers,
            query_params,
            pre_request_script,
            post_response_script,
        }
    }
}

/// Convert internal RequestData to TOML request
impl From<crate::request_editor::RequestData> for RequestToml {
    fn from(data: crate::request_editor::RequestData) -> Self {
        // Detect body type from headers before we consume headers
        let body_type = if !data.body.is_empty() {
            Some(get_body_type_from_headers(&data.headers))
        } else {
            Some("none".to_string())
        };

        // Convert headers to TOML format
        let headers = if data.headers.is_empty() {
            None
        } else {
            Some(
                data.headers
                    .into_iter()
                    .map(|h| HeaderToml {
                        key: h.key,
                        value: h.value,
                    })
                    .collect(),
            )
        };

        // Convert query params to TOML format
        let query = if data.query_params.is_empty() {
            None
        } else {
            Some(
                data.query_params
                    .into_iter()
                    .map(|q| QueryToml {
                        key: q.key,
                        value: q.value,
                    })
                    .collect(),
            )
        };

        // Convert path params to TOML format
        let params = if data.path_params.is_empty() {
            None
        } else {
            let path_params: std::collections::HashMap<String, String> = data
                .path_params
                .into_iter()
                .filter(|p| p.enabled && !p.key.is_empty())
                .map(|p| (p.key, p.value))
                .collect();

            if path_params.is_empty() {
                None
            } else {
                Some(RequestParams {
                    path: Some(path_params),
                })
            }
        };

        RequestToml {
            meta: RequestMeta {
                name: data.name,
                request_type: "http".to_string(),
                seq: "1".to_string(),
            },
            http: RequestHttp {
                method: data.method.as_str().to_string(),
                url: data.url,
                body: body_type.clone(),
                auth: "none".to_string(),
            },
            script: if data.pre_request_script.is_some() || data.post_response_script.is_some() {
                Some(RequestScript {
                    pre_request: data.pre_request_script,
                    post_response: data.post_response_script,
                })
            } else {
                None
            },
            headers,
            query,
            body: if data.body.is_empty() {
                None
            } else {
                // Create body section based on detected content type
                create_body_toml(
                    &data.body,
                    body_type.as_ref().unwrap_or(&"json".to_string()),
                )
            },
            params,
        }
    }
}

/// Get body type string from headers (Content-Type)
fn get_body_type_from_headers(headers: &[crate::request_editor::KeyValuePair]) -> String {
    for header in headers {
        if header.key.to_lowercase() == "content-type" && header.enabled {
            let content_type = header.value.to_lowercase();
            if content_type.contains("application/json") {
                return "json".to_string();
            } else if content_type.contains("text/xml") || content_type.contains("application/xml")
            {
                return "xml".to_string();
            } else if content_type.contains("application/graphql") {
                return "graphql".to_string();
            } else if content_type.contains("application/x-www-form-urlencoded") {
                return "form".to_string();
            } else if content_type.contains("text/") {
                return "text".to_string();
            }
        }
    }
    // Default to "json" if no content type is found
    "json".to_string()
}

/// Create RequestBodyToml based on body type
fn create_body_toml(body: &str, body_type: &str) -> Option<RequestBodyToml> {
    match body_type {
        "json" => Some(RequestBodyToml {
            json: Some(body.to_string()),
            text: None,
            form: None,
            graphql: None,
            xml: None,
        }),
        "text" => Some(RequestBodyToml {
            json: None,
            text: Some(body.to_string()),
            form: None,
            graphql: None,
            xml: None,
        }),
        "xml" => Some(RequestBodyToml {
            json: None,
            text: None,
            form: None,
            graphql: None,
            xml: Some(body.to_string()),
        }),
        "form" => {
            // Try to parse URL-encoded form data into a HashMap
            let mut form_map = std::collections::HashMap::new();
            for pair in body.split('&') {
                if let Some((key, value)) = pair.split_once('=')
                    && let (Ok(decoded_key), Ok(decoded_value)) =
                        (urlencoding::decode(key), urlencoding::decode(value))
                    {
                        form_map.insert(decoded_key.into_owned(), decoded_value.into_owned());
                    }
            }
            Some(RequestBodyToml {
                json: None,
                text: None,
                form: Some(form_map),
                graphql: None,
                xml: None,
            })
        }
        "graphql" => {
            // Try to parse GraphQL JSON into query and variables
            if let Ok(graphql_json) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(obj) = graphql_json.as_object() {
                    Some(RequestBodyToml {
                        json: None,
                        text: None,
                        form: None,
                        graphql: Some(GraphQLBody {
                            query: obj
                                .get("query")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            variables: obj.get("variables").cloned(),
                        }),
                        xml: None,
                    })
                } else {
                    // If it's not an object, treat as plain text
                    Some(RequestBodyToml {
                        json: None,
                        text: Some(body.to_string()),
                        form: None,
                        graphql: None,
                        xml: None,
                    })
                }
            } else {
                // If parsing fails, treat as plain text
                Some(RequestBodyToml {
                    json: None,
                    text: Some(body.to_string()),
                    form: None,
                    graphql: None,
                    xml: None,
                })
            }
        }
        _ => Some(RequestBodyToml {
            json: Some(body.to_string()), // Default to JSON for unknown types
            text: None,
            form: None,
            graphql: None,
            xml: None,
        }),
    }
}

/// Create an empty collection with default values
pub fn create_empty_collection() -> CollectionToml {
    CollectionToml {
        collection: CollectionMeta {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            collection_type: "collection".to_string(),
            description: "".to_string(),
            ignore: Vec::new(),
        },
        environments: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_serialize_empty_environments() {
        let collection = CollectionToml {
            collection: CollectionMeta {
                name: "Test Collection".to_string(),
                version: "1.0.0".to_string(),
                collection_type: "collection".to_string(),
                description: "Test Description".to_string(),
                ignore: Vec::new(),
            },
            environments: vec![],
        };

        let toml_string = toml::to_string(&collection).expect("Failed to serialize collection");

        // The result should NOT contain any environment section when environments is empty
        assert!(
            !toml_string.contains("environment"),
            "TOML should not contain 'environment' key when environments is empty"
        );
        assert!(
            !toml_string.contains("[environment]"),
            "TOML should not contain '[environment]' section when environments is empty"
        );

        // It should be possible to deserialize the result back
        let deserialized: CollectionToml =
            toml::from_str(&toml_string).expect("Failed to deserialize");
        assert_eq!(deserialized.environments.len(), 0);
    }

    #[test]
    fn test_serialize_non_empty_environments() {
        let collection = CollectionToml {
            collection: CollectionMeta {
                name: "Test Collection".to_string(),
                version: "1.0.0".to_string(),
                collection_type: "collection".to_string(),
                description: "Test Description".to_string(),
                ignore: Vec::new(),
            },
            environments: vec![EnvironmentToml {
                name: "Development".to_string(),
                variables: std::collections::HashMap::new(),
            }],
        };

        let toml_string = toml::to_string(&collection).expect("Failed to serialize collection");

        // The result should contain the environment section when environments is not empty
        assert!(toml_string.contains("[[environment]]"));
        assert!(toml_string.contains("Development"));

        // It should be possible to deserialize the result back
        let deserialized: CollectionToml =
            toml::from_str(&toml_string).expect("Failed to deserialize");
        assert_eq!(deserialized.environments.len(), 1);
        assert_eq!(deserialized.environments[0].name, "Development");
    }

    #[test]
    fn test_content_type_detection() {
        let request_data_json = crate::request_editor::RequestData {
            name: "JSON Request".to_string(),
            method: crate::request_editor::HttpMethod::Post,
            url: "https://api.example.com".to_string(),
            path_params: vec![],
            body: r#"{"key": "value"}"#.to_string(),
            headers: vec![crate::request_editor::KeyValuePair {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                enabled: true,
            }],
            query_params: vec![],
            pre_request_script: None,
            post_response_script: None,
        };

        let toml_request: RequestToml = request_data_json.into();
        assert_eq!(toml_request.http.body.unwrap(), "json");
        assert!(toml_request.body.is_some());
        assert!(toml_request.body.unwrap().json.is_some());

        let request_data_form = crate::request_editor::RequestData {
            name: "Form Request".to_string(),
            method: crate::request_editor::HttpMethod::Post,
            url: "https://api.example.com".to_string(),
            path_params: vec![],
            body: "key=value&foo=bar".to_string(),
            headers: vec![crate::request_editor::KeyValuePair {
                key: "Content-Type".to_string(),
                value: "application/x-www-form-urlencoded".to_string(),
                enabled: true,
            }],
            query_params: vec![],
            pre_request_script: None,
            post_response_script: None,
        };

        let toml_form_request: RequestToml = request_data_form.into();
        assert_eq!(toml_form_request.http.body.unwrap(), "form");
        assert!(toml_form_request.body.is_some());
        assert!(toml_form_request.body.unwrap().form.is_some());
    }
}
