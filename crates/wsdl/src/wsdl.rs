use std::collections::HashMap;
use std::path::Path;

use quick_xml::NsReader;
use quick_xml::events::Event;

use crate::types::{QName, SoapStyle, WsdlError};

/// Parsed WSDL 1.1 document.
#[derive(Debug, Default)]
pub struct WsdlDocument {
    pub target_namespace: Option<String>,
    pub services: Vec<WsdlService>,
    pub bindings: Vec<WsdlBinding>,
    pub port_types: Vec<WsdlPortType>,
    pub messages: Vec<WsdlMessage>,
    /// Paths to imported XSD files (resolved relative to WSDL file).
    pub imported_schema_paths: Vec<String>,
    /// Raw `schemaLocation` attribute values from `xs:import` elements.
    pub imported_schema_locations: Vec<String>,
    /// Namespace prefix → URI mapping from the WSDL document.
    pub namespaces: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WsdlService {
    pub name: String,
    pub ports: Vec<WsdlPort>,
}

#[derive(Debug, Clone)]
pub struct WsdlPort {
    pub name: String,
    pub binding: QName,
    pub address_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WsdlBinding {
    pub name: String,
    pub port_type: QName,
    pub soap_style: SoapStyle,
    pub soap_transport: Option<String>,
    pub operations: Vec<WsdlBindingOperation>,
}

#[derive(Debug, Clone)]
pub struct WsdlBindingOperation {
    pub name: String,
    pub soap_action: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WsdlPortType {
    pub name: String,
    pub operations: Vec<WsdlPortTypeOperation>,
}

#[derive(Debug, Clone)]
pub struct WsdlPortTypeOperation {
    pub name: String,
    pub input_message: QName,
    pub output_message: Option<QName>,
}

#[derive(Debug, Clone)]
pub struct WsdlMessage {
    pub name: String,
    pub parts: Vec<WsdlPart>,
}

#[derive(Debug, Clone)]
pub struct WsdlPart {
    pub name: String,
    pub element: Option<QName>,
    pub type_: Option<QName>,
}

/// Parse a WSDL 1.1 document from a file path.
pub fn parse_wsdl_file(path: &Path) -> Result<WsdlDocument, WsdlError> {
    let content = std::fs::read_to_string(path)?;
    let base_dir = path.parent().unwrap_or(Path::new("."));
    parse_wsdl_str(&content, Some(base_dir))
}

/// Parse a WSDL 1.1 document from a string.
pub fn parse_wsdl_str(content: &str, base_dir: Option<&Path>) -> Result<WsdlDocument, WsdlError> {
    let mut reader = NsReader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut doc = WsdlDocument::default();
    let mut buf = Vec::new();

    let mut depth = 0usize;
    let mut in_types = false;

    // Track which binding we're inside to correlate operations
    let mut current_binding_idx: Option<usize> = None;
    let mut current_binding_op_name: Option<String> = None;
    // Track soap:operation soapAction within a binding/wsdl:operation
    let mut pending_soap_action: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                depth += 1;
                let local_name = e.local_name();
                let local_str = local_name.as_ref();

                capture_namespaces(&e, &mut doc.namespaces);

                match local_str {
                    b"definitions" => {
                        doc.target_namespace = get_attr(&e, "targetNamespace");
                    }
                    b"types" => {
                        in_types = true;
                    }
                    b"service" => {
                        let name = get_attr(&e, "name").unwrap_or_default();
                        doc.services.push(WsdlService {
                            name,
                            ports: Vec::new(),
                        });
                    }
                    b"port" => {
                        let name = get_attr(&e, "name").unwrap_or_default();
                        let binding = resolve_qname(&e, "binding", &doc.namespaces);
                        if let Some(service) = doc.services.last_mut() {
                            service.ports.push(WsdlPort {
                                name,
                                binding,
                                address_url: None,
                            });
                        }
                    }
                    b"binding" => {
                        let name = get_attr(&e, "name").unwrap_or_default();
                        let port_type = resolve_qname(&e, "type", &doc.namespaces);
                        doc.bindings.push(WsdlBinding {
                            name,
                            port_type,
                            soap_style: SoapStyle::Document,
                            soap_transport: None,
                            operations: Vec::new(),
                        });
                        current_binding_idx = Some(doc.bindings.len() - 1);
                    }
                    b"portType" => {
                        let name = get_attr(&e, "name").unwrap_or_default();
                        doc.port_types.push(WsdlPortType {
                            name,
                            operations: Vec::new(),
                        });
                    }
                    b"message" => {
                        let name = get_attr(&e, "name").unwrap_or_default();
                        doc.messages.push(WsdlMessage {
                            name,
                            parts: Vec::new(),
                        });
                    }
                    b"operation" if current_binding_idx.is_some() => {
                        // This is a wsdl:operation inside a binding
                        current_binding_op_name = get_attr(&e, "name");
                        pending_soap_action = None;
                    }
                    b"operation" => {
                        // This is a wsdl:operation inside a portType
                        let name = get_attr(&e, "name").unwrap_or_default();
                        if let Some(pt) = doc.port_types.last_mut() {
                            pt.operations.push(WsdlPortTypeOperation {
                                name,
                                input_message: QName::new(""),
                                output_message: None,
                            });
                        }
                    }
                    _ => {}
                }

                // Handle SOAP namespace elements
                handle_soap_element(&e, &mut doc, current_binding_idx, &mut pending_soap_action);
            }
            Event::Empty(e) => {
                let local_name = e.local_name();
                let local_str = local_name.as_ref();

                if in_types
                    && local_str == b"import"
                    && let Some(loc) = get_attr(&e, "schemaLocation")
                {
                    doc.imported_schema_locations.push(loc.clone());
                    if let Some(dir) = base_dir {
                        let full_path = dir.join(&loc);
                        doc.imported_schema_paths
                            .push(full_path.to_string_lossy().to_string());
                    } else {
                        doc.imported_schema_paths.push(loc);
                    }
                }

                // Handle message parts (may be self-closing)
                match local_str {
                    b"part" => {
                        let name = get_attr(&e, "name").unwrap_or_default();
                        let element = resolve_qname_opt(&e, "element", &doc.namespaces);
                        let type_ = resolve_qname_opt(&e, "type", &doc.namespaces);
                        if let Some(msg) = doc.messages.last_mut() {
                            msg.parts.push(WsdlPart {
                                name,
                                element,
                                type_,
                            });
                        }
                    }
                    b"input" => {
                        let msg = resolve_qname(&e, "message", &doc.namespaces);
                        if let Some(pt) = doc.port_types.last_mut()
                            && let Some(op) = pt.operations.last_mut()
                        {
                            op.input_message = msg;
                        }
                    }
                    b"output" => {
                        let msg = resolve_qname_opt(&e, "message", &doc.namespaces);
                        if let Some(pt) = doc.port_types.last_mut()
                            && let Some(op) = pt.operations.last_mut()
                        {
                            op.output_message = msg;
                        }
                    }
                    _ => {}
                }

                // Handle SOAP namespace elements (self-closing)
                handle_soap_element(&e, &mut doc, current_binding_idx, &mut pending_soap_action);
            }
            Event::End(e) => {
                match e.local_name().as_ref() {
                    b"types" => {
                        in_types = false;
                    }
                    b"binding" => {
                        current_binding_idx = None;
                    }
                    b"operation" if current_binding_idx.is_some() => {
                        // End of binding operation - save the collected soap action
                        if let (Some(bi), Some(op_name)) =
                            (current_binding_idx, current_binding_op_name.take())
                            && let Some(binding) = doc.bindings.get_mut(bi)
                        {
                            binding.operations.push(WsdlBindingOperation {
                                name: op_name,
                                soap_action: pending_soap_action.take(),
                            });
                        }
                    }
                    _ => {}
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(doc)
}

fn handle_soap_element(
    e: &quick_xml::events::BytesStart<'_>,
    doc: &mut WsdlDocument,
    current_binding_idx: Option<usize>,
    pending_soap_action: &mut Option<String>,
) {
    // Check if this element is in a SOAP namespace by looking up its prefix
    let prefix = e
        .name()
        .prefix()
        .map(|p| String::from_utf8_lossy(p.as_ref()).to_string());

    let Some(prefix_str) = &prefix else { return };
    let Some(ns_uri) = doc.namespaces.get(prefix_str) else {
        return;
    };

    let is_soap_ns = ns_uri.contains("wsdl/soap");

    if !is_soap_ns {
        return;
    }

    match e.local_name().as_ref() {
        b"binding" => {
            if let Some(bi) = current_binding_idx
                && let Some(binding) = doc.bindings.get_mut(bi)
            {
                if let Some(style) = get_attr(e, "style") {
                    binding.soap_style = SoapStyle::parse(&style);
                }
                binding.soap_transport = get_attr(e, "transport");
            }
        }
        b"operation" => {
            *pending_soap_action = get_attr(e, "soapAction");
        }
        b"address" => {
            if let Some(url) = get_attr(e, "location")
                && let Some(service) = doc.services.last_mut()
                && let Some(port) = service.ports.last_mut()
            {
                port.address_url = Some(url);
            }
        }
        _ => {}
    }
}

fn capture_namespaces(
    e: &quick_xml::events::BytesStart<'_>,
    namespaces: &mut HashMap<String, String>,
) {
    for attr in e.attributes().flatten() {
        if let Some(prefix) = attr.key.prefix() {
            if prefix.as_ref() == b"xmlns" {
                let ns_prefix = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                let ns_uri = String::from_utf8_lossy(&attr.value).to_string();
                namespaces.insert(ns_prefix, ns_uri);
            }
        } else if attr.key.local_name().as_ref() == b"xmlns" {
            let ns_uri = String::from_utf8_lossy(&attr.value).to_string();
            namespaces.insert("_default".to_string(), ns_uri);
        }
    }
}

fn get_attr(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == name.as_bytes() {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

fn resolve_qname(
    e: &quick_xml::events::BytesStart<'_>,
    attr_name: &str,
    namespaces: &HashMap<String, String>,
) -> QName {
    resolve_qname_opt(e, attr_name, namespaces).unwrap_or_else(|| QName::new(""))
}

fn resolve_qname_opt(
    e: &quick_xml::events::BytesStart<'_>,
    attr_name: &str,
    namespaces: &HashMap<String, String>,
) -> Option<QName> {
    let value = get_attr(e, attr_name)?;
    parse_qname_str(&value, namespaces)
}

fn parse_qname_str(value: &str, namespaces: &HashMap<String, String>) -> Option<QName> {
    if let Some((prefix, local)) = value.split_once(':') {
        let ns_uri = namespaces.get(prefix).cloned();
        Some(QName {
            namespace: ns_uri,
            local_name: local.to_string(),
        })
    } else {
        let ns_uri = namespaces.get("_default").cloned();
        Some(QName {
            namespace: ns_uri,
            local_name: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_wsdl() {
        let wsdl = r#"<?xml version="1.0" encoding="UTF-8"?>
<wsdl:definitions
    xmlns:wsdl="http://schemas.xmlsoap.org/wsdl/"
    xmlns:soap="http://schemas.xmlsoap.org/wsdl/soap/"
    xmlns:tns="http://example.com/test"
    targetNamespace="http://example.com/test">

    <wsdl:message name="testRequest">
        <wsdl:part name="parameters" element="tns:testElement"/>
    </wsdl:message>

    <wsdl:portType name="testPortType">
        <wsdl:operation name="testOp">
            <wsdl:input message="tns:testRequest"/>
        </wsdl:operation>
    </wsdl:portType>

    <wsdl:binding name="testBinding" type="tns:testPortType">
        <soap:binding style="document" transport="http://schemas.xmlsoap.org/soap/http"/>
        <wsdl:operation name="testOp">
            <soap:operation soapAction="http://example.com/test/testOp"/>
        </wsdl:operation>
    </wsdl:binding>

    <wsdl:service name="testService">
        <wsdl:port name="testPort" binding="tns:testBinding">
            <soap:address location="http://example.com/endpoint"/>
        </wsdl:port>
    </wsdl:service>
</wsdl:definitions>"#;

        let doc = parse_wsdl_str(wsdl, None).unwrap();

        assert_eq!(
            doc.target_namespace.as_deref(),
            Some("http://example.com/test")
        );
        assert_eq!(doc.services.len(), 1);
        assert_eq!(doc.services[0].name, "testService");
        assert_eq!(doc.services[0].ports.len(), 1);
        assert_eq!(
            doc.services[0].ports[0].address_url.as_deref(),
            Some("http://example.com/endpoint")
        );
        assert_eq!(doc.port_types.len(), 1);
        assert_eq!(doc.port_types[0].operations.len(), 1);
        assert_eq!(doc.port_types[0].operations[0].name, "testOp");
        assert_eq!(doc.messages.len(), 1);
        assert_eq!(doc.messages[0].parts.len(), 1);
        assert_eq!(
            doc.messages[0].parts[0]
                .element
                .as_ref()
                .unwrap()
                .local_name,
            "testElement"
        );
        assert_eq!(doc.bindings.len(), 1);
        assert_eq!(doc.bindings[0].operations.len(), 1);
        assert_eq!(doc.bindings[0].operations[0].name, "testOp");
        assert_eq!(
            doc.bindings[0].operations[0].soap_action.as_deref(),
            Some("http://example.com/test/testOp")
        );
    }
}
