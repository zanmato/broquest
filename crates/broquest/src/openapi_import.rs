use crate::collection_types::{EnvironmentToml, EnvironmentVariable};
use crate::http::HttpMethod;
use crate::request_editor::KeyValuePair;
use crate::request_editor::RequestData;
use oas3::spec::{ObjectOrReference, Operation, ParameterIn, SchemaType};
use serde_json::json;
use std::collections::BTreeMap;

pub struct OpenAPIImporter {
    spec: oas3::spec::Spec,
    base_url: String,
}

impl OpenAPIImporter {
    /// Parse OpenAPI spec from file path
    pub fn from_path(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the file content
        let content = std::fs::read_to_string(path)?;

        // Try to parse as YAML first, then JSON
        let spec = if path.ends_with(".json") {
            oas3::from_json(content)?
        } else {
            // Default to YAML (also handles .yaml, .yml files)
            oas3::from_yaml(content)?
        };

        let base_url = Self::extract_base_url(&spec);
        Ok(Self { spec, base_url })
    }

    fn extract_base_url(spec: &oas3::spec::Spec) -> String {
        spec.servers
            .first()
            .map(|s| s.url.clone())
            .unwrap_or_else(|| "http://localhost".to_string())
    }

    /// Create a Default environment with baseUrl variable
    pub fn create_environment(&self) -> EnvironmentToml {
        let mut variables = std::collections::HashMap::new();
        variables.insert(
            "baseUrl".to_string(),
            EnvironmentVariable {
                value: self.base_url.clone(),
                secret: false,
                temporary: false,
            },
        );

        EnvironmentToml {
            name: "Default".to_string(),
            variables,
        }
    }

    /// Import spec as collection groups and requests
    pub fn import(&self) -> Result<ImportResult, Box<dyn std::error::Error>> {
        let mut groups = Vec::new();
        let mut requests = Vec::new();

        // Group operations by tag (if present) or by path prefix
        let mut grouped: BTreeMap<Option<String>, Vec<OperationInfo>> = BTreeMap::new();

        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths.iter() {
                for (method, operation) in path_item.methods() {
                    let tags = operation.tags.clone();
                    let group_name = tags.first().cloned();

                    let info = OperationInfo {
                        method: method.to_string(),
                        path: path.clone(),
                        operation: operation.clone(),
                    };

                    grouped.entry(group_name).or_default().push(info);
                }
            }
        }

        // Convert to groups and requests
        for (group_name, operations) in grouped {
            let group_requests: Vec<RequestData> = operations
                .iter()
                .map(|op| self.create_request(op))
                .collect();

            if let Some(name) = group_name {
                groups.push((name, group_requests));
            } else {
                // No tag, add to root
                requests.extend(group_requests);
            }
        }

