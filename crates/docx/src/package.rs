//! OPC package access: bounded zip reading, relationships, part names.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Cursor, Read, Write};

use crate::DocxError;
use crate::xml::{self, El};

/// Largest single part we decompress.
pub const MAX_PART: u64 = 256 * 1024 * 1024;
/// Largest total of all decompressed parts.
pub const MAX_TOTAL: u64 = 512 * 1024 * 1024;
/// Most zip entries we look at.
pub const MAX_ENTRIES: usize = 20_000;

/// Relationship type suffixes (after `.../relationships/`).
pub mod rt {
    pub const OFFICE_DOC: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
    pub const CORE: &str = "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
    pub const APP: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
    pub const CUSTOM: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
    pub const CUSTOM_XML_PROPS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps";
    pub const STYLES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
    pub const FONT_TABLE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable";
    pub const NUMBERING: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering";
    pub const SETTINGS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";
    pub const WEB_SETTINGS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings";
    pub const ATTACHED_TEMPLATE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/attachedTemplate";
    pub const THEME: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
    pub const FOOTNOTES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes";
    pub const ENDNOTES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes";
    pub const COMMENTS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
    pub const COMMENTS_EX: &str = "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";
    pub const COMMENTS_IDS: &str = "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds";
    pub const COMMENTS_EXTENSIBLE: &str = "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible";
    pub const HEADER: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
    pub const FOOTER: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
    pub const IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
    pub const HYPERLINK: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
    pub const VBA_PROJECT: &str = "http://schemas.microsoft.com/office/2006/relationships/vbaProject";
    pub const CHART: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart";
    pub const DIAGRAM_DATA: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData";
    pub const DIAGRAM_DRAWING: &str = "http://schemas.microsoft.com/office/2007/relationships/diagramDrawing";
    pub const DIAGRAM_LAYOUT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramLayout";
    pub const DIAGRAM_QUICK_STYLE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramQuickStyle";
    pub const DIAGRAM_COLORS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramColors";
    pub const CUSTOM_XML: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
}

/// `Document::passthrough` key for a macro project, kept as opaque bytes (never parsed or run).
pub const VBA_PROJECT_PART: &str = "word/vbaProject.bin";
/// `Document::passthrough` key listing the parts the project relates to (VBA data, signatures…),
/// one `relationship type \t part name \t content type` line each; each part's bytes are kept
/// under its part name. Not a package part: Doc4Me's own bookkeeping.
pub const VBA_RELATED: &str = "doc4me:vbaProject.related";
/// Most parts we carry along with a macro project.
pub const MAX_VBA_RELATED: usize = 64;

/// `Document::passthrough` key listing the parts that objects read from a file refer to (charts,
/// SmartArt diagrams, OLE objects: see `doc4me_doc::graphic::Embedded`), with their content
/// types and own relationships, as [`EmbeddedManifest`] lines. Each part's bytes are kept under its
/// part name. Not a package part: Doc4Me's own bookkeeping.
pub const EMBEDDED_PARTS: &str = "doc4me:embedded.parts";
/// Custom XML roots kept by the package (P) or main document (D): `owner\ttype\tpart`.
/// Their opaque bytes and related parts use the same bounded manifest as embedded objects.
pub const CUSTOM_XML_ROOTS: &str = "doc4me:customXml.roots";
/// The original `word/settings.xml` when it holds settings the model doesn't (merged into on
/// save by `write::settings_merge`). Not a package part: Doc4Me's own bookkeeping.
pub const SETTINGS_ORIG: &str = "doc4me:settings.orig";
/// Most parts we carry along for a file's embedded objects.
pub const MAX_EMBEDDED_PARTS: usize = 4096;
/// Most relationships we keep for one such part.
pub const MAX_EMBEDDED_RELS: usize = 1024;
/// Longest markup we keep for one embedded object.
pub const MAX_EMBEDDED_XML: usize = 1024 * 1024;

