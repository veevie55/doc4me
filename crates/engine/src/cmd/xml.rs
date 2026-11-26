//! Custom XML stores: schema references, element attributes, and the editor's XML preferences.
//!
//! DOCX custom XML is carried opaquely for round-trip. These commands inspect and edit only the
//! selected, bounded XML store or its datastore-properties part; they never evaluate XPath or
//! fetch a schema URL.

use std::collections::HashMap;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{CmdError, CmdResult, CommandSpec, Session, p};

const ROOTS_KEY: &str = "doc4me:customXml.roots";
const MAX_STORES: usize = 16;
const MAX_ROOTS_BYTES: usize = 64 * 1024;
const MAX_STORE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_STORE_BYTES: usize = 8 * 1024 * 1024;
const MAX_XML_DEPTH: usize = 128;
const MAX_XML_ELEMENTS: usize = 512;
const MAX_ATTRS_PER_ELEMENT: usize = 128;
const MAX_ATTR_NAME: usize = 512;
const MAX_ATTR_VALUE: usize = 4096;
const MAX_SCHEMA_URI: usize = 2048;
const CUSTOM_XML_PROPS_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/customXml";
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";
const PACKAGE_RELATIONSHIPS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const STRICT_PACKAGE_RELATIONSHIPS_NS: &str = "http://purl.oclc.org/ooxml/package/relationships";
const CUSTOM_XML_PROPS_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps";
const STRICT_CUSTOM_XML_PROPS_REL: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/customXmlProps";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct XmlAttribute {
    name: String,
    value: String,
    namespace: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct XmlElement {
    path: String,
    name: String,
    namespace: Option<String>,
    attributes: Vec<XmlAttribute>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct XmlStoreInfo {
    id: String,
    bytes: usize,
    properties_part: Option<String>,
    schemas: Vec<String>,
    elements: Vec<XmlElement>,
    total_elements: usize,
    editable: bool,
    error: Option<String>,
}

#[derive(Default)]
struct PathNode {
    path: String,
    name: String,
    children: HashMap<String, usize>,
    namespaces: HashMap<String, String>,
}

fn element_path(stack: &mut [PathNode], roots: &mut HashMap<String, usize>, name: &str) -> String {
    let count = if let Some(parent) = stack.last_mut() {
        parent.children.entry(name.to_string()).or_default()
    } else {
        roots.entry(name.to_string()).or_default()
    };
    *count += 1;
    let index = *count;
    let parent = stack.last().map_or("", |p| p.path.as_str());
    format!("{parent}/{name}[{index}]")
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

fn qname(bytes: &[u8]) -> Result<String, String> {
    let name = std::str::from_utf8(bytes).map_err(|_| "XML name is not UTF-8".to_string())?;
    if name.is_empty() || name.len() > MAX_ATTR_NAME || !valid_xml_qname(name) {
        return Err("XML contains an unsupported name".into());
    }
    Ok(name.to_string())
}

fn valid_xml_qname(name: &str) -> bool {
    let mut parts = name.split(':');
    let first = parts.next().unwrap_or_default();
    let valid_part = |p: &str| {
        let mut chars = p.chars();
        chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
            && chars.all(|c| c == '_' || c == '-' || c == '.' || c.is_alphanumeric() || c == '\u{B7}')
    };
    let second = parts.next();
    valid_part(first) && second.is_none_or(valid_part) && parts.next().is_none()
}

fn xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r')
        || (' '..='\u{D7FF}').contains(&c)
        || ('\u{E000}'..='\u{FFFD}').contains(&c)
        || ('\u{10000}'..='\u{10FFFF}').contains(&c)
}

fn validate_entity_reference(reference: &quick_xml::events::BytesRef<'_>) -> Result<(), String> {
    if let Some(character) = reference.resolve_char_ref().map_err(|error| format!("invalid XML character reference: {error}"))? {
        if !xml_char(character) {
            return Err("XML contains a forbidden character reference".into());
        }
        return Ok(());
    }
    match reference.decode().map_err(|error| format!("invalid XML entity reference: {error}"))?.as_ref() {
        "lt" | "gt" | "amp" | "apos" | "quot" => Ok(()),
        _ => Err("custom XML uses an undeclared entity reference".into()),
    }
}

fn namespace_for(qname: &str, namespaces: &HashMap<String, String>, attribute: bool) -> Option<String> {
    if qname == "xmlns" || qname.starts_with("xmlns:") {
        return Some(XMLNS_NAMESPACE.into());
    }
    if let Some((prefix, _)) = qname.split_once(':') {
        if prefix == "xml" {
            return Some(XML_NAMESPACE.into());
        }
        return namespaces.get(prefix).cloned();
    }
    if attribute { None } else { namespaces.get("").cloned() }
}

fn attributes(e: &BytesStart<'_>, reader: &Reader<&[u8]>) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|e| format!("malformed XML attribute: {e}"))?;
        let name = qname(attr.key.as_ref())?;
        let value = attr
            .decoded_and_normalized_value(quick_xml::XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|e| format!("malformed XML attribute value: {e}"))?
            .into_owned();
        if value.len() > MAX_ATTR_VALUE || out.iter().any(|(n, _)| n == &name) || out.len() >= MAX_ATTRS_PER_ELEMENT {
            return Err("XML element has duplicate or oversized attributes".into());
        }
        out.push((name, value));
    }
    Ok(out)
}

fn scopes(parent: Option<&HashMap<String, String>>, attrs: &[(String, String)]) -> HashMap<String, String> {
    let mut scope = parent.cloned().unwrap_or_default();
    for (name, value) in attrs {
        if name == "xmlns" {
            scope.insert(String::new(), value.clone());
        } else if let Some(prefix) = name.strip_prefix("xmlns:") {
            scope.insert(prefix.to_string(), value.clone());
        }
    }
    scope
}