        Ok(ImportResult {
            environment: self.create_environment(),
            groups,
            requests,
        })
    }

    fn create_request(&self, op_info: &OperationInfo) -> RequestData {
        let method = match op_info.method.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => HttpMethod::Get,
        };

        // Convert path params from {param} to :param format
        let converted_path = convert_path_params_format(&op_info.path);

        // Use {{baseUrl}} variable instead of actual base URL
        let url = format!("{{{{baseUrl}}}}{}", converted_path);
        let name = op_info.operation.operation_id.clone().unwrap_or_else(|| {
            format!(
                "{} {}",
                op_info.method.to_uppercase(),
                sanitize_path_name(&op_info.path)
            )
        });

        let mut headers = Vec::new();
        let mut query_params = Vec::new();
        let mut path_params = Vec::new();
        let mut cookie_params = Vec::new();

        // Extract parameters
        for param in &op_info.operation.parameters {
            if let Ok(param) = param.resolve(&self.spec) {
                let kv = KeyValuePair {
                    key: param.name.clone(),
                    value: String::new(),
                    enabled: true,
                };

                // Use pattern matching on ParameterIn enum
                match param.location {
                    ParameterIn::Header => headers.push(kv),
                    ParameterIn::Query => query_params.push(kv),
                    ParameterIn::Path => path_params.push(kv),
                    ParameterIn::Cookie => {
                        // Collect cookie params to add as Cookie header
                        cookie_params.push(kv);
                    }
                }
            }
        }

        // Add cookie parameters as a Cookie header
        if !cookie_params.is_empty() {
            let cookie_value = cookie_params
                .iter()
                .map(|p| format!("{}={}", p.key, p.value))
                .collect::<Vec<_>>()
                .join("; ");
            headers.push(KeyValuePair {
                key: "Cookie".to_string(),
                value: cookie_value,
                enabled: true,
            });
        }

        // Extract request body if present
        let body = self.extract_request_body(&op_info.operation, &op_info.method);

        RequestData {
            name,
            method,
            url,
            path_params,
            query_params,
            headers,
            body,
            pre_request_script: None,
            post_response_script: None,
        }
    }

    fn extract_request_body(&self, operation: &Operation, method: &str) -> String {
        if let Some(request_body_ref) = &operation.request_body
            && let Ok(request_body) = request_body_ref.resolve(&self.spec)
        {
            // Check for any JSON-like content type or wildcard
            for (content_type, content) in &request_body.content {
                // Check for JSON or wildcard content types
                if content_type.contains("json")
                    || content_type.contains("*/*")
                    || content_type.contains("+json")
                {
                    // Try to generate sample from schema
                    if let Some(schema_ref) = &content.schema
                        && let Ok(sample) = self.generate_sample_json_from_schema_ref(schema_ref)
                    {
                        return sample;
                    }
                    // Return placeholder if no schema or generation failed
                    return "{}".to_string();
                }
            }
        }

        // Return empty string for methods that typically don't have bodies
        // Return a placeholder for methods that typically have bodies
        if matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
            // Return a placeholder JSON object
            "{}".to_string()
        } else {
            String::new()
        }
    }

    /// Generate a JSON string sample from an ObjectOrReference<ObjectSchema>
    fn generate_sample_json_from_schema_ref(
        &self,
        schema_ref: &ObjectOrReference<oas3::spec::ObjectSchema>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let value = self.generate_value_from_object_schema_ref(schema_ref)?;
        Ok(serde_json::to_string_pretty(&value)?)
    }

    fn resolve_schema_and_generate_sample(
        &self,
        schema_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let components = self
            .spec
            .components
            .as_ref()
            .ok_or("No components in spec")?;

        let schema_ref = components
            .schemas
            .get(schema_name)
            .ok_or(format!("Schema '{}' not found", schema_name))?;

        match schema_ref {
            ObjectOrReference::Ref { ref_path, .. } => {
                // Handle nested references
                let nested_name = ref_path
                    .split('/')
                    .next_back()
                    .ok_or("Invalid nested ref")?;
                self.resolve_schema_and_generate_sample(nested_name)
            }
            ObjectOrReference::Object(schema) => self.generate_sample_from_object_schema(schema),
        }
    }

    fn generate_sample_from_object_schema(
        &self,
        schema: &oas3::spec::ObjectSchema,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check if this is an object by looking at properties
        let has_properties = !schema.properties.is_empty();

        let value = if has_properties {
            let mut map = serde_json::Map::new();
            for (key, prop_ref) in &schema.properties {
                let prop_value = self.generate_value_from_object_schema_ref(prop_ref)?;
                map.insert(key.clone(), prop_value);
            }
            json!(map)
        } else {
            json!({})
        };

        Ok(serde_json::to_string_pretty(&value)?)
    }

    /// Generate a sample value from an ObjectOrReference<ObjectSchema>
    /// This handles both direct schemas and references to other schemas
    fn generate_value_from_object_schema_ref(
        &self,
        schema_ref: &ObjectOrReference<oas3::spec::ObjectSchema>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match schema_ref {
            ObjectOrReference::Ref { ref_path, .. } => {
                let schema_name = ref_path.split('/').next_back().ok_or("Invalid ref")?;
                let sample = self.resolve_schema_and_generate_sample(schema_name)?;
                Ok(serde_json::from_str(&sample)?)
            }
            ObjectOrReference::Object(schema) => {
                // Check the schema type to add appropriate sample values
                match &schema.schema_type {
                    Some(type_set) => {
                        // SchemaTypeSet can be Single(SchemaType) or Multiple(Vec<SchemaType>)
                        match type_set {
                            oas3::spec::SchemaTypeSet::Single(schema_type) => match schema_type {
                                SchemaType::String => {
                                    // Add sample string based on format
                                    match schema.format.as_deref() {
                                        Some("date") => Ok(serde_json::json!("2024-01-01")),
                                        Some("date-time") => {
                                            Ok(serde_json::json!("2024-01-01T00:00:00Z"))
                                        }
                                        Some("email") => {
                                            Ok(serde_json::json!("example@example.com"))
                                        }
                                        Some("uuid") => Ok(serde_json::json!(
                                            "550e8400-e29b-41d4-a716-446655440000"
                                        )),
                                        Some("uri") | Some("url") => {
                                            Ok(serde_json::json!("https://example.com"))
                                        }
                                        Some("hostname") => Ok(serde_json::json!("example.com")),
                                        Some("ipv4") => Ok(serde_json::json!("192.168.1.1")),
                                        Some("ipv6") => Ok(serde_json::json!("::1")),
                                        Some("byte") => Ok(serde_json::json!(
                                            "VGhpcyBpcyBhIGJhc2U2NCBleGFtcGxl"
                                        )),
                                        _ => Ok(serde_json::json!("string")),
                                    }
                                }
                                SchemaType::Integer => Ok(serde_json::json!(0)),
                                SchemaType::Number => Ok(serde_json::json!(0.0)),
                                SchemaType::Boolean => Ok(serde_json::json!(true)),
                                SchemaType::Array => {
                                    // Handle array with items
                                    if let Some(items_schema) = &schema.items {
                                        match items_schema.as_ref() {
                                            oas3::spec::Schema::Object(obj_ref) => {
                                                let item_value = self
                                                    .generate_value_from_object_schema_ref(
                                                        obj_ref,
                                                    )?;
                                                Ok(serde_json::json!([item_value]))
                                            }
                                            oas3::spec::Schema::Boolean(_) => {
                                                Ok(serde_json::json!([]))
                                            }
                                        }
                                    } else {
                                        Ok(serde_json::json!([]))
                                    }
                                }
                                SchemaType::Object => {
                                    // Nested object, generate from properties
                                    let sample = self.generate_sample_from_object_schema(schema)?;
                                    Ok(serde_json::from_str(&sample)?)
                                }
                                SchemaType::Null => Ok(serde_json::json!(null)),
                            },
                            oas3::spec::SchemaTypeSet::Multiple(types) => {
                                // Multiple possible types, use the first non-null one
                                types
                                    .iter()
                                    .find(|t| !matches!(t, SchemaType::Null))
                                    .map(|_| Ok(serde_json::json!(null)))
                                    .unwrap_or_else(|| Ok(serde_json::json!(null)))
                            }
                        }
                    }
                    None => {
                        // No type specified, default to empty object
                        Ok(serde_json::json!({}))
                    }
                }
            }
        }
    }
}