/// One part an embedded object needs: its content type and its own relationships (internal
/// targets are part names).
#[derive(Clone, Debug, Default)]
pub struct EmbeddedPart {
    pub content_type: String,
    pub rels: Vec<Rel>,
}

/// The parts kept for embedded objects, by part name (see [`EMBEDDED_PARTS`]). Stored as text:
/// `P\tpart\tcontent type` per part, then `R\tpart\tid\ttype\ttarget\tI|E` per relationship of
/// that part (`E`: `target` is an external URL).
#[derive(Clone, Debug, Default)]
pub struct EmbeddedManifest {
    pub parts: BTreeMap<String, EmbeddedPart>,
}

/// Can `s` be a field of a manifest line?
pub fn manifest_field_ok(s: &str) -> bool {
    !s.is_empty() && !s.contains(['\t', '\r', '\n'])
}

impl EmbeddedManifest {
    pub fn parse(bytes: &[u8]) -> EmbeddedManifest {
        let mut m = EmbeddedManifest::default();
        for line in String::from_utf8_lossy(bytes).lines() {
            let f: Vec<&str> = line.split('\t').collect();
            match f.as_slice() {
                ["P", part, ct] if m.parts.len() < MAX_EMBEDDED_PARTS => {
                    m.parts.entry((*part).to_string()).or_insert_with(|| EmbeddedPart { content_type: (*ct).to_string(), rels: Vec::new() });
                }
                ["R", part, id, kind, target, mode] => {
                    if let Some(p) = m.parts.get_mut(*part)
                        && p.rels.len() < MAX_EMBEDDED_RELS
                        && !p.rels.iter().any(|r| r.id == *id)
                    {
                        p.rels.push(Rel { id: (*id).to_string(), kind: (*kind).to_string(), target: (*target).to_string(), external: *mode == "E" });
                    }
                }
                _ => {}
            }
        }
        m
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut s = String::new();
        for (part, p) in &self.parts {
            if !manifest_field_ok(part) || !manifest_field_ok(&p.content_type) {
                continue;
            }
            s.push_str(&format!("P\t{part}\t{}\n", p.content_type));
        }
        for (part, p) in &self.parts {
            for r in p.rels.iter().filter(|r| [&r.id, &r.kind, &r.target].iter().all(|f| manifest_field_ok(f))) {
                s.push_str(&format!("R\t{part}\t{}\t{}\t{}\t{}\n", r.id, r.kind, r.target, if r.external { "E" } else { "I" }));
            }
        }
        s.into_bytes()
    }
}

/// Does relationship type `t` end with `suffix` (ignoring the transitional/strict prefix)?
pub fn rel_is(t: &str, full: &str) -> bool {
    let tail = |s: &str| s.rsplit('/').next().unwrap_or(s).to_string();
    t == full || tail(t) == tail(full)
}

/// The decompressed parts of a package (bounded).
pub struct Package {
    files: BTreeMap<String, Vec<u8>>,
    /// Lowercased part name → its stored spelling: part names are case-insensitive (ECMA-376
    /// Part 2 §6.2.2.3), so every spelling is one part.
    lower: HashMap<String, String>,
    /// What opening the package and reading its parts had to skip (Word's Show Repairs):
    /// (message with `{what}`, what).
    repairs: std::sync::Mutex<Vec<(&'static str, String)>>,
}

/// Show Repairs messages (`{what}` is the part or entry). Fixed English, translated by the front end.
pub const REPAIR_ZIP_ENTRY: &str = "An unreadable entry in the file was skipped: {what}";
pub const REPAIR_PART: &str = "An unreadable part was left out: {what}";
pub const REPAIR_RELS: &str = "Unreadable relationships were ignored: {what}";
pub const REPAIR_EMBEDDED: &str = "A missing part of an embedded object was left out: {what}";

impl Package {
    pub fn open(bytes: &[u8]) -> Result<Package, DocxError> {
        Package::open_limited(bytes, MAX_PART, MAX_TOTAL)
    }