fn inspect_xml(bytes: &[u8]) -> Result<(Vec<XmlElement>, usize), String> {
    if bytes.len() > MAX_STORE_BYTES {
        return Err(format!("custom XML part exceeds {MAX_STORE_BYTES} bytes"));
    }
    let source = std::str::from_utf8(bytes).map_err(|_| "editing this XML part requires UTF-8".to_string())?;
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<PathNode> = Vec::new();
    let mut roots = HashMap::new();
    let mut elements = Vec::new();
    let mut total = 0;
    let mut root_count = 0;
    loop {
        match reader.read_event().map_err(|e| format!("malformed custom XML: {e}"))? {
            Event::Start(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                if stack.is_empty() {
                    root_count += 1;
                    if root_count > 1 {
                        return Err("custom XML has more than one root element".into());
                    }
                }
                let path = element_path(&mut stack, &mut roots, &name);
                let namespaces = scopes(stack.last().map(|n| &n.namespaces), &attrs);
                validate_namespaces(&name, &attrs, &namespaces)?;
                let node_name = name.clone();
                let view = xml_element(path.clone(), name, attrs, &namespaces);
                total += 1;
                if total > MAX_XML_ELEMENTS {
                    return Err(format!("custom XML has more than {MAX_XML_ELEMENTS} elements"));
                }
                elements.push(view);
                if stack.len() >= MAX_XML_DEPTH {
                    return Err(format!("custom XML exceeds {MAX_XML_DEPTH} nested elements"));
                }
                stack.push(PathNode { path, name: node_name, children: HashMap::new(), namespaces });
            }
            Event::Empty(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                if stack.is_empty() {
                    root_count += 1;
                    if root_count > 1 {
                        return Err("custom XML has more than one root element".into());
                    }
                }
                let path = element_path(&mut stack, &mut roots, &name);
                let namespaces = scopes(stack.last().map(|n| &n.namespaces), &attrs);
                validate_namespaces(&name, &attrs, &namespaces)?;
                total += 1;
                if total > MAX_XML_ELEMENTS {
                    return Err(format!("custom XML has more than {MAX_XML_ELEMENTS} elements"));
                }
                elements.push(xml_element(path, name, attrs, &namespaces));
            }
            Event::End(e) => {
                let name = qname(e.name().as_ref())?;
                if stack.pop().is_none_or(|node| node.name != name) {
                    return Err("custom XML has an unmatched end tag".into());
                }
            }
            Event::GeneralRef(reference) => validate_entity_reference(&reference)?,
            Event::DocType(_) => return Err("custom XML with a document type declaration cannot be edited".into()),
            Event::Eof => break,
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err("custom XML has unclosed elements".into());
    }
    if root_count != 1 {
        return Err("custom XML must have exactly one root element".into());
    }
    Ok((elements, total))
}

fn validate_namespaces(name: &str, attrs: &[(String, String)], namespaces: &HashMap<String, String>) -> Result<(), String> {
    let valid = |name: &str| {
        let Some((prefix, _)) = name.split_once(':') else { return true };
        prefix == "xml" || namespaces.get(prefix).is_some_and(|uri| !uri.is_empty())
    };
    for (prefix, uri) in namespaces {
        if prefix == "xmlns"
            || (prefix == "xml" && uri != XML_NAMESPACE)
            || (prefix != "xml" && uri == XML_NAMESPACE)
            || uri == XMLNS_NAMESPACE
            || (!prefix.is_empty() && uri.is_empty())
        {
            return Err("custom XML contains an invalid namespace binding".into());
        }
    }
    if !valid(name) || attrs.iter().any(|(name, _)| !name.starts_with("xmlns:") && !valid(name)) {
        return Err("custom XML contains an undeclared namespace prefix".into());
    }
    Ok(())
}

fn xml_element(path: String, name: String, attrs: Vec<(String, String)>, namespaces: &HashMap<String, String>) -> XmlElement {
    XmlElement {
        path,
        namespace: namespace_for(&name, namespaces, false),
        name,
        attributes: attrs.into_iter().map(|(name, value)| XmlAttribute { namespace: namespace_for(&name, namespaces, true), name, value }).collect(),
    }
}

fn valid_store_path(path: &str) -> bool {
    path.len() <= 512
        && path.starts_with("customXml/")
        && path.to_ascii_lowercase().ends_with(".xml")
        && !path.contains(['\\', ':', '\0', '\t', '\r', '\n'])
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn roots(doc: &doc4me_doc::Document) -> Vec<String> {
    let Some(bytes) = doc.passthrough.get(ROOTS_KEY) else { return Vec::new() };
    if bytes.len() > MAX_ROOTS_BYTES {
        return Vec::new();
    }
    let mut found = Vec::new();
    String::from_utf8_lossy(bytes)
        .lines()
        .take(MAX_STORES * 2)
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let owner = fields.next()?;
            let kind = fields.next()?;
            let part = fields.next()?;
            (matches!(owner, "P" | "D") && kind.rsplit('/').next() == Some("customXml") && fields.next().is_none() && valid_store_path(part))
                .then(|| part.to_string())
        })
        .filter(|part| {
            if found.len() >= MAX_STORES || found.contains(part) {
                false
            } else {
                found.push(part.clone());
                true
            }
        })
        .collect()
}

fn resolve_part(base_part: &str, target: &str) -> Option<String> {
    if target.is_empty() || target.len() > 512 || target.contains(['\\', ':', '\0', '\t', '\r', '\n']) {
        return None;
    }
    let mut components: Vec<&str> =
        if target.starts_with('/') { Vec::new() } else { base_part.rsplit_once('/').map(|(d, _)| d.split('/').collect()).unwrap_or_default() };
    for part in target.trim_start_matches('/').split('/') {
        match part {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            c => components.push(c),
        }
    }
    let out = components.join("/");
    (out.starts_with("customXml/") && valid_store_path(&out)).then_some(out)
}

fn read_relationship_target(bytes: &[u8]) -> Option<String> {
    if bytes.len() > 64 * 1024 {
        return None;
    }
    let source = std::str::from_utf8(bytes).ok()?;
    let mut reader = Reader::from_str(source);
    let mut namespace_stack: Vec<HashMap<String, String>> = Vec::new();
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) => {
                let name = qname(e.name().as_ref()).ok()?;
                let attrs = attributes(&e, &reader).ok()?;
                let namespaces = scopes(namespace_stack.last(), &attrs);
                if namespace_stack.is_empty()
                    && (local_name(&name) != "Relationships"
                        || !matches!(
                            namespace_for(&name, &namespaces, false).as_deref(),
                            Some(PACKAGE_RELATIONSHIPS_NS | STRICT_PACKAGE_RELATIONSHIPS_NS)
                        ))
                {
                    return None;
                }
                if namespace_stack.len() == 1
                    && local_name(&name) == "Relationship"
                    && matches!(namespace_for(&name, &namespaces, false).as_deref(), Some(PACKAGE_RELATIONSHIPS_NS | STRICT_PACKAGE_RELATIONSHIPS_NS))
                {
                    let get = |name: &str| attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str());
                    let exact_type = matches!(get("Type"), Some(CUSTOM_XML_PROPS_REL | STRICT_CUSTOM_XML_PROPS_REL));
                    if exact_type && get("TargetMode") != Some("External") {
                        return get("Target").map(str::to_string);
                    }
                }
                namespace_stack.push(namespaces);
            }
            Event::Empty(e) => {
                let name = qname(e.name().as_ref()).ok()?;
                let attrs = attributes(&e, &reader).ok()?;
                let namespaces = scopes(namespace_stack.last(), &attrs);
                if namespace_stack.is_empty() {
                    return None;
                }
                if namespace_stack.len() == 1
                    && local_name(&name) == "Relationship"
                    && matches!(namespace_for(&name, &namespaces, false).as_deref(), Some(PACKAGE_RELATIONSHIPS_NS | STRICT_PACKAGE_RELATIONSHIPS_NS))
                {
                    let get = |name: &str| attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str());
                    let exact_type = matches!(get("Type"), Some(CUSTOM_XML_PROPS_REL | STRICT_CUSTOM_XML_PROPS_REL));
                    if exact_type && get("TargetMode") != Some("External") {
                        return get("Target").map(str::to_string);
                    }
                }
            }
            Event::End(e) => {
                let _name = qname(e.name().as_ref()).ok()?;
                namespace_stack.pop()?;
            }
            Event::DocType(_) | Event::Eof => break,
            _ => {}
        }
    }
    None
}

