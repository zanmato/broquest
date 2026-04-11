use std::collections::HashMap;
use std::fmt;
use std::io::{BufReader, Cursor};

use xsd_parser::pipeline::parser::resolver::{
    FileResolver, ManyResolver, ResolveRequest, ResolveResult, Resolver,
};
use xsd_parser::{
    Interpreter, MetaTypes, Parser,
    models::meta::{
        BuildInMeta, ComplexMeta, ElementMeta, ElementMetaVariant, EnumerationMeta, GroupMeta,
        MetaType, MetaTypeVariant, SimpleMeta,
    },
};

use crate::types::WsdlError;

/// Resolver that provides schema content from an in-memory map keyed by filename.
#[derive(Debug)]
struct InMemoryResolver {
    schemas: HashMap<String, String>,
}

impl InMemoryResolver {
    fn new(schemas: HashMap<String, String>) -> Self {
        Self { schemas }
    }
}

/// Helper to extract the filename from a URL path.
fn url_filename(url: &url::Url) -> Option<&str> {
    url.path().rsplit('/').next()
}

impl Resolver for InMemoryResolver {
    type Buffer = BufReader<Cursor<String>>;
    type Error = InMemoryResolverError;

    fn resolve(&mut self, req: &ResolveRequest) -> ResolveResult<Self> {
        // Try matching by requested_location directly (e.g. "VatRetrievalServiceType.xsd")
        if let Some(content) = self.schemas.get(&req.requested_location) {
            let url = url::Url::parse(&req.requested_location)
                .unwrap_or_else(|_| url::Url::parse("http://inline/").unwrap());
            let name = req
                .requested_location
                .rsplit(['/', '\\'])
                .next()
                .and_then(|s: &str| s.strip_suffix(".xsd").map(|s| s.to_string()));
            let buffer = BufReader::new(Cursor::new(content.clone()));
            return Ok(Some((name, url, buffer)));
        }

        // Try resolving relative path against current_location, then match by filename
        if let Some(current) = &req.current_location
            && let Ok(resolved) = current.join(&req.requested_location)
        {
            let resolved_str = resolved.to_string();
            if let Some(content) = self.schemas.get(&resolved_str) {
                let name = url_filename(&resolved)
                    .and_then(|s| s.strip_suffix(".xsd").map(|s| s.to_string()));
                let buffer = BufReader::new(Cursor::new(content.clone()));
                return Ok(Some((name, resolved, buffer)));
            }
            // Also try matching just the filename part
            if let Some(filename) = url_filename(&resolved)
                && let Some(content) = self.schemas.get(filename)
            {
                let name = filename.strip_suffix(".xsd").map(|s: &str| s.to_string());
                let buffer = BufReader::new(Cursor::new(content.clone()));
                return Ok(Some((name, resolved, buffer)));
            }
        }

        Ok(None)
    }
}

#[derive(Debug)]
struct InMemoryResolverError;

impl fmt::Display for InMemoryResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "in-memory resolver error")
    }
}

impl std::error::Error for InMemoryResolverError {}

/// Wrapper around xsd-parser for XSD type introspection.
pub struct XsdIntrospector {
    meta_types: MetaTypes,
    /// Maps namespace id (usize) → namespace URI
    ns_id_to_uri: HashMap<usize, String>,
    /// Maps (namespace_uri, local_name) → element/type info for lookup
    element_index: HashMap<(Option<String>, String), String>,
    /// Maps namespace URI → preferred prefix
    ns_prefixes: HashMap<String, String>,
}