    pub fn open_limited(bytes: &[u8], max_part: u64, max_total: u64) -> Result<Package, DocxError> {
        // Before `zip` parses every entry the end record declares (#3660).
        doc4me_doc::zip_guard::check(bytes, MAX_ENTRIES as u64).map_err(DocxError::Limit)?;
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| DocxError::Zip(e.to_string()))?;
        if zip.len() > MAX_ENTRIES {
            return Err(DocxError::Limit(format!("{} zip entries", zip.len())));
        }
        let mut files = BTreeMap::new();
        let mut repairs = Vec::new();
        let mut total: u64 = 0;
        for i in 0..zip.len() {
            let mut f = match zip.by_index(i) {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("docx: skipping unreadable zip entry {i}: {e}");
                    repairs.push((REPAIR_ZIP_ENTRY, format!("#{}", i + 1)));
                    continue;
                }
            };
            if f.is_dir() {
                continue;
            }
            let name = match f.name() {
                Ok(n) => n.trim_start_matches('/').to_string(),
                Err(e) => {
                    log::warn!("docx: skipping zip entry {i} with an unreadable name: {e}");
                    repairs.push((REPAIR_ZIP_ENTRY, format!("#{}", i + 1)));
                    continue;
                }
            };
            if f.size() > max_part {
                return Err(DocxError::Limit(format!("part {name} declares {} bytes", f.size())));
            }
            let room = max_part.min(max_total.saturating_sub(total));
            let mut buf = Vec::with_capacity(f.size().min(16 * 1024 * 1024) as usize);
            let read = (&mut f).take(room + 1).read_to_end(&mut buf);
            // Charged whether or not the read succeeds: an entry inflates in full before its CRC
            // is checked, so failed entries would otherwise inflate without limit (#3723).
            total = total.saturating_add(buf.len() as u64);
            if buf.len() as u64 > room {
                return Err(DocxError::Limit(format!("part {name} decompresses past the size limit")));
            }
            if let Err(e) = read {
                // A corrupt entry: keep going unless it's a part we can't do without.
                log::warn!("docx: zip entry {name} unreadable: {e}");
                repairs.push((REPAIR_ZIP_ENTRY, name));
                continue;
            }
            files.insert(name, buf);
        }
        Ok(Package::new(files, repairs))
    }

    /// A package from parts already in memory (a Flat OPC file, a converted document), by part
    /// name without the leading `/`.
    pub fn from_parts(files: BTreeMap<String, Vec<u8>>) -> Package {
        Package::new(files, Vec::new())
    }

    fn new(mut files: BTreeMap<String, Vec<u8>>, repairs: Vec<(&'static str, String)>) -> Package {
        let mut lower: HashMap<String, String> = HashMap::new();
        let mut collides = false;
        for name in files.keys() {
            match lower.entry(name.to_ascii_lowercase()) {
                Entry::Occupied(_) => collides = true,
                Entry::Vacant(v) => {
                    v.insert(name.clone());
                }
            }
        }
        if collides {
            // Entries whose names differ only in case are one part, so every spelling of the name
            // reaches the same stored entry and the part is kept, cached and written once. The
            // spelling kept is the one a relationship names, so the part the document points at is
            // the one read; the first in byte order only when no relationship names any of them.
            let named = rel_targets(&files);
            for name in files.keys().rev().filter(|n| named.contains(n.as_str())) {
                lower.insert(name.to_ascii_lowercase(), name.clone());
            }
            let before = files.len();
            files.retain(|name, _| lower.get(&name.to_ascii_lowercase()) == Some(name));
            let dropped = before.saturating_sub(files.len());
            log::warn!("docx: {dropped} zip entries are other spellings of parts already read; they are left out");
        }
        Package { files, lower, repairs: std::sync::Mutex::new(repairs) }
    }

    /// The stored spelling of part `name` (leading `/` ignored; case-insensitive).
    pub fn canonical(&self, name: &str) -> Option<&str> {
        self.lower.get(&name.trim_start_matches('/').to_ascii_lowercase()).map(String::as_str)
    }

    /// Note something reading had to skip (bounded by [`doc4me_doc::MAX_REPAIRS`]).
    pub fn note_repair(&self, message: &'static str, what: impl Into<String>) {
        if let Ok(mut r) = self.repairs.lock()
            && r.len() < doc4me_doc::MAX_REPAIRS
        {
            r.push((message, what.into()));
        }
    }

    /// The repairs noted so far.
    pub fn repairs(&self) -> Vec<(&'static str, String)> {
        self.repairs.lock().map(|r| r.clone()).unwrap_or_default()
    }

    /// Part bytes by name (leading `/` ignored; case-insensitive fallback).
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        let n = name.trim_start_matches('/');
        if let Some(v) = self.files.get(n) {
            return Some(v);
        }
        // An index lookup, not a scan of every part for every miss.
        self.canonical(n).and_then(|c| self.files.get(c)).map(Vec::as_slice)
    }

    pub fn xml(&self, name: &str) -> Result<Option<El>, DocxError> {
        match self.get(name) {
            Some(b) => xml::parse(b).map(Some),
            None => Ok(None),
        }
    }

    /// Relationships of `part` (from `dir/_rels/name.rels`).
    pub fn rels(&self, part: &str) -> Rels {
        let (dir, file) = split_dir(part);
        let path = if dir.is_empty() { format!("_rels/{file}.rels") } else { format!("{dir}/_rels/{file}.rels") };
        let mut rels = Rels::default();
        let root = match self.xml(&path) {
            Ok(Some(root)) => root,
            Ok(None) => return rels,
            Err(e) => {
                log::warn!("docx: ignoring unreadable relationships {path}: {e}");
                self.note_repair(REPAIR_RELS, path);
                return rels;
            }
        };
        for r in root.els().filter(|e| e.local() == "Relationship") {
            let (Some(id), Some(target)) = (r.attr("Id"), r.attr("Target")) else { continue };
            let external = r.attr("TargetMode").is_some_and(|m| m.eq_ignore_ascii_case("External"));
            // Internal targets name the part as stored, so a part reached under several spellings
            // is kept, cached and written once.
            let resolved = if external {
                target.to_string()
            } else {
                let path = resolve(dir, target);
                self.canonical(&path).map_or(path, str::to_string)
            };
            rels.list.push(Rel { id: id.to_string(), kind: r.attr("Type").unwrap_or("").to_string(), target: resolved, external });
        }
        rels
    }
}