fn properties_part(doc: &doc4me_doc::Document, store: &str) -> Option<String> {
    let (dir, file) = store.rsplit_once('/')?;
    let rels = format!("{dir}/_rels/{file}.rels");
    let bytes = doc.passthrough.get(&rels)?;
    let target = read_relationship_target(bytes)?;
    resolve_part(store, &target)
}

fn schema_refs(bytes: &[u8]) -> Result<Vec<String>, String> {
    if bytes.len() > MAX_STORE_BYTES {
        return Err(format!("custom XML properties exceed {MAX_STORE_BYTES} bytes"));
    }
    inspect_xml(bytes).map_err(|error| format!("invalid datastore properties XML: {error}"))?;
    let source = std::str::from_utf8(bytes).map_err(|_| "editing this XML part requires UTF-8".to_string())?;
    let mut reader = Reader::from_str(source);
    let mut found = Vec::new();
    let mut refs_depth = 0usize;
    let mut names: Vec<String> = Vec::new();
    let mut namespace_stack: Vec<HashMap<String, String>> = Vec::new();
    loop {
        match reader.read_event().map_err(|e| format!("malformed XML properties: {e}"))? {
            Event::Start(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                let namespaces = scopes(namespace_stack.last(), &attrs);
                if is_custom_xml_element(&name, "schemaRefs", &namespaces) {
                    if refs_depth >= MAX_XML_DEPTH {
                        return Err(format!("datastore properties exceed {MAX_XML_DEPTH} nested schemaRefs containers"));
                    }
                    refs_depth = refs_depth.saturating_add(1);
                } else if is_custom_xml_element(&name, "schemaRef", &namespaces) && refs_depth > 0 {
                    collect_schema_uri(&e, &reader, &namespaces, &mut found)?;
                }
                names.push(name);
                namespace_stack.push(namespaces);
            }
            Event::Empty(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                let namespaces = scopes(namespace_stack.last(), &attrs);
                if is_custom_xml_element(&name, "schemaRef", &namespaces) && refs_depth > 0 {
                    collect_schema_uri(&e, &reader, &namespaces, &mut found)?;
                }
            }
            Event::End(e) => {
                let name = qname(e.name().as_ref())?;
                let Some(open_name) = names.pop() else { return Err("malformed XML properties: unmatched end element".into()) };
                if open_name != name {
                    return Err("malformed XML properties: mismatched end element".into());
                }
                let Some(namespaces) = namespace_stack.pop() else { return Err("malformed XML properties: unmatched end element".into()) };
                if is_custom_xml_element(&name, "schemaRefs", &namespaces) {
                    refs_depth = refs_depth.saturating_sub(1);
                }
            }
            Event::GeneralRef(reference) => validate_entity_reference(&reference)?,
            Event::DocType(_) => return Err("XML properties contain a forbidden document type".into()),
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(found)
}

fn is_custom_xml_element(name: &str, local: &str, namespaces: &HashMap<String, String>) -> bool {
    local_name(name) == local && namespace_for(name, namespaces, false).as_deref() == Some(CUSTOM_XML_PROPS_NS)
}

fn collect_schema_uri(
    e: &BytesStart<'_>,
    reader: &Reader<&[u8]>,
    namespaces: &HashMap<String, String>,
    found: &mut Vec<String>,
) -> Result<(), String> {
    if let Some(uri) = schema_uri_attribute(&attributes(e, reader).map_err(|e| format!("malformed schema reference: {e}"))?, namespaces) {
        if uri.len() > MAX_SCHEMA_URI {
            return Err("schema reference URI exceeds the supported size".into());
        }
        if !uri.is_empty() && !found.contains(&uri) {
            if found.len() >= 256 {
                return Err("datastore properties contain too many schema references".into());
            }
            found.push(uri);
        }
    }
    Ok(())
}

fn schema_uri_attribute(attrs: &[(String, String)], namespaces: &HashMap<String, String>) -> Option<String> {
    attrs
        .iter()
        .find(|(name, _)| local_name(name) == "uri" && namespace_for(name, namespaces, true).as_deref() == Some(CUSTOM_XML_PROPS_NS))
        .map(|(_, uri)| uri.clone())
}

fn store_infos(s: &Session) -> Vec<XmlStoreInfo> {
    let mut inspected_bytes = 0usize;
    roots(&s.doc)
        .into_iter()
        .filter_map(|id| {
            let bytes = s.doc.passthrough.get(&id)?;
            let props = properties_part(&s.doc, &id);
            let property_bytes = props.as_ref().and_then(|p| s.doc.passthrough.get(p));
            let cost = bytes.len().saturating_add(property_bytes.map_or(0, |part| part.len()));
            if inspected_bytes.saturating_add(cost) > MAX_TOTAL_STORE_BYTES {
                return Some(XmlStoreInfo {
                    id,
                    bytes: bytes.len(),
                    properties_part: props,
                    schemas: Vec::new(),
                    elements: Vec::new(),
                    total_elements: 0,
                    editable: false,
                    error: Some(format!(
                        "document XML inspection is limited to {MAX_TOTAL_STORE_BYTES} bytes; this store is preserved but not shown"
                    )),
                });
            }
            inspected_bytes += cost;
            let schemas = props.as_ref().and(property_bytes).and_then(|b| schema_refs(b).ok()).unwrap_or_default();
            match inspect_xml(bytes) {
                Ok((elements, total)) => Some(XmlStoreInfo {
                    id,
                    bytes: bytes.len(),
                    properties_part: props,
                    schemas,
                    elements,
                    total_elements: total,
                    editable: true,
                    error: None,
                }),
                Err(error) => Some(XmlStoreInfo {
                    id,
                    bytes: bytes.len(),
                    properties_part: props,
                    schemas,
                    elements: Vec::new(),
                    total_elements: 0,
                    editable: false,
                    error: Some(error),
                }),
            }
        })
        .collect()
}

fn uri_ok(uri: &str) -> bool {
    if uri.len() > MAX_SCHEMA_URI || uri.is_empty() || uri.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return false;
    }
    let lower = uri.to_ascii_lowercase();
    for scheme in ["http://", "https://"] {
        if lower.starts_with(scheme) {
            let authority = uri.get(scheme.len()..).unwrap_or_default().split(['/', '?', '#']).next().unwrap_or_default();
            let host_port = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
            let host = if host_port.starts_with('[') {
                host_port.split_once(']').map(|(host, _)| host.trim_start_matches('[')).unwrap_or_default()
            } else {
                host_port.split(':').next().unwrap_or_default()
            };
            return !host.is_empty();
        }
    }
    lower.strip_prefix("urn:").is_some_and(|namespace| !namespace.is_empty())
}

fn rewrite_schema_refs(bytes: &[u8], uri: &str, add: bool) -> Result<(Vec<u8>, bool), String> {
    let source = std::str::from_utf8(bytes).map_err(|_| "editing this XML part requires UTF-8".to_string())?;
    if source.len() > MAX_STORE_BYTES {
        return Err(format!("custom XML properties exceed {MAX_STORE_BYTES} bytes"));
    }
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::with_capacity(source.len().saturating_add(128)));
    let mut in_schema_refs = false;
    let mut seen_schema_refs = false;
    let mut changed = false;
    let mut schema_prefix = "ds".to_string();
    let mut skip_schema_ref_depth = 0usize;
    let mut depth = 0usize;
    let mut namespace_stack: Vec<HashMap<String, String>> = Vec::new();
    let existing = schema_refs(bytes)?;
    loop {
        match reader.read_event().map_err(|e| format!("malformed XML properties: {e}"))? {
            Event::Start(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                let namespaces = scopes(namespace_stack.last(), &attrs);
                if skip_schema_ref_depth > 0 {
                    skip_schema_ref_depth = skip_schema_ref_depth.saturating_add(1);
                    depth = depth.saturating_add(1);
                    namespace_stack.push(namespaces);
                    continue;
                }
                if is_custom_xml_element(&name, "schemaRefs", &namespaces) {
                    if seen_schema_refs {
                        return Err("datastore properties have multiple schemaRefs containers".into());
                    }
                    seen_schema_refs = true;
                    in_schema_refs = true;
                    schema_prefix = name.split_once(':').map_or("ds", |(p, _)| p).to_string();
                } else if is_custom_xml_element(&name, "schemaRef", &namespaces) && in_schema_refs {
                    let uri_value = schema_uri_attribute(&attrs, &namespaces);
                    if !add && uri_value.as_deref() == Some(uri) {
                        skip_schema_ref_depth = 1;
                        depth = depth.saturating_add(1);
                        namespace_stack.push(namespaces);
                        changed = true;
                        continue;
                    }
                }
                writer.write_event(Event::Start(e)).map_err(|e| format!("could not write XML properties: {e}"))?;
                depth = depth.saturating_add(1);
                namespace_stack.push(namespaces);
            }
            Event::Empty(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                let namespaces = scopes(namespace_stack.last(), &attrs);
                if skip_schema_ref_depth > 0 {
                    continue;
                }
                if is_custom_xml_element(&name, "schemaRef", &namespaces) && in_schema_refs {
                    let existing = schema_uri_attribute(&attrs, &namespaces);
                    if existing.as_deref() == Some(uri) && !add {
                        changed = true;
                        continue;
                    }
                }
                if is_custom_xml_element(&name, "schemaRefs", &namespaces) {
                    if seen_schema_refs {
                        return Err("datastore properties have multiple schemaRefs containers".into());
                    }
                    seen_schema_refs = true;
                    if add {
                        schema_prefix = name.split_once(':').map_or("ds", |(p, _)| p).to_string();
                        let mut start = BytesStart::new(name.clone());
                        for (key, value) in attributes(&e, &reader)? {
                            start.push_attribute((key.as_str(), value.as_str()));
                        }
                        writer.write_event(Event::Start(start)).map_err(|e| format!("could not write XML properties: {e}"))?;
                        write_schema_ref(&mut writer, &schema_prefix, uri)?;
                        writer
                            .write_event(Event::End(quick_xml::events::BytesEnd::new(name)))
                            .map_err(|e| format!("could not write XML properties: {e}"))?;
                        changed = true;
                        continue;
                    }
                }
                if depth == 0 && add && !seen_schema_refs {
                    let mut start = BytesStart::new(name.clone());
                    for (key, value) in attributes(&e, &reader)? {
                        start.push_attribute((key.as_str(), value.as_str()));
                    }
                    writer.write_event(Event::Start(start)).map_err(|e| format!("could not write XML properties: {e}"))?;
                    write_schema_refs(&mut writer, &schema_prefix, uri)?;
                    writer
                        .write_event(Event::End(quick_xml::events::BytesEnd::new(name)))
                        .map_err(|e| format!("could not write XML properties: {e}"))?;
                    seen_schema_refs = true;
                    changed = true;
                    continue;
                }
                writer.write_event(Event::Empty(e)).map_err(|e| format!("could not write XML properties: {e}"))?;
            }
            Event::End(e) => {
                let name = qname(e.name().as_ref())?;
                if skip_schema_ref_depth > 0 {
                    skip_schema_ref_depth -= 1;
                    depth = depth.saturating_sub(1);
                    namespace_stack.pop();
                    continue;
                }
                let Some(namespaces) = namespace_stack.pop() else { return Err("malformed XML properties: unmatched end element".into()) };
                if is_custom_xml_element(&name, "schemaRefs", &namespaces) && in_schema_refs {
                    if add && !existing.iter().any(|old| old == uri) {
                        write_schema_ref(&mut writer, &schema_prefix, uri)?;
                        changed = true;
                    }
                    in_schema_refs = false;
                }
                if depth == 1 && add && !seen_schema_refs {
                    write_schema_refs(&mut writer, &schema_prefix, uri)?;
                    seen_schema_refs = true;
                    changed = true;
                }
                depth = depth.saturating_sub(1);
                writer.write_event(Event::End(e)).map_err(|e| format!("could not write XML properties: {e}"))?;
            }
            Event::GeneralRef(reference) => {
                validate_entity_reference(&reference)?;
                if skip_schema_ref_depth == 0 {
                    writer.write_event(Event::GeneralRef(reference)).map_err(|e| format!("could not write XML properties: {e}"))?;
                }
            }
            Event::DocType(_) => return Err("XML properties contain a forbidden document type".into()),
            Event::Eof => break,
            event if skip_schema_ref_depth == 0 => writer.write_event(event).map_err(|e| format!("could not write XML properties: {e}"))?,
            _ => {}
        }
    }
    if skip_schema_ref_depth != 0 {
        return Err("schema reference has unclosed elements".into());
    }
    let out = writer.into_inner();
    if out.len() > MAX_STORE_BYTES {
        return Err(format!("updated XML properties exceed {MAX_STORE_BYTES} bytes"));
    }
    // Always validate the result before replacing the opaque source bytes.
    schema_refs(&out)?;
    Ok((out, changed))
}

