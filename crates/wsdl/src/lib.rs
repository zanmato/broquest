pub mod soap;
pub mod types;
pub mod wsdl;
pub mod xsd;

pub use types::{WsdlError, WsdlImportOutput, WsdlOperationInfo};

use std::path::Path;

/// WSDL importer that parses a WSDL file and generates SOAP request templates.
pub struct WsdlImporter {
    output: WsdlImportOutput,
}

impl WsdlImporter {
    /// Parse a WSDL file and prepare for import.
    pub fn from_path(path: &str) -> Result<Self, WsdlError> {
        let wsdl_path = Path::new(path);
        let wsdl_doc = wsdl::parse_wsdl_file(wsdl_path)?;

        let schema_files = wsdl_doc.imported_schema_paths.clone();
        let inline_schemas: Vec<(String, String)> = Vec::new();

        let xsd_introspector = xsd::XsdIntrospector::new(&schema_files, &inline_schemas)?;
        let output = soap::generate_soap_import(&wsdl_doc, &xsd_introspector)?;

        Ok(Self { output })
    }

    /// Parse WSDL content from a string with named inline XSD schemas (e.g. downloaded from a URL).
    ///
    /// `inline_schemas` is a list of `(filename, content)` pairs where the filename
    /// should match the `schemaLocation` value used in `<xs:import>` elements.
    pub fn from_content(
        wsdl_content: &str,
        inline_schemas: Vec<(String, String)>,
    ) -> Result<Self, WsdlError> {
        let wsdl_doc = wsdl::parse_wsdl_str(wsdl_content, None)?;

        let xsd_introspector = xsd::XsdIntrospector::new(&[], &inline_schemas)?;
        let output = soap::generate_soap_import(&wsdl_doc, &xsd_introspector)?;

        Ok(Self { output })
    }

    /// Get the import output.
    pub fn import(&self) -> &WsdlImportOutput {
        &self.output
    }

    /// Consume the importer and get the output.
    pub fn into_output(self) -> WsdlImportOutput {
        self.output
    }
}