impl XsdIntrospector {
    /// Parse XSD schemas from the given file paths and named inline schemas.
    ///
    /// `inline_schemas` is a list of `(name, content)` pairs where `name` is
    /// the original filename (e.g. `"VatRetrievalServiceType.xsd"`) used for
    /// import resolution between schemas.
    pub fn new(
        schema_files: &[String],
        inline_schemas: &[(String, String)],
    ) -> Result<Self, WsdlError> {
        let resolver = if inline_schemas.is_empty() {
            ManyResolver::new().add_resolver(FileResolver::new())
        } else {
            let mut map = HashMap::new();
            for (name, content) in inline_schemas {
                map.insert(name.clone(), content.clone());
            }
            ManyResolver::new()
                .add_resolver(InMemoryResolver::new(map))
                .add_resolver(FileResolver::new())
        };

        let mut parser = Parser::new()
            .with_resolver(resolver)
            .with_default_namespaces();

        for path in schema_files {
            let p = std::path::Path::new(path);
            if p.exists() {
                parser = parser.add_schema_from_file(p).map_err(|e| {
                    WsdlError::InvalidWsdl(format!("Failed to parse XSD file {}: {}", path, e))
                })?;
            }
        }

        for (name, content) in inline_schemas {
            parser = parser
                .add_named_schema_from_str(name.clone(), content)
                .map_err(|e| {
                    WsdlError::InvalidWsdl(format!("Failed to parse inline schema {}: {}", name, e))
                })?;
        }

        let schemas = parser.finish();

        let (meta_types, _ident_cache) = Interpreter::new(&schemas)
            .with_buildin_types()
            .map_err(|e| WsdlError::InvalidWsdl(format!("Interpreter buildin types error: {}", e)))?
            .with_default_typedefs()
            .map_err(|e| WsdlError::InvalidWsdl(format!("Interpreter typedefs error: {}", e)))?
            .finish()
            .map_err(WsdlError::XsdParser)?;

        let mut introspector = Self {
            meta_types,
            element_index: HashMap::new(),
            ns_id_to_uri: HashMap::new(),
            ns_prefixes: HashMap::new(),
        };

        introspector.build_indices();
        Ok(introspector)
    }

    fn build_indices(&mut self) {
        // Build namespace ID → URI map
        for (ns_id, module) in &self.meta_types.modules {
            if let Some(ns) = &module.namespace {
                self.ns_id_to_uri.insert(ns_id.0, ns.to_string());
            }
            if let Some(ns) = &module.namespace
                && let Some(prefix) = &module.prefix
            {
                self.ns_prefixes.insert(ns.to_string(), prefix.to_string());
            }
        }

        // Index types by (namespace_uri, name) → type_ident name string
        for type_ident in self.meta_types.items.keys() {
            let name = type_ident.name.as_str().to_string();
            let ns_uri = self.ns_id_to_uri.get(&type_ident.ns.0).cloned();
            self.element_index.insert((ns_uri, name.clone()), name);
        }
    }

    /// Find a type ident string by QName.
    pub fn find_type_ident(&self, qname: &crate::types::QName) -> Option<String> {
        self.element_index
            .get(&(qname.namespace.clone(), qname.local_name.clone()))
            .cloned()
    }

    /// Look up a MetaType by its TypeIdent string representation.
    pub fn get_type_by_name(&self, name: &str) -> Option<&MetaType> {
        for (ident, meta_type) in &self.meta_types.items {
            if ident.name.as_str() == name {
                return Some(meta_type);
            }
        }
        None
    }

    /// Get resolved type by name (follows references).
    pub fn get_resolved_by_name(&self, name: &str) -> Option<&MetaType> {
        for ident in self.meta_types.items.keys() {
            if ident.name.as_str() == name {
                return self.meta_types.get_resolved_type(ident);
            }
        }
        None
    }

    /// Get the prefix for a namespace URI.
    pub fn get_prefix(&self, namespace: &str) -> Option<&str> {
        self.ns_prefixes.get(namespace).map(|s| s.as_str())
    }

    /// Reference to the underlying MetaTypes.
    pub fn meta_types(&self) -> &MetaTypes {
        &self.meta_types
    }
}