fn write_schema_refs(writer: &mut Writer<Vec<u8>>, prefix: &str, uri: &str) -> Result<(), String> {
    let name = format!("{prefix}:schemaRefs");
    let namespace_name = format!("xmlns:{prefix}");
    let mut refs = BytesStart::new(name.clone());
    refs.push_attribute((namespace_name.as_str(), CUSTOM_XML_PROPS_NS));
    writer.write_event(Event::Start(refs)).map_err(|e| format!("could not write schema references: {e}"))?;
    write_schema_ref(writer, prefix, uri)?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new(name))).map_err(|e| format!("could not close schema references: {e}"))
}

fn write_schema_ref(writer: &mut Writer<Vec<u8>>, prefix: &str, uri: &str) -> Result<(), String> {
    let mut item = BytesStart::new(format!("{prefix}:schemaRef"));
    let namespace_name = format!("xmlns:{prefix}");
    let uri_name = format!("{prefix}:uri");
    item.push_attribute((namespace_name.as_str(), CUSTOM_XML_PROPS_NS));
    item.push_attribute((uri_name.as_str(), uri));
    writer.write_event(Event::Empty(item)).map_err(|e| format!("could not write schema reference: {e}"))
}

fn path_for_next(stack: &mut [PathNode], roots: &mut HashMap<String, usize>, name: &str) -> String {
    element_path(stack, roots, name)
}