/// `[Content_Types].xml`: Override by part name, then Default by extension (both case-insensitive).
#[derive(Default)]
pub struct ContentTypes {
    overrides: HashMap<String, String>,
    defaults: HashMap<String, String>,
}

impl ContentTypes {
    pub fn read(pkg: &Package) -> ContentTypes {
        let mut ct = ContentTypes::default();
        let root = match pkg.xml("[Content_Types].xml") {
            Ok(Some(root)) => root,
            Ok(None) => return ct,
            Err(e) => {
                log::warn!("docx: ignoring unreadable [Content_Types].xml: {e}");
                pkg.note_repair(REPAIR_PART, "[Content_Types].xml");
                return ct;
            }
        };
        for e in root.els() {
            let (Some(key), Some(t)) = (e.attr("PartName").or_else(|| e.attr("Extension")), e.attr("ContentType")) else { continue };
            let map = if e.local() == "Override" { &mut ct.overrides } else { &mut ct.defaults };
            map.entry(key.trim_start_matches('/').to_ascii_lowercase()).or_insert_with(|| t.to_string());
        }
        ct
    }

    /// Content type of `part` (a package path without the leading `/`).
    pub fn of(&self, part: &str) -> Option<&str> {
        let part = part.trim_start_matches('/').to_ascii_lowercase();
        if let Some(t) = self.overrides.get(&part) {
            return Some(t);
        }
        let ext = part.rsplit_once('.').map(|(_, e)| e)?;
        self.defaults.get(ext).map(String::as_str)
    }
}