/// Generate sample XML content for a type, returning the XML as a string.
pub fn generate_sample_xml(
    introspector: &XsdIntrospector,
    element_name: &str,
    element_ns: Option<&str>,
    prefix: Option<&str>,
    indent: usize,
) -> String {
    let mut xml = String::new();
    let mut tracker = crate::types::VisitTracker::default();

    // Look up the type for this element
    let qname = crate::types::QName {
        namespace: element_ns.map(String::from),
        local_name: element_name.to_string(),
    };

    if let Some(type_ident_str) = introspector.find_type_ident(&qname) {
        render_type_by_ident(
            introspector,
            &type_ident_str,
            element_name,
            prefix,
            indent,
            &mut xml,
            &mut tracker,
        );
    } else {
        // Fallback: generate a placeholder element
        let ind = "  ".repeat(indent);
        let pref = prefix.map(|p| format!("{p}:")).unwrap_or_default();
        xml.push_str(&format!(
            "{ind}<{pref}{element_name}>string</{pref}{element_name}>\n"
        ));
    }

    xml
}

fn render_type_by_ident(
    introspector: &XsdIntrospector,
    type_ident_str: &str,
    element_name: &str,
    prefix: Option<&str>,
    indent: usize,
    xml: &mut String,
    tracker: &mut crate::types::VisitTracker,
) {
    if tracker.is_visited(type_ident_str) {
        let ind = "  ".repeat(indent);
        xml.push_str(&format!(
            "{ind}<!-- Circular reference to {} -->\n",
            type_ident_str
        ));
        return;
    }
    tracker.enter(type_ident_str);

    let meta_type = introspector.get_resolved_by_name(type_ident_str);

    match meta_type {
        Some(mt) => {
            render_meta_type(
                introspector,
                mt,
                element_name,
                type_ident_str,
                prefix,
                indent,
                xml,
                tracker,
            );
        }
        None => {
            let value = crate::types::sample_value_for_builtin(type_ident_str);
            let ind = "  ".repeat(indent);
            let pref = prefix.map(|p| format!("{p}:")).unwrap_or_default();
            xml.push_str(&format!(
                "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
            ));
        }
    }

    tracker.exit(type_ident_str);
}

#[allow(clippy::too_many_arguments)]
fn render_meta_type(
    introspector: &XsdIntrospector,
    mt: &MetaType,
    element_name: &str,
    type_name_hint: &str,
    prefix: Option<&str>,
    indent: usize,
    xml: &mut String,
    tracker: &mut crate::types::VisitTracker,
) {
    let ind = "  ".repeat(indent);
    let pref = prefix.map(|p| format!("{p}:")).unwrap_or_default();

    match &mt.variant {
        MetaTypeVariant::BuildIn(builtin) => {
            // xsd-parser maps many XSD types to Rust String (e.g. xs:date → String).
            // Use the original type name to produce a better sample value.
            let value = if matches!(builtin, BuildInMeta::String | BuildInMeta::Str) {
                crate::types::sample_value_for_builtin(type_name_hint)
            } else {
                builtin_to_sample(builtin)
            };
            xml.push_str(&format!(
                "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
            ));
        }
        MetaTypeVariant::Reference(reference) => {
            let ref_name = reference.type_.name.as_str().to_string();
            render_type_by_ident(
                introspector,
                &ref_name,
                element_name,
                prefix,
                indent,
                xml,
                tracker,
            );
        }
        MetaTypeVariant::ComplexType(complex) => {
            render_complex_type(
                introspector,
                complex,
                element_name,
                prefix,
                indent,
                xml,
                tracker,
            );
        }
        MetaTypeVariant::Sequence(group) | MetaTypeVariant::All(group) => {
            xml.push_str(&format!("{ind}<{pref}{element_name}>\n"));
            render_group_elements(introspector, group, prefix, indent + 1, xml, tracker);
            xml.push_str(&format!("{ind}</{pref}{element_name}>\n"));
        }
        MetaTypeVariant::Choice(group) => {
            xml.push_str(&format!("{ind}<{pref}{element_name}>\n"));
            let choice_ind = "  ".repeat(indent + 1);
            xml.push_str(&format!(
                "{choice_ind}<!-- Choice: use one of the following -->\n"
            ));
            for (i, el) in group.elements.iter().enumerate() {
                if i > 0 {
                    xml.push_str(&format!("{choice_ind}<!-- OR -->\n"));
                }
                render_element(introspector, el, prefix, indent + 1, xml, tracker);
            }
            xml.push_str(&format!("{ind}</{pref}{element_name}>\n"));
        }
        MetaTypeVariant::Enumeration(enumeration) => {
            let value = first_enum_value(enumeration);
            xml.push_str(&format!(
                "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
            ));
        }
        MetaTypeVariant::SimpleType(simple) => {
            let base_name = simple.base.name.as_str();
            let value: &str = if matches!(base_name, "String" | "str") {
                crate::types::sample_value_for_builtin(type_name_hint)
            } else {
                crate::types::sample_value_for_builtin(base_name)
            };
            xml.push_str(&format!(
                "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
            ));
        }
        MetaTypeVariant::Union(_) | MetaTypeVariant::Custom(_) | MetaTypeVariant::Dynamic(_) => {
            xml.push_str(&format!(
                "{ind}<{pref}{element_name}>string</{pref}{element_name}>\n"
            ));
        }
    }
}