fn edit_attribute(bytes: &[u8], wanted_path: &str, attr_name: &str, value: Option<&str>) -> Result<(Vec<u8>, bool), String> {
    if wanted_path.len() > 2048 || !wanted_path.starts_with('/') {
        return Err("`path` must be an XML element path returned by this command".into());
    }
    if attr_name.len() > MAX_ATTR_NAME || !valid_xml_qname(attr_name) || attr_name == "xmlns" || attr_name.starts_with("xmlns:") {
        return Err("`name` must be a valid non-namespace XML attribute name".into());
    }
    if let Some(value) = value
        && (value.len() > MAX_ATTR_VALUE || !value.chars().all(xml_char))
    {
        return Err(format!("attribute values must be valid XML text up to {MAX_ATTR_VALUE} bytes"));
    }
    let source = std::str::from_utf8(bytes).map_err(|_| "editing this XML part requires UTF-8".to_string())?;
    if source.len() > MAX_STORE_BYTES {
        return Err(format!("custom XML part exceeds {MAX_STORE_BYTES} bytes"));
    }
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::with_capacity(source.len().saturating_add(64)));
    let mut stack: Vec<PathNode> = Vec::new();
    let mut root_counts = HashMap::new();
    let mut found = false;
    let mut changed = false;
    let mut root_count = 0usize;
    let mut total = 0usize;
    loop {
        match reader.read_event().map_err(|e| format!("malformed custom XML: {e}"))? {
            Event::Start(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                if stack.is_empty() {
                    root_count += 1;
                    if root_count > 1 {
                        return Err("custom XML has more than one root element".into());
                    }
                }
                let path = path_for_next(&mut stack, &mut root_counts, &name);
                let ns = scopes(stack.last().map(|n| &n.namespaces), &attrs);
                validate_namespaces(&name, &attrs, &ns)?;
                if path == wanted_path {
                    found = true;
                    let (rewritten, did_change) = rewrite_attributes(&name, attrs, &ns, attr_name, value)?;
                    writer.write_event(Event::Start(rewritten)).map_err(|e| format!("could not write custom XML: {e}"))?;
                    changed |= did_change;
                } else {
                    writer.write_event(Event::Start(e)).map_err(|e| format!("could not write custom XML: {e}"))?;
                }
                total += 1;
                if total > MAX_XML_ELEMENTS {
                    return Err(format!("custom XML has more than {MAX_XML_ELEMENTS} elements"));
                }
                if stack.len() >= MAX_XML_DEPTH {
                    return Err(format!("custom XML exceeds {MAX_XML_DEPTH} nested elements"));
                }
                stack.push(PathNode { path, name, children: HashMap::new(), namespaces: ns });
            }
            Event::Empty(e) => {
                let name = qname(e.name().as_ref())?;
                let attrs = attributes(&e, &reader)?;
                if stack.is_empty() {
                    root_count += 1;
                    if root_count > 1 {
                        return Err("custom XML has more than one root element".into());
                    }
                }
                let path = path_for_next(&mut stack, &mut root_counts, &name);
                let ns = scopes(stack.last().map(|n| &n.namespaces), &attrs);
                validate_namespaces(&name, &attrs, &ns)?;
                if path == wanted_path {
                    found = true;
                    let (rewritten, did_change) = rewrite_attributes(&name, attrs, &ns, attr_name, value)?;
                    writer.write_event(Event::Empty(rewritten)).map_err(|e| format!("could not write custom XML: {e}"))?;
                    changed |= did_change;
                } else {
                    writer.write_event(Event::Empty(e)).map_err(|e| format!("could not write custom XML: {e}"))?;
                }
                total += 1;
                if total > MAX_XML_ELEMENTS {
                    return Err(format!("custom XML has more than {MAX_XML_ELEMENTS} elements"));
                }
            }
            Event::End(e) => {
                let name = qname(e.name().as_ref())?;
                if stack.pop().is_none_or(|node| node.name != name) {
                    return Err("custom XML has an unmatched end tag".into());
                }
                writer.write_event(Event::End(e)).map_err(|e| format!("could not write custom XML: {e}"))?;
            }
            Event::GeneralRef(reference) => {
                validate_entity_reference(&reference)?;
                writer.write_event(Event::GeneralRef(reference)).map_err(|e| format!("could not write custom XML: {e}"))?;
            }
            Event::DocType(_) => return Err("custom XML with a document type declaration cannot be edited".into()),
            Event::Eof => break,
            event => writer.write_event(event).map_err(|e| format!("could not write custom XML: {e}"))?,
        }
    }
    if !found {
        return Err("the XML element path no longer exists".into());
    }
    if !stack.is_empty() || root_count != 1 {
        return Err("custom XML must have exactly one root element".into());
    }
    if !changed {
        return Err("the attribute already has that value or does not exist".into());
    }
    let out = writer.into_inner();
    if out.len() > MAX_STORE_BYTES {
        return Err(format!("updated custom XML exceeds {MAX_STORE_BYTES} bytes"));
    }
    inspect_xml(&out)?;
    Ok((out, true))
}