#[derive(Clone, Debug)]
pub struct Rel {
    pub id: String,
    pub kind: String,
    /// Package path (internal) or URL (external).
    pub target: String,
    pub external: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Rels {
    pub list: Vec<Rel>,
}

impl Rels {
    pub fn by_id(&self, id: &str) -> Option<&Rel> {
        self.list.iter().find(|r| r.id == id)
    }
    pub fn by_type(&self, full: &str) -> Option<&Rel> {
        self.list.iter().find(|r| rel_is(&r.kind, full))
    }
}

fn split_dir(part: &str) -> (&str, &str) {
    let p = part.trim_start_matches('/');
    match p.rfind('/') {
        Some(i) => (p.get(..i).unwrap_or(""), p.get(i + 1..).unwrap_or(p)),
        None => ("", p),
    }
}

/// Every internal part name the relationship parts in `files` target, spelled as they spell it.
fn rel_targets(files: &BTreeMap<String, Vec<u8>>) -> HashSet<String> {
    let mut named = HashSet::new();
    for (name, bytes) in files {
        // `dir/_rels/file.rels` relates `dir/file`, so its targets resolve against `dir`.
        let lower = name.to_ascii_lowercase();
        let Some(at) = lower.rfind("_rels/").filter(|_| lower.ends_with(".rels")) else { continue };
        let dir = name.get(..at).unwrap_or("").trim_end_matches('/');
        let Ok(root) = xml::parse(bytes) else { continue };
        for r in root.els().filter(|e| e.local() == "Relationship") {
            let external = r.attr("TargetMode").is_some_and(|m| m.eq_ignore_ascii_case("External"));
            if let (Some(target), false) = (r.attr("Target"), external) {
                named.insert(resolve(dir, target));
            }
        }
    }
    named
}

/// Resolve a relative target against a source directory into a package path.
pub fn resolve(dir: &str, target: &str) -> String {
    let target = target.split('#').next().unwrap_or(target);
    let mut parts: Vec<&str> = if target.starts_with('/') { Vec::new() } else { dir.split('/').filter(|s| !s.is_empty()).collect() };
    for seg in target.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// The relative reference from part `from` to part `to` (both package paths), as a
/// relationship target: `word/document.xml` → `word/charts/chart1.xml` is `charts/chart1.xml`.
pub fn relative(from: &str, to: &str) -> String {
    let from_dir: Vec<&str> = from.split('/').filter(|s| !s.is_empty()).collect();
    let from_dir = from_dir.get(..from_dir.len().saturating_sub(1)).unwrap_or(&[]);
    let to: Vec<&str> = to.split('/').filter(|s| !s.is_empty()).collect();
    let to_dir = to.get(..to.len().saturating_sub(1)).unwrap_or(&[]);
    let common = from_dir.iter().zip(to_dir).take_while(|(a, b)| a == b).count();
    let mut out: Vec<&str> = vec![".."; from_dir.len() - common];
    out.extend(to.get(common..).unwrap_or(&[]));
    let s = out.join("/");
    // A relative reference whose first segment has a colon would read as a URI scheme.
    if s.split('/').next().is_some_and(|seg| seg.contains(':')) { format!("./{s}") } else { s }
}

/// Build a zip from (name, bytes) entries.
pub fn zip_entries(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, DocxError> {
    let mut zw = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).compression_level(Some(6));
    let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in entries {
        // Already-compressed media gain nothing from deflate.
        let lower = name.to_ascii_lowercase();
        let o = if [".png", ".jpg", ".jpeg", ".gif", ".webp"].iter().any(|e| lower.ends_with(e)) { stored } else { opts };
        zw.start_file(name.as_str(), o).map_err(|e| DocxError::Zip(e.to_string()))?;
        zw.write_all(bytes).map_err(|e| DocxError::Zip(e.to_string()))?;
    }
    let c = zw.finish().map_err(|e| DocxError::Zip(e.to_string()))?;
    Ok(c.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zip_of(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let e: Vec<(String, Vec<u8>)> = entries.iter().map(|(n, b)| (n.to_string(), b.clone())).collect();
        zip_entries(&e).unwrap()
    }

    #[test]
    fn case_colliding_entries_are_one_part() {
        let z = zip_of(&[("word/media/Image.png", b"upper".to_vec()), ("word/media/image.png", b"lower".to_vec())]);
        let p = Package::open(&z).unwrap();
        for spelling in ["word/media/Image.png", "word/media/image.png", "/WORD/MEDIA/IMAGE.PNG"] {
            assert_eq!(p.canonical(spelling), Some("word/media/Image.png"), "{spelling}");
            assert_eq!(p.get(spelling), Some(&b"upper"[..]), "{spelling}");
        }
        assert_eq!(p.files.len(), 1);
    }

    #[test]
    fn the_spelling_a_relationship_names_is_kept() {
        // The part a relationship points at is the one read; every other spelling reaches it.
        let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="image" Target="media/image.png"/><Relationship Id="rId2" Type="image" Target="MEDIA/IMAGE.PNG"/></Relationships>"#;
        let z = zip_of(&[
            ("word/_rels/document.xml.rels", rels.as_bytes().to_vec()),
            ("word/media/Image.png", b"upper".to_vec()),
            ("word/media/image.png", b"lower".to_vec()),
            ("word/media/Other.png", b"first".to_vec()),
            ("word/media/other.png", b"second".to_vec()),
        ]);
        let p = Package::open(&z).unwrap();
        assert_eq!(p.canonical("word/media/IMAGE.png"), Some("word/media/image.png"));
        assert_eq!(p.get("word/media/Image.png"), Some(&b"lower"[..]));
        assert_eq!(p.get("word/media/OTHER.PNG"), Some(&b"first"[..]), "no relationship names it: first in byte order");
        assert_eq!(p.files.len(), 3);
        // Both relationships resolve to the one stored spelling, so the picture is kept once.
        let targets: Vec<String> = p.rels("word/document.xml").list.into_iter().map(|r| r.target).collect();
        assert_eq!(targets, ["word/media/image.png", "word/media/image.png"]);
    }

    #[test]
    fn a_package_declaring_too_many_entries_is_refused_before_zip_reads_them() {
        // #3660: a ZIP64 end record declaring more than MAX_ENTRIES.
        let bomb = doc4me_doc::zip_guard::zip64_entry_count_sample(MAX_ENTRIES as u64 + 1);
        assert!(matches!(Package::open(&bomb), Err(DocxError::Limit(e)) if e.contains("declares")));
        // A real package still opens.
        let z = zip_of(&[(
            "docProps/core.xml",
            br#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Plan</dc:title></cp:coreProperties>"#.to_vec(),
        )]);
        assert!(Package::open(&z).is_ok());
    }

    #[test]
    fn decompression_bomb_is_rejected() {
        let z = zip_of(&[("word/document.xml", vec![b' '; 2_000_000])]);
        assert!(z.len() < 100_000);
        assert!(matches!(Package::open_limited(&z, 1_000_000, 10_000_000), Err(DocxError::Limit(_))));
        assert!(matches!(Package::open_limited(&z, 10_000_000, 1_000_000), Err(DocxError::Limit(_))));
        assert!(Package::open_limited(&z, 10_000_000, 10_000_000).is_ok());
    }

    #[test]
    fn entries_that_fail_their_check_still_count_against_the_total() {
        // Each entry inflates in full before its CRC is checked; a failed one was dropped without
        // being charged, so any number of them inflated (#3723).
        let names: Vec<String> = (0..20).map(|i| format!("p{i}.bin")).collect();
        let entries: Vec<(&str, Vec<u8>)> = names.iter().map(|n| (n.as_str(), vec![0u8; 1_000_000])).collect();
        let mut z = zip_of(&entries);
        let mut patched = 0;
        for sig in [[0x50u8, 0x4B, 0x01, 0x02], [0x50, 0x4B, 0x03, 0x04]] {
            let crc_at = if sig[2] == 1 { 16 } else { 14 };
            let found: Vec<usize> = z.windows(4).enumerate().filter(|(_, w)| *w == sig).map(|(i, _)| i).collect();
            for pos in found {
                z[pos + crc_at] ^= 0xFF;
                patched += 1;
            }
        }
        assert_eq!(patched, 40);
        assert!(matches!(Package::open_limited(&z, 2_000_000, 4_000_000), Err(DocxError::Limit(_))));
    }

    #[test]
    fn lying_declared_size_is_rejected_or_bounded() {
        let mut z = zip_of(&[("a.xml", b"<a/>".to_vec())]);
        // Patch the uncompressed size in the central directory and local header to ~4 GB.
        let mut patched = 0;
        for sig in [[0x50u8, 0x4B, 0x01, 0x02], [0x50, 0x4B, 0x03, 0x04]] {
            if let Some(pos) = z.windows(4).position(|w| w == sig) {
                let off = if sig[2] == 1 { pos + 24 } else { pos + 22 };
                z[off..off + 4].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
                patched += 1;
            }
        }
        assert_eq!(patched, 2);
        match Package::open(&z) {
            Err(_) => {}
            Ok(p) => assert!(p.get("a.xml").is_none_or(|b| b.len() < 1024)),
        }
    }

    #[test]
    fn resolves_targets() {
        assert_eq!(resolve("word", "media/image1.png"), "word/media/image1.png");
        assert_eq!(resolve("word", "../customXml/item1.xml"), "customXml/item1.xml");
        assert_eq!(resolve("word", "/word/styles.xml"), "word/styles.xml");
        assert_eq!(resolve("", "word/document.xml"), "word/document.xml");
        assert_eq!(resolve("word", "../../../x"), "x");
    }

    #[test]
    fn relative_targets_resolve_back() {
        for (from, to, want) in [
            ("word/document.xml", "word/charts/chart1.xml", "charts/chart1.xml"),
            ("word/charts/chart1.xml", "word/embeddings/book.xlsx", "../embeddings/book.xlsx"),
            ("word/charts/chart1.xml", "word/charts/colors1.xml", "colors1.xml"),
            ("word/document.xml", "customXml/item1.xml", "../customXml/item1.xml"),
            ("word/document.xml", "word/a:b.bin", "./a:b.bin"),
        ] {
            let r = relative(from, to);
            assert_eq!(r, want);
            assert_eq!(resolve(from.rsplit_once('/').map_or("", |(d, _)| d), &r), to);
        }
    }

    #[test]
    fn embedded_manifest_round_trips_and_ignores_junk() {
        let mut m = EmbeddedManifest::default();
        let rel = Rel { id: "rId1".into(), kind: "k".into(), target: "word/embeddings/x.xlsx".into(), external: false };
        let url = Rel { id: "rId2".into(), kind: "k".into(), target: "https://example.com/x".into(), external: true };
        m.parts.insert("word/charts/chart1.xml".into(), EmbeddedPart { content_type: "ct".into(), rels: vec![rel, url] });
        let back = EmbeddedManifest::parse(&m.to_bytes());
        let p = &back.parts["word/charts/chart1.xml"];
        assert_eq!((p.content_type.as_str(), p.rels.len(), p.rels[1].external), ("ct", 2, true));
        let junk = EmbeddedManifest::parse(b"R\tnope\trId1\tk\tt\tI\nP\tonly-two\n\t\t\n\xff\xfe");
        assert!(junk.parts.is_empty());
    }
}