fn render_complex_type(
    introspector: &XsdIntrospector,
    complex: &ComplexMeta,
    element_name: &str,
    prefix: Option<&str>,
    indent: usize,
    xml: &mut String,
    tracker: &mut crate::types::VisitTracker,
) {
    let ind = "  ".repeat(indent);
    let pref = prefix.map(|p| format!("{p}:")).unwrap_or_default();

    if let Some(content_ident) = &complex.content {
        if let Some(content_type) = introspector.meta_types().items.get(content_ident) {
            match &content_type.variant {
                MetaTypeVariant::Sequence(group) | MetaTypeVariant::All(group) => {
                    xml.push_str(&format!("{ind}<{pref}{element_name}>\n"));
                    render_group_elements(introspector, group, prefix, indent + 1, xml, tracker);
                    xml.push_str(&format!("{ind}</{pref}{element_name}>\n"));
                }
                MetaTypeVariant::Choice(group) => {
                    xml.push_str(&format!("{ind}<{pref}{element_name}>\n"));
                    let choice_ind = "  ".repeat(indent + 1);
                    xml.push_str(&format!(
                        "{choice_ind}<!-- Choice: use one of the following -->\n"
                    ));
                    for (i, el) in group.elements.iter().enumerate() {
                        if i > 0 {
                            xml.push_str(&format!("{choice_ind}<!-- OR -->\n"));
                        }
                        render_element(introspector, el, prefix, indent + 1, xml, tracker);
                    }
                    xml.push_str(&format!("{ind}</{pref}{element_name}>\n"));
                }
                MetaTypeVariant::BuildIn(builtin) => {
                    let value = builtin_to_sample(builtin);
                    xml.push_str(&format!(
                        "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
                    ));
                }
                MetaTypeVariant::Enumeration(enumeration) => {
                    let value = first_enum_value(enumeration);
                    xml.push_str(&format!(
                        "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
                    ));
                }
                MetaTypeVariant::SimpleType(simple) => {
                    let value = simple_type_sample(simple);
                    xml.push_str(&format!(
                        "{ind}<{pref}{element_name}>{value}</{pref}{element_name}>\n"
                    ));
                }
                MetaTypeVariant::Reference(reference) => {
                    let ref_name = reference.type_.name.as_str().to_string();
                    render_type_by_ident(
                        introspector,
                        &ref_name,
                        element_name,
                        prefix,
                        indent,
                        xml,
                        tracker,
                    );
                }
                _ => {
                    xml.push_str(&format!("{ind}<{pref}{element_name}>\n"));
                    xml.push_str(&format!("{ind}</{pref}{element_name}>\n"));
                }
            }
        } else {
            xml.push_str(&format!("{ind}<{pref}{element_name}>\n"));
            xml.push_str(&format!("{ind}</{pref}{element_name}>\n"));
        }
    } else {
        xml.push_str(&format!("{ind}<{pref}{element_name}/>\n"));
    }
}