fn rewrite_attributes(
    element: &str,
    attrs: Vec<(String, String)>,
    namespaces: &HashMap<String, String>,
    target: &str,
    value: Option<&str>,
) -> Result<(BytesStart<'static>, bool), String> {
    if let Some((prefix, _)) = target.split_once(':')
        && prefix != "xml"
        && !namespaces.contains_key(prefix)
    {
        return Err(format!("attribute prefix `{prefix}` is not declared on this element"));
    }
    let exists = attrs.iter().any(|(name, _)| name == target);
    if value.is_none() && !exists {
        return Err("the selected XML element has no such attribute".into());
    }
    if value.is_some() && exists && attrs.iter().find(|(name, _)| name == target).map(|(_, v)| v.as_str()) == value {
        return Ok((build_element(element, attrs), false));
    }
    let mut out = BytesStart::new(element.to_string());
    for (name, old) in attrs.into_iter().filter(|(name, _)| name != target || value.is_some()) {
        let current = if name == target { value.unwrap_or_default() } else { old.as_str() };
        out.push_attribute((name.as_str(), current));
    }
    if !exists && let Some(value) = value {
        out.push_attribute((target, value));
    }
    Ok((out, true))
}

fn build_element(name: &str, attrs: Vec<(String, String)>) -> BytesStart<'static> {
    let mut out = BytesStart::new(name.to_string());
    for (key, value) in attrs {
        out.push_attribute((key.as_str(), value.as_str()));
    }
    out
}

fn schema_library(s: &mut Session, v: &Value) -> CmdResult {
    let action = p::str(v, "action").unwrap_or("list");
    let mut changed = false;
    if action != "list" {
        if !matches!(action, "add" | "remove") {
            return Err(CmdError::Params("`action` must be `list`, `add`, or `remove`".into()));
        }
        let store = p::req_str(v, "store")?;
        if !roots(&s.doc).iter().any(|root| root == store) {
            return Err(CmdError::Params("`store` must name a custom XML store in this document".into()));
        }
        let uri = p::req_str(v, "schema")?.trim();
        if !uri_ok(uri) {
            return Err(CmdError::Params("`schema` must be an absolute HTTP(S) or URN URI without whitespace".into()));
        }
        let props = properties_part(&s.doc, store)
            .ok_or_else(|| CmdError::Failed("this custom XML store has no editable datastore properties part".into()))?;
        let bytes = s.doc.passthrough.get(&props).ok_or_else(|| CmdError::Failed("the datastore properties part is missing".into()))?;
        let existing = schema_refs(bytes).map_err(CmdError::Failed)?;
        let add = action == "add";
        let already_in_requested_state = if add { existing.iter().any(|item| item == uri) } else { !existing.iter().any(|item| item == uri) };
        let (out, was_changed) = if already_in_requested_state {
            (bytes.as_ref().clone(), false)
        } else {
            rewrite_schema_refs(bytes, uri, add).map_err(CmdError::Failed)?
        };
        if was_changed {
            s.doc.passthrough.insert(props, Arc::new(out));
            changed = true;
        } else {
            s.only_read();
        }
    }
    if action == "list" && v.as_object().is_none_or(|m| m.is_empty()) {
        s.only_read();
        s.push_ui_request(json!({"open": "xmlSchemaLibrary"}));
    }
    if !changed {
        s.only_read();
    }
    Ok(json!({"stores": store_infos(s), "changed": changed, "opened": action == "list" && v.as_object().is_none_or(|m| m.is_empty())}))
}

fn xml_element_attributes(s: &mut Session, v: &Value) -> CmdResult {
    if v.as_object().is_none_or(|m| m.is_empty()) {
        s.only_read();
        s.push_ui_request(json!({"open": "xmlElementAttributes"}));
        return Ok(json!({"stores": store_infos(s), "opened": true}));
    }
    let Some(store) = v.get("store").and_then(Value::as_str) else {
        s.only_read();
        return Ok(json!({"stores": store_infos(s), "opened": false}));
    };
    if !roots(&s.doc).iter().any(|root| root == store) {
        return Err(CmdError::Params("`store` must name a custom XML store in this document".into()));
    }
    let Some(name) = v.get("name").and_then(Value::as_str) else {
        s.only_read();
        return Ok(json!({"stores": store_infos(s), "opened": false}));
    };
    let path = p::req_str(v, "path")?;
    let value = match v.get("value") {
        Some(Value::String(value)) => Some(value.as_str()),
        Some(Value::Null) => None,
        _ => return Err(CmdError::Params("`value` must be a string or null (null removes the attribute)".into())),
    };
    let bytes = s.doc.passthrough.get(store).ok_or_else(|| CmdError::Failed("custom XML store bytes are missing".into()))?;
    let (out, _) = edit_attribute(bytes, path, name, value).map_err(CmdError::Failed)?;
    s.doc.passthrough.insert(store.to_string(), Arc::new(out));
    Ok(json!({"stores": store_infos(s), "changed": true, "store": store, "path": path, "name": name, "value": value}))
}

