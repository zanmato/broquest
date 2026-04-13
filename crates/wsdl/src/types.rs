use std::collections::HashSet;
use std::fmt;

use thiserror::Error;

/// Qualified name with namespace and local name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QName {
    pub namespace: Option<String>,
    pub local_name: String,
}

impl QName {
    pub fn new(local_name: impl Into<String>) -> Self {
        Self {
            namespace: None,
            local_name: local_name.into(),
        }
    }

    pub fn with_namespace(namespace: impl Into<String>, local_name: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            local_name: local_name.into(),
        }
    }
}

impl fmt::Display for QName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.namespace {
            Some(ns) => write!(f, "{{{}}}:{}", ns, self.local_name),
            None => write!(f, "{}", self.local_name),
        }
    }
}

/// SOAP style (document or rpc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoapStyle {
    Document,
    Rpc,
}

impl SoapStyle {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rpc" => SoapStyle::Rpc,
            _ => SoapStyle::Document,
        }
    }
}

/// A parsed WSDL operation with all info needed to generate a SOAP request.
#[derive(Debug, Clone)]
pub struct WsdlOperationInfo {
    /// Operation name.
    pub name: String,
    /// SOAPAction header value.
    pub soap_action: Option<String>,
    /// Endpoint URL for this operation.
    pub endpoint_url: String,
    /// Pre-generated SOAP envelope XML.
    pub soap_envelope: String,
    /// Port type name for grouping.
    pub group: Option<String>,
}

/// Output of a WSDL import.
#[derive(Debug, Clone)]
pub struct WsdlImportOutput {
    /// The service endpoint URL.
    pub endpoint_url: String,
    /// All operations found in the WSDL.
    pub operations: Vec<WsdlOperationInfo>,
}

/// Errors that can occur during WSDL processing.
#[derive(Debug, Error)]
pub enum WsdlError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),

    #[error("XSD parsing error: {0}")]
    XsdParser(#[from] xsd_parser::InterpreterError),

    #[error("Invalid WSDL: {0}")]
    InvalidWsdl(String),

    #[error("No service endpoint found in WSDL")]
    NoEndpoint,

    #[error("No operations found in WSDL")]
    NoOperations,

    #[error("Type not found: {0}")]
    TypeNotFound(String),

    #[error("Circular type reference detected: {0}")]
    CircularReference(String),
}

/// Generate a sample value for an XSD built-in type by its name.
/// Maps xs:* type names to placeholder values.
pub fn sample_value_for_builtin(type_name: &str) -> &'static str {
    match type_name {
        "string" | "anySimpleType" | "anyType" | "normalizedString" | "token" | "Name"
        | "NCName" | "language" | "NMTOKEN" | "ID" | "IDREF" | "ENTITY" => "string",
        "int" | "integer" | "long" | "short" | "byte" | "unsignedInt" | "unsignedLong"
        | "unsignedShort" | "unsignedByte" | "nonNegativeInteger" | "nonPositiveInteger"
        | "positiveInteger" | "negativeInteger" => "0",
        "decimal" | "float" | "double" => "0.0",
        "boolean" => "false",
        "date" => "2024-01-01",
        "dateTime" => "2024-01-01T00:00:00Z",
        "time" => "00:00:00Z",
        "anyURI" => "https://example.com",
        "base64Binary" => "SGVsbG8=",
        "hexBinary" => "00",
        "QName" | "NOTATION" => "ns:name",
        "gYear" => "2024",
        "gYearMonth" => "2024-01",
        "gMonth" => "01",
        "gMonthDay" => "01-01",
        "gDay" => "01",
        "duration" => "P1D",
        // xsd-parser maps these to Rust types, not XSD names
        "String" => "string",
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => "0",
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => "0",
        "f32" | "f64" => "0.0",
        "bool" => "false",
        "str" => "string",
        _ => "string",
    }
}

/// Tracks visited types during envelope generation to detect cycles.
#[derive(Debug, Default)]
pub struct VisitTracker {
    visited: HashSet<String>,
}

impl VisitTracker {
    pub fn enter(&mut self, key: &str) -> bool {
        self.visited.insert(key.to_string())
    }

    pub fn exit(&mut self, key: &str) {
        self.visited.remove(key);
    }

    pub fn is_visited(&self, key: &str) -> bool {
        self.visited.contains(key)
    }
}