struct OperationInfo {
    method: String,
    path: String,
    operation: Operation,
}

pub struct ImportResult {
    pub environment: EnvironmentToml,
    pub groups: Vec<(String, Vec<RequestData>)>,
    pub requests: Vec<RequestData>,
}

fn sanitize_path_name(path: &str) -> String {
    path.trim_start_matches('/')
        .replace('/', "-")
        .replace(['{', '}'], "")
        .to_string()
}

/// Convert OpenAPI path param format {param} to :param format
fn convert_path_params_format(path: &str) -> String {
    let mut result = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            // Found start of path param, skip to closing brace
            let mut param_name = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == '}' {
                    chars.next(); // consume the closing brace
                    break;
                }
                param_name.push(chars.next().unwrap());
            }
            // Convert to :param format
            result.push(':');
            result.push_str(&param_name);
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_path_params_format() {
        // Single param
        assert_eq!(convert_path_params_format("/pets/{id}"), "/pets/:id");
        assert_eq!(
            convert_path_params_format("/users/{userId}"),
            "/users/:userId"
        );

        // Multiple params
        assert_eq!(
            convert_path_params_format("/pets/{petId}/owner/{ownerId}"),
            "/pets/:petId/owner/:ownerId"
        );

        // No params
        assert_eq!(convert_path_params_format("/pets"), "/pets");
        assert_eq!(convert_path_params_format("/api/v1/users"), "/api/v1/users");

        // Mixed path with params
        assert_eq!(
            convert_path_params_format("/api/v1/pets/{petId}/comments/{commentId}"),
            "/api/v1/pets/:petId/comments/:commentId"
        );

        // Trailing slash
        assert_eq!(convert_path_params_format("/pets/{id}/"), "/pets/:id/");
    }

    #[test]
    fn test_parse_petstore_spec() {
        let path = "resources/petstore-v3.1.json";
        let result = std::fs::read_to_string(path);
        assert!(
            result.is_ok(),
            "Failed to read petstore spec file from: {:?}",
            path
        );

        let content = result.unwrap();
        let spec = oas3::from_json(content);
        assert!(spec.is_ok(), "Failed to parse petstore spec");

        let spec = spec.unwrap();

        // Assert basic metadata
        assert_eq!(spec.info.title, "Swagger Petstore");
        assert_eq!(spec.info.version, "1.0.0");

        // Assert servers exist and have expected URL
        assert!(!spec.servers.is_empty());
        assert_eq!(spec.servers[0].url, "http://petstore.swagger.io/v1");

        // Assert paths exist
        let paths = spec.paths.as_ref().expect("No paths found");
        assert!(paths.contains_key("/pets"));
        assert!(paths.contains_key("/pets/{petId}"));

        // Assert components/schemas exist
        let components = spec.components.as_ref().expect("No components found");
        assert!(components.schemas.contains_key("Pet"));
        assert!(components.schemas.contains_key("Pets"));
        assert!(components.schemas.contains_key("Error"));

        // Assert Pet schema structure
        let pet_schema = components.schemas.get("Pet").unwrap();
        let ObjectOrReference::Object(pet) = pet_schema else {
            panic!("Pet schema is not an Object");
        };
        assert_eq!(pet.properties.len(), 4); // id, name, tag, createdAt
        assert!(pet.properties.contains_key("id"));
        assert!(pet.properties.contains_key("name"));
        assert!(pet.properties.contains_key("tag"));
        assert!(pet.properties.contains_key("createdAt"));

        // Assert there's an operation with requestBody (POST /pets)
        let pets_path = paths.get("/pets").unwrap();
        let post_operation = pets_path.post.as_ref().expect("No POST operation on /pets");
        assert!(
            post_operation.request_body.is_some(),
            "POST /pets should have a requestBody"
        );

        // Assert the requestBody has the correct schema reference
        let request_body_ref = post_operation.request_body.as_ref().unwrap();
        let request_body = request_body_ref
            .resolve(&spec)
            .expect("Failed to resolve request body");
        assert!(
            request_body.content.contains_key("application/json"),
            "requestBody should have application/json content type"
        );

        let json_content = request_body.content.get("application/json").unwrap();
        let schema_ref = json_content
            .schema
            .as_ref()
            .expect("No schema in content type");
        let ObjectOrReference::Ref { ref_path, .. } = schema_ref else {
            panic!("Expected a Ref schema, got inline schema");
        };
        assert_eq!(ref_path, "#/components/schemas/Pet");

        // Test sample generation from Pet schema
        let importer = OpenAPIImporter {
            spec: spec.clone(),
            base_url: "http://test".to_string(),
        };
        let sample = importer
            .generate_sample_from_object_schema(pet)
            .expect("Failed to generate sample");

        // Parse the sample as JSON to verify structure
        let sample_value: serde_json::Value =
            serde_json::from_str(&sample).expect("Failed to parse sample as JSON");
        let sample_obj = sample_value
            .as_object()
            .expect("Sample should be an object");

        // Assert all expected properties are present
        assert!(sample_obj.contains_key("id"));
        assert!(sample_obj.contains_key("name"));
        assert!(sample_obj.contains_key("tag"));
        assert!(sample_obj.contains_key("createdAt"));

        // Assert format-specific values are correct
        assert_eq!(sample_obj["id"], 0);
        assert_eq!(sample_obj["name"], "string");
        assert_eq!(sample_obj["tag"], "string");
        assert_eq!(
            sample_obj["createdAt"], "2024-01-01T00:00:00Z",
            "date-time format should generate ISO 8601 timestamp"
        );
    }
}