fn xml_options(s: &mut Session, v: &Value) -> CmdResult {
    for key in ["showNamespaces", "confirmSchemaRemoval"] {
        if v.get(key).is_some_and(|value| !value.is_boolean()) {
            return Err(CmdError::Params(format!("`{key}` must be true or false")));
        }
    }
    let mut changed = false;
    if let Some(value) = v.get("showNamespaces").and_then(Value::as_bool)
        && s.prefs.xml_show_namespaces != value
    {
        s.prefs.xml_show_namespaces = value;
        changed = true;
    }
    if let Some(value) = v.get("confirmSchemaRemoval").and_then(Value::as_bool)
        && s.prefs.xml_confirm_schema_removal != value
    {
        s.prefs.xml_confirm_schema_removal = value;
        changed = true;
    }
    if v.as_object().is_none_or(|m| m.is_empty()) {
        s.only_read();
        s.push_ui_request(json!({"open": "xmlOptions"}));
    }
    s.only_read();
    Ok(
        json!({"showNamespaces": s.prefs.xml_show_namespaces, "confirmSchemaRemoval": s.prefs.xml_confirm_schema_removal, "changed": changed, "opened": v.as_object().is_none_or(|m| m.is_empty())}),
    )
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("file.schemaLibrary", "Schema Library", "File › Info › XML", schema_library)
            .params(r#"{"action"?: "list|add|remove", "store"?: "customXml/item1.xml", "schema"?: absolute HTTP(S) or URN URI} → this document's schema references; add/remove only changes its datastore-properties part; no params opens Schema Library"#),
        CommandSpec::new("file.xmlElementAttributes", "XML Element Attributes", "File › Info › XML", xml_element_attributes)
            .params(r#"{} → custom XML stores, bounded element paths and attributes; {"store", "path", "name", "value"?: string|null} edits or removes one attribute; no params opens XML Element Attributes"#),
        CommandSpec::new("file.xmlOptions", "XML Options", "File › Options › Advanced", xml_options)
            .params(r#"{"showNamespaces"?: bool, "confirmSchemaRemoval"?: bool} → custom XML editor preferences; no params opens XML Options"#),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r#"<?xml version="1.0"?>
<!-- keep this comment -->
<c:customer xmlns:c="urn:customer" xmlns:m="urn:meta"><c:name m:lang="en">Atlas</c:name><c:other/></c:customer>"#;
    const PROPS: &str = r#"<ds:datastoreItem xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml" ds:itemID="{22222222-2222-2222-2222-222222222222}"><ds:schemaRefs><ds:schemaRef ds:uri="urn:original:customer"/></ds:schemaRefs></ds:datastoreItem>"#;

    fn session() -> Session {
        let mut doc = doc4me_doc::Document::new();
        doc.passthrough.insert(
            ROOTS_KEY.into(),
            Arc::new(b"D\thttp://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml\tcustomXml/item1.xml\n".to_vec()),
        );
        doc.passthrough.insert("customXml/item1.xml".into(), Arc::new(DATA.as_bytes().to_vec()));
        doc.passthrough.insert(
            "customXml/_rels/item1.xml.rels".into(),
            Arc::new(br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="props" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="itemProps1.xml"/></Relationships>"#.to_vec()),
        );
        doc.passthrough.insert("customXml/itemProps1.xml".into(), Arc::new(PROPS.as_bytes().to_vec()));
        Session::new(doc)
    }

    #[test]
    fn inspection_lists_namespaces_paths_and_attributes() {
        let (elements, total) = inspect_xml(DATA.as_bytes()).unwrap();
        assert_eq!(total, 3);
        assert_eq!(elements[0].path, "/c:customer[1]");
        assert_eq!(elements[0].namespace.as_deref(), Some("urn:customer"));
        assert_eq!(elements[1].path, "/c:customer[1]/c:name[1]");
        assert_eq!(elements[1].attributes[0].name, "m:lang");
        assert_eq!(elements[1].attributes[0].namespace.as_deref(), Some("urn:meta"));
        assert_eq!(elements[1].attributes[0].value, "en");
    }

    #[test]
    fn attribute_edit_preserves_other_xml_content_and_supports_add_update_remove() {
        let (updated, changed) = edit_attribute(DATA.as_bytes(), "/c:customer[1]/c:name[1]", "m:lang", Some("fr & CA")).unwrap();
        assert!(changed);
        let updated_text = std::str::from_utf8(&updated).unwrap();
        assert!(updated_text.contains("keep this comment"));
        assert!(updated_text.contains("<c:other/>"));
        assert!(updated_text.contains("m:lang=\"fr &amp; CA\""));
        let (added, _) = edit_attribute(&updated, "/c:customer[1]/c:other[1]", "m:source", Some("user")).unwrap();
        assert!(std::str::from_utf8(&added).unwrap().contains("m:source=\"user\""));
        let (removed, _) = edit_attribute(&added, "/c:customer[1]/c:name[1]", "m:lang", None).unwrap();
        assert!(!std::str::from_utf8(&removed).unwrap().contains("m:lang="));
        assert!(edit_attribute(&removed, "/c:customer[1]/c:name[1]", "m:lang", None).is_err());
        assert!(edit_attribute(DATA.as_bytes(), "/c:customer[1]/c:name[1]", "x:unknown", Some("v")).is_err());
    }

    #[test]
    fn unsafe_or_malformed_xml_is_never_rewritten() {
        assert!(inspect_xml(b"<!DOCTYPE x [<!ENTITY a 'bad'>]><x>&a;</x>").is_err());
        assert!(inspect_xml(b"<x><y></x>").is_err());
        assert!(inspect_xml(b"<x/><y/>").is_err());
        assert!(inspect_xml(b"<a:b:c xmlns:a=\"urn:a\" xmlns:b=\"urn:b\"/>").is_err());
        assert!(inspect_xml(b"<x:p xmlns:x=\"\"/>").is_err());
        assert!(inspect_xml(b"<x:p xmlns:xml=\"urn:wrong\"/>").is_err());
        assert!(edit_attribute(b"<x a='1' a='2'/>", "/x[1]", "a", Some("3")).is_err());
        assert!(edit_attribute(b"<x>&external;</x>", "/x[1]", "a", Some("3")).is_err());
    }

    #[test]
    fn schema_reference_add_remove_and_noop_are_bounded_and_valid() {
        let (added, changed) = rewrite_schema_refs(PROPS.as_bytes(), "https://schemas.example.test/customer.xsd", true).unwrap();
        assert!(changed);
        let listed = schema_refs(&added).unwrap();
        assert!(listed.contains(&"urn:original:customer".to_string()));
        assert!(listed.contains(&"https://schemas.example.test/customer.xsd".to_string()));
        let (same, changed) = rewrite_schema_refs(&added, "https://schemas.example.test/customer.xsd", true).unwrap();
        assert!(!changed);
        assert_eq!(same, added);
        let (removed, changed) = rewrite_schema_refs(&added, "urn:original:customer", false).unwrap();
        assert!(changed);
        assert_eq!(schema_refs(&removed).unwrap(), ["https://schemas.example.test/customer.xsd"]);
        let (removed_nonempty, changed) = rewrite_schema_refs(
            br#"<ds:datastoreItem xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"><ds:schemaRefs><ds:schemaRef ds:uri="urn:remove"><ds:note>discard with the reference</ds:note></ds:schemaRef></ds:schemaRefs></ds:datastoreItem>"#,
            "urn:remove",
            false,
        )
        .unwrap();
        assert!(changed);
        assert!(!String::from_utf8(removed_nonempty).unwrap().contains("discard with the reference"));
        let (created, changed) = rewrite_schema_refs(
            br#"<ds:datastoreItem xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"/>"#,
            "urn:new-store-schema",
            true,
        )
        .unwrap();
        assert!(changed);
        assert_eq!(schema_refs(&created).unwrap(), ["urn:new-store-schema"]);
    }

    #[test]
    fn schema_matching_uses_the_custom_xml_properties_namespace_only() {
        let props = br#"<ds:datastoreItem xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml" xmlns:x="urn:extension"><ds:schemaRefs><ds:schemaRef ds:uri="urn:real"/></ds:schemaRefs><x:schemaRefs><x:schemaRef x:uri="urn:foreign"/></x:schemaRefs></ds:datastoreItem>"#;
        assert_eq!(schema_refs(props).unwrap(), ["urn:real"]);
        let (foreign_unchanged, changed) = rewrite_schema_refs(props, "urn:foreign", false).unwrap();
        assert!(!changed);
        let foreign_text = String::from_utf8(foreign_unchanged).unwrap();
        assert!(foreign_text.contains("<x:schemaRefs>"));
        assert!(foreign_text.contains("<x:schemaRef x:uri=\"urn:foreign\"/>"));
        let (removed, changed) = rewrite_schema_refs(props, "urn:real", false).unwrap();
        assert!(changed);
        let removed_text = String::from_utf8(removed).unwrap();
        assert!(removed_text.contains("<x:schemaRef x:uri=\"urn:foreign\"/>"));
        assert!(!removed_text.contains("<ds:schemaRef "));
    }

    #[test]
    fn properties_relationship_requires_the_opc_namespace_and_exact_type() {
        let rels = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships" xmlns:x="urn:extension"><x:Relationship Id="extension" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="extension.xml"/><Relationship Id="lookalike" Type="urn:extension/customXmlProps" Target="lookalike.xml"/><Relationship Id="real" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="itemProps1.xml"/></Relationships>"#;
        assert_eq!(read_relationship_target(rels).as_deref(), Some("itemProps1.xml"));
        let strict = br#"<Relationships xmlns="http://purl.oclc.org/ooxml/package/relationships"><Relationship Id="real" Type="http://purl.oclc.org/ooxml/officeDocument/relationships/customXmlProps" Target="itemProps1.xml"/></Relationships>"#;
        assert_eq!(read_relationship_target(strict).as_deref(), Some("itemProps1.xml"));
        let wrong_root = br#"<x:Relationships xmlns:x="urn:extension"><x:Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="extension.xml"/></x:Relationships>"#;
        assert_eq!(read_relationship_target(wrong_root), None);
    }

    #[test]
    fn schema_library_and_attribute_commands_edit_custom_xml_parts() {
        let mut s = session();
        let added = schema_library(&mut s, &json!({"action":"add", "store":"customXml/item1.xml", "schema":"urn:customer:v2"})).unwrap();
        assert_eq!(added["changed"], true);
        assert!(schema_refs(&s.doc.passthrough["customXml/itemProps1.xml"]).unwrap().contains(&"urn:customer:v2".to_string()));
        let edited =
            xml_element_attributes(&mut s, &json!({"store":"customXml/item1.xml", "path":"/c:customer[1]/c:name[1]", "name":"m:lang", "value":"fr"}))
                .unwrap();
        assert_eq!(edited["changed"], true);
        assert!(std::str::from_utf8(&s.doc.passthrough["customXml/item1.xml"]).unwrap().contains("m:lang=\"fr\""));
        let removed = schema_library(&mut s, &json!({"action":"remove", "store":"customXml/item1.xml", "schema":"urn:customer:v2"})).unwrap();
        assert_eq!(removed["changed"], true);
        assert!(!schema_refs(&s.doc.passthrough["customXml/itemProps1.xml"]).unwrap().contains(&"urn:customer:v2".to_string()));
    }

    #[test]
    fn options_and_empty_invocations_are_read_only_dialog_requests() {
        let mut s = session();
        let opened = schema_library(&mut s, &json!({})).unwrap();
        assert_eq!(opened["opened"], true);
        assert_eq!(s.ui_requests.last().and_then(|v| v["open"].as_str()), Some("xmlSchemaLibrary"));
        let options = xml_options(&mut s, &json!({"showNamespaces":true})).unwrap();
        assert_eq!(options["showNamespaces"], true);
        assert_eq!(options["changed"], true);
        assert_eq!(xml_options(&mut s, &json!({"showNamespaces":true})).unwrap()["changed"], false);
        assert!(xml_options(&mut s, &json!({"showNamespaces":false, "confirmSchemaRemoval":"invalid"})).is_err());
        assert!(s.prefs.xml_show_namespaces, "an invalid request must not partially change preferences");
        assert!(!uri_ok("http://"));
        assert!(!uri_ok("https:///schema.xsd"));
        assert!(!uri_ok("urn:"));
        assert!(uri_ok("https://schemas.example.test/schema.xsd"));
        assert!(uri_ok("urn:customer"));
    }

    #[test]
    fn registered_custom_xml_edits_are_undoable_without_changing_unrelated_parts() {
        let mut s = session();
        let original_data = s.doc.passthrough["customXml/item1.xml"].clone();
        let original_properties = s.doc.passthrough["customXml/itemProps1.xml"].clone();
        s.run("file.xmlElementAttributes", &json!({"store":"customXml/item1.xml", "path":"/c:customer[1]/c:name[1]", "name":"m:lang", "value":"fr"}))
            .unwrap();
        assert_ne!(s.doc.passthrough["customXml/item1.xml"], original_data);
        assert_eq!(s.doc.passthrough["customXml/itemProps1.xml"], original_properties);
        assert!(s.undo());
        assert_eq!(s.doc.passthrough["customXml/item1.xml"], original_data);

        s.run("file.schemaLibrary", &json!({"action":"add", "store":"customXml/item1.xml", "schema":"urn:new-schema"})).unwrap();
        assert_ne!(s.doc.passthrough["customXml/itemProps1.xml"], original_properties);
        assert!(s.undo());
        assert_eq!(s.doc.passthrough["customXml/itemProps1.xml"], original_properties);
    }
}
