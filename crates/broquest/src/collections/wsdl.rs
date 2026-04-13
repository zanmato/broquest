use super::types::{EnvironmentToml, EnvironmentVariable, ImportResult};
use crate::domain::{AuthType, HttpMethod, KeyValuePair, RequestData};

/// Import from a local WSDL file and produce an ImportResult compatible with the collection system.
#[allow(dead_code)]
pub fn import_from_wsdl(path: &str) -> Result<ImportResult, Box<dyn std::error::Error>> {
    let importer = wsdl::WsdlImporter::from_path(path)?;
    let output = importer.into_output();
    Ok(convert_import_output(output))
}

/// Import from a WSDL URL by downloading it and all referenced XSD schemas.
pub async fn import_from_wsdl_url(
    url: &str,
) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Download the WSDL
    let wsdl_content = async_compat::Compat::new(client.get(url).send())
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Parse the WSDL to find schema imports
    let wsdl_doc = wsdl::wsdl::parse_wsdl_str(&wsdl_content, None)?;

    let base_url = url::Url::parse(url)?;
    let mut inline_schemas = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Recursively download all referenced XSD schemas
    download_schema_refs(
        &client,
        &base_url,
        &wsdl_doc.imported_schema_locations,
        &mut inline_schemas,
        &mut visited,
    )
    .await?;

    let importer = wsdl::WsdlImporter::from_content(&wsdl_content, inline_schemas)?;
    let output = importer.into_output();
    Ok(convert_import_output(output))
}

/// Recursively download XSD schemas referenced by `schemaLocation` attributes.
/// Returns `(filename, content)` pairs where filename is the last path segment of the URL.
async fn download_schema_refs(
    client: &reqwest::Client,
    base_url: &url::Url,
    locations: &[String],
    inline_schemas: &mut Vec<(String, String)>,
    visited: &mut std::collections::HashSet<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for loc in locations {
        let resolved = base_url.join(loc)?;
        let resolved_str = resolved.to_string();

        if visited.contains(&resolved_str) {
            continue;
        }
        visited.insert(resolved_str.clone());

        let content = async_compat::Compat::new(client.get(resolved.clone()).send())
            .await?
            .error_for_status()?
            .text()
            .await?;

        // Extract any nested xs:import schemaLocation from this XSD
        let nested_locs = extract_schema_imports(&content);
        if !nested_locs.is_empty() {
            let nested_base = resolved.clone();
            Box::pin(download_schema_refs(
                client,
                &nested_base,
                &nested_locs,
                inline_schemas,
                visited,
            ))
            .await?;
        }

        // Use the original loc as the name so xs:import schemaLocation matches work
        let filename = loc.clone();
        inline_schemas.push((filename, content));
    }
    Ok(())
}

/// Extract raw `schemaLocation` attribute values from `xs:import` elements in an XSD string.
fn extract_schema_imports(xsd_content: &str) -> Vec<String> {
    let mut reader = quick_xml::NsReader::from_str(xsd_content);
    reader.config_mut().trim_text(true);
    let mut locations = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Empty(e)) => {
                if e.local_name().as_ref() == b"import" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"schemaLocation" {
                            locations.push(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    locations
}

/// Convert WsdlImportOutput into the collection-compatible ImportResult.
fn convert_import_output(output: wsdl::WsdlImportOutput) -> ImportResult {
    let mut groups: Vec<(String, Vec<RequestData>)> = Vec::new();
    let mut requests: Vec<RequestData> = Vec::new();

    // Group by port type name
    let mut grouped: std::collections::BTreeMap<Option<String>, Vec<wsdl::WsdlOperationInfo>> =
        std::collections::BTreeMap::new();
    for op in output.operations {
        grouped.entry(op.group.clone()).or_default().push(op);
    }

    for (group_name, operations) in grouped {
        let group_requests: Vec<RequestData> = operations.iter().map(create_soap_request).collect();

        match group_name {
            Some(name) => groups.push((name, group_requests)),
            None => requests.extend(group_requests),
        }
    }

    let mut variables = std::collections::HashMap::new();
    variables.insert(
        "baseUrl".to_string(),
        EnvironmentVariable {
            value: output.endpoint_url,
            secret: false,
            temporary: false,
        },
    );

    ImportResult {
        environment: EnvironmentToml {
            name: "Default".to_string(),
            variables,
        },
        groups,
        requests,
    }
}

fn create_soap_request(op: &wsdl::WsdlOperationInfo) -> RequestData {
    let headers = vec![
        KeyValuePair {
            key: "Content-Type".into(),
            value: "text/xml; charset=utf-8".into(),
            enabled: true,
        },
        KeyValuePair {
            key: "SOAPAction".into(),
            value: op.soap_action.clone().unwrap_or_default(),
            enabled: true,
        },
    ];

    RequestData {
        name: op.name.clone(),
        method: HttpMethod::Post,
        url: "{{baseUrl}}".to_string(),
        path_params: vec![],
        query_params: vec![],
        headers,
        body: op.soap_envelope.clone(),
        auth: AuthType::None,
        pre_request_script: None,
        post_response_script: None,
    }
}
