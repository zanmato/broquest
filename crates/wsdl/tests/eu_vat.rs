use std::path::Path;

use wsdl::wsdl as wsdl_parser;

#[test]
fn test_parse_eu_vat_wsdl() {
    let path = Path::new("tests/fixtures/VatRetrievalService.wsdl");
    let doc = wsdl_parser::parse_wsdl_file(path).expect("Failed to parse WSDL");

    // Basic structure
    assert_eq!(
        doc.target_namespace.as_deref(),
        Some("urn:ec.europa.eu:taxud:tedb:services:v1:VatRetrievalService")
    );

    // Services
    assert_eq!(doc.services.len(), 1);
    assert_eq!(doc.services[0].name, "vatRetrievalServiceService");
    assert_eq!(doc.services[0].ports.len(), 1);
    assert_eq!(
        doc.services[0].ports[0].address_url.as_deref(),
        Some("http://ec.europa.eu/taxation_customs/tedb/ws/")
    );

    // Port types
    assert_eq!(doc.port_types.len(), 1);
    assert_eq!(doc.port_types[0].name, "vatRetrievalService");
    assert_eq!(doc.port_types[0].operations.len(), 1);
    assert_eq!(doc.port_types[0].operations[0].name, "retrieveVatRates");

    // Binding
    assert_eq!(doc.bindings.len(), 1);
    assert_eq!(doc.bindings[0].operations.len(), 1);
    assert_eq!(doc.bindings[0].operations[0].name, "retrieveVatRates");
    assert_eq!(
        doc.bindings[0].operations[0].soap_action.as_deref(),
        Some("urn:ec.europa.eu:taxud:tedb:services:v1:VatRetrievalService/RetrieveVatRates")
    );

    // Messages
    assert_eq!(doc.messages.len(), 3);
    let req_msg = doc
        .messages
        .iter()
        .find(|m| m.name == "retrieveVatRatesReqMsg")
        .unwrap();
    assert_eq!(req_msg.parts.len(), 1);
    assert_eq!(
        req_msg.parts[0].element.as_ref().unwrap().local_name,
        "retrieveVatRatesReqMsg"
    );

    // Imported schema paths
    assert!(!doc.imported_schema_paths.is_empty());
}

#[test]
fn test_full_import_eu_vat() {
    let path = "tests/fixtures/VatRetrievalService.wsdl";
    let importer = wsdl::WsdlImporter::from_path(path).expect("WSDL import should succeed");

    let output = importer.import();
    assert!(output.endpoint_url.contains("ec.europa.eu"));
    assert!(!output.operations.is_empty());

    let op = output.operations.first().unwrap();
    assert_eq!(op.name, "retrieveVatRates");
    assert!(op.soap_action.is_some());

    // Print the envelope for inspection
    println!("=== SOAP Envelope for {} ===", op.name);
    println!("{}", op.soap_envelope);

    // Verify the envelope is valid XML
    let mut reader = quick_xml::Reader::from_str(&op.soap_envelope);
    let mut buf = Vec::new();
    let mut depth = 0;
    loop {
        match reader.read_event_into(&mut buf).unwrap() {
            quick_xml::events::Event::Start(_) => depth += 1,
            quick_xml::events::Event::End(_) => depth -= 1,
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    assert_eq!(depth, 0, "SOAP envelope should be well-formed XML");

    // Verify SOAP envelope structure
    assert!(op.soap_envelope.contains("soapenv:Envelope"));
    assert!(op.soap_envelope.contains("soapenv:Body"));
}
