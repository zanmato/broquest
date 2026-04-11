use crate::types::{WsdlError, WsdlImportOutput, WsdlOperationInfo};
use crate::wsdl::WsdlDocument;
use crate::xsd;

/// SOAP 1.1 envelope namespace.
const SOAP_ENV_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";

/// Generate SOAP import output from a parsed WSDL document and XSD introspector.
pub fn generate_soap_import(
    wsdl: &WsdlDocument,
    introspector: &xsd::XsdIntrospector,
) -> Result<WsdlImportOutput, WsdlError> {
    let endpoint_url = wsdl
        .services
        .first()
        .and_then(|s| s.ports.first())
        .and_then(|p| p.address_url.clone())
        .ok_or(WsdlError::NoEndpoint)?;

    let mut operations = Vec::new();

    for binding in &wsdl.bindings {
        let port_type = wsdl
            .port_types
            .iter()
            .find(|pt| {
                pt.name == binding.port_type.local_name
                    && binding.port_type.namespace.as_deref() == wsdl.target_namespace.as_deref()
            })
            .or_else(|| {
                wsdl.port_types
                    .iter()
                    .find(|pt| pt.name == binding.port_type.local_name)
            });

        let Some(port_type) = port_type else {
            continue;
        };

        for pt_op in &port_type.operations {
            // Find the matching binding operation for soap action
            let soap_action = binding
                .operations
                .iter()
                .find(|bop| bop.name == pt_op.name)
                .and_then(|op| op.soap_action.clone());

            let envelope = generate_envelope(pt_op, wsdl, introspector)?;

            operations.push(WsdlOperationInfo {
                name: pt_op.name.clone(),
                soap_action,
                endpoint_url: endpoint_url.clone(),
                soap_envelope: envelope,
                group: Some(port_type.name.clone()),
            });
        }
    }

    if operations.is_empty() {
        return Err(WsdlError::NoOperations);
    }

    Ok(WsdlImportOutput {
        endpoint_url,
        operations,
    })
}

fn generate_envelope(
    operation: &crate::wsdl::WsdlPortTypeOperation,
    wsdl: &WsdlDocument,
    introspector: &xsd::XsdIntrospector,
) -> Result<String, WsdlError> {
    let target_ns = wsdl.target_namespace.as_deref().unwrap_or("");
    let tns_prefix = introspector.get_prefix(target_ns).unwrap_or("tns");

    // Find input message
    let input_msg = wsdl
        .messages
        .iter()
        .find(|m| m.name == operation.input_message.local_name);

    let body_content = if let Some(msg) = input_msg {
        if let Some(part) = msg.parts.first() {
            if let Some(element_qname) = &part.element {
                let prefix = introspector
                    .get_prefix(element_qname.namespace.as_deref().unwrap_or(target_ns))
                    .unwrap_or(tns_prefix);

                xsd::generate_sample_xml(
                    introspector,
                    &element_qname.local_name,
                    element_qname.namespace.as_deref(),
                    Some(prefix),
                    2,
                )
            } else {
                format!(
                    "    <{tns_prefix}:{}>string</{tns_prefix}:{}>\n",
                    part.name, part.name
                )
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Build SOAP envelope
    let mut envelope = String::new();
    envelope.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    envelope.push_str(&format!(
        "<soapenv:Envelope xmlns:soapenv=\"{SOAP_ENV_NS}\"\n"
    ));
    envelope.push_str(&format!(
        "                  xmlns:{tns_prefix}=\"{target_ns}\">\n"
    ));
    envelope.push_str("  <soapenv:Body>\n");
    envelope.push_str(&body_content);
    envelope.push_str("  </soapenv:Body>\n");
    envelope.push_str("</soapenv:Envelope>");

    Ok(envelope)
}