fn render_group_elements(
    introspector: &XsdIntrospector,
    group: &GroupMeta,
    prefix: Option<&str>,
    indent: usize,
    xml: &mut String,
    tracker: &mut crate::types::VisitTracker,
) {
    for el in group.elements.iter() {
        render_element(introspector, el, prefix, indent, xml, tracker);
    }
}

fn render_element(
    introspector: &XsdIntrospector,
    el: &ElementMeta,
    prefix: Option<&str>,
    indent: usize,
    xml: &mut String,
    tracker: &mut crate::types::VisitTracker,
) {
    let name = el.ident.name.as_str();

    match &el.variant {
        ElementMetaVariant::Text => {
            let ind = "  ".repeat(indent);
            let pref = prefix.map(|p| format!("{p}:")).unwrap_or_default();
            xml.push_str(&format!("{ind}<{pref}{name}>string</{pref}{name}>\n"));
        }
        ElementMetaVariant::Any { .. } => {
            let ind = "  ".repeat(indent);
            xml.push_str(&format!("{ind}<!-- xs:any element -->\n"));
        }
        ElementMetaVariant::Type { type_, .. } => {
            let type_name = type_.name.as_str().to_string();

            // If the element name is auto-generated (e.g. "Content8" from a choice group)
            // and the type resolves to a Choice, inline the choice contents without a wrapper.
            if el.ident.name.is_generated()
                && let Some(mt) = introspector.get_resolved_by_name(&type_name)
            {
                match &mt.variant {
                    MetaTypeVariant::Choice(group) => {
                        let choice_ind = "  ".repeat(indent);
                        xml.push_str(&format!(
                            "{choice_ind}<!-- Choice: use one of the following -->\n"
                        ));
                        for (i, choice_el) in group.elements.iter().enumerate() {
                            if i > 0 {
                                xml.push_str(&format!("{choice_ind}<!-- OR -->\n"));
                            }
                            render_element(introspector, choice_el, prefix, indent, xml, tracker);
                        }
                        return;
                    }
                    MetaTypeVariant::Sequence(group) | MetaTypeVariant::All(group) => {
                        // Inline generated sequence/all groups too
                        render_group_elements(introspector, group, prefix, indent, xml, tracker);
                        return;
                    }
                    _ => {}
                }
            }

            render_type_by_ident(introspector, &type_name, name, prefix, indent, xml, tracker);
        }
    }
}

fn builtin_to_sample(builtin: &BuildInMeta) -> &'static str {
    match builtin {
        BuildInMeta::String | BuildInMeta::Str => "string",
        BuildInMeta::Bool => "false",
        BuildInMeta::U8
        | BuildInMeta::U16
        | BuildInMeta::U32
        | BuildInMeta::U64
        | BuildInMeta::U128
        | BuildInMeta::Usize
        | BuildInMeta::I8
        | BuildInMeta::I16
        | BuildInMeta::I32
        | BuildInMeta::I64
        | BuildInMeta::I128
        | BuildInMeta::Isize => "0",
        BuildInMeta::F32 | BuildInMeta::F64 => "0.0",
    }
}

fn first_enum_value(enumeration: &EnumerationMeta) -> String {
    enumeration
        .variants
        .first()
        .map(|v| v.ident.name.as_str().to_string())
        .unwrap_or_else(|| "enum_value".to_string())
}

fn simple_type_sample(simple: &SimpleMeta) -> String {
    let base_name = simple.base.name.as_str();
    crate::types::sample_value_for_builtin(base_name).to_string()
}
