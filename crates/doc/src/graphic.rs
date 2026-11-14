//! Charts and SmartArt diagrams: drawings kept as neutral items ready to draw. Charts and SmartArt
//! Doc4Me can edit also keep their model ([`crate::chart::ChartSpec`],
//! [`crate::smart_art::SmartArtSpec`]); the rest are shown, not edited.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::para::ShapeKind;
use crate::props::Rgb;

/// A chart or a SmartArt diagram, as items ready to draw (and, for an editable chart or SmartArt
/// graphic, its model).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Graphic {
    pub kind: GraphicKind,
    /// What to draw, in points from the object's top-left corner.
    #[serde(default)]
    pub items: Vec<GraphicItem>,
    /// The size (points) the items were built for. Drawn scaled to the object's rectangle when
    /// that differs (layout shrinks oversized inline objects); 0 means the rectangle's own size.
    #[serde(default)]
    pub w: f32,
    #[serde(default)]
    pub h: f32,
    /// Where the graphic was read from, so saving writes the chart or diagram itself back (not
    /// only these items). `None` for graphics made some other way.
    #[serde(skip)]
    pub source: Option<std::sync::Arc<Embedded>>,
    /// A SmartArt graphic Doc4Me can edit (one it made, or one read back exactly as it wrote
    /// it): saving writes the diagram parts from it, and the items are drawn from it. See
    /// [`crate::smart_art`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smart_art: Option<std::sync::Arc<crate::smart_art::SmartArtSpec>>,
    /// A chart Doc4Me can edit (one it made, or one read back that holds nothing more): saving
    /// writes the chart part from it, and the items are drawn from it. See [`crate::chart`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart: Option<std::sync::Arc<crate::chart::ChartSpec>>,
}

/// What an object read from a file needs to be written back as it was: its own markup in the
/// source format and the package parts that markup refers to. Never edited; Doc4Me only moves
/// and resizes the object around it. The parts themselves (bytes, content type and their own
/// relationships) are kept in [`crate::Document::passthrough`] under their part names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Embedded {
    /// The object's markup as a standalone XML element with its namespace declarations (DOCX:
    /// the `a:graphic` of a chart or SmartArt diagram, the `w:object` of an OLE object).
    pub xml: String,
    /// The relationships it uses, by the ids in `xml` (or in its parts, for a SmartArt drawing).
    pub refs: Vec<EmbeddedRef>,
}

/// One relationship an [`Embedded`] object uses.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmbeddedRef {
    /// The relationship id the markup uses.
    pub id: String,
    /// Relationship type (a URI).
    pub kind: String,
    /// The part name (a `Document::passthrough` key), or the URL of an external target.
    pub target: String,
    pub external: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GraphicKind {
    #[default]
    Chart,
    Diagram,
}

impl GraphicKind {
    /// What the graphic is, in words: the alt text of one that has none of its own.
    pub fn noun(self) -> &'static str {
        match self {
            GraphicKind::Chart => "chart",
            GraphicKind::Diagram => "diagram",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GraphicItem {
    /// A preset shape filling `rect` ([x, y, w, h]).
    Shape {
        rect: [f32; 4],
        kind: ShapeKind,
        fill: Option<Rgb>,
        stroke: Option<Rgb>,
        #[serde(default)]
        stroke_width: f32,
    },
    /// A path: polygons, pie slices, line series.
    Path {
        segs: Vec<PathSeg>,
        fill: Option<Rgb>,
        stroke: Option<Rgb>,
        #[serde(default)]
        stroke_width: f32,
    },
    /// One line of text in `rect`, aligned horizontally by `align`, centred vertically.
    Text {
        rect: [f32; 4],
        text: String,
        size: f32,
        color: Rgb,
        #[serde(default)]
        bold: bool,
        #[serde(default)]
        align: TextAlign,
        #[serde(default)]
        font: Option<String>,
    },
    /// A picture (`Document::media` key) stretched into `rect`.
    Image { rect: [f32; 4], media: String },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PathSeg {
    Move(f32, f32),
    Line(f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

/// Internal part-graph metadata shared by the document clipboard and DOCX I/O.
pub const EMBEDDED_PARTS: &str = "doc4me:embedded.parts";
pub const MAX_EMBEDDED_PARTS: usize = 4096;
pub const MAX_EMBEDDED_RELS: usize = 1024;

/// One part an embedded object needs: its content type and its own relationships (internal
/// targets are part names).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmbeddedPart {
    pub content_type: String,
    pub rels: Vec<EmbeddedRef>,
}

/// The parts kept for embedded objects, by part name (see [`EMBEDDED_PARTS`]). Stored as text:
/// `P\tpart\tcontent type` per part, then `R\tpart\tid\ttype\ttarget\tI|E` per relationship of
/// that part (`E`: `target` is an external URL).
#[derive(Clone, Debug, Default, PartialEq)]
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
                        p.rels.push(EmbeddedRef {
                            id: (*id).to_string(),
                            kind: (*kind).to_string(),
                            target: (*target).to_string(),
                            external: *mode == "E",
                        });
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
