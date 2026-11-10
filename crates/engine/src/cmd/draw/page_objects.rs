//! Per-call object lookups for drawing-order page walks, which can revisit a paragraph's
//! offsets out of text order. Scan only as far as the furthest requested object, once.

use doc4me_doc::para::OBJ;
use doc4me_doc::{Document, InlineObject, Path, Pos, StoryRef};
use std::collections::HashMap;

#[derive(Default)]
struct ParagraphObjects<'a> {
    scanned: usize,
    ordinal: usize,
    objects: Vec<(usize, &'a InlineObject)>,
}

pub(super) struct PageObjects<'a> {
    doc: &'a Document,
    paragraphs: HashMap<(StoryRef, Path), ParagraphObjects<'a>>,
}

impl<'a> PageObjects<'a> {
    pub(super) fn new(doc: &'a Document) -> Self {
        Self { doc, paragraphs: HashMap::new() }
    }

    pub(super) fn get(&mut self, pos: &Pos) -> Option<&'a InlineObject> {
        let p = self.doc.para_at(pos)?;
        if !p.text.get(pos.off..)?.starts_with(OBJ) {
            return None;
        }
        let end = pos.off.checked_add(OBJ.len_utf8())?;
        let cached = self.paragraphs.entry((pos.story, pos.path.clone())).or_default();
        if end > cached.scanned {
            // Start after the last scanned marker, not at the paragraph's start. In particular,
            // a lone object near the start never scans a long text suffix it doesn't need.
            for (off, c) in p.text.get(cached.scanned..end)?.char_indices() {
                if c == OBJ {
                    if let Some(object) = p.objects.get(cached.ordinal) {
                        cached.objects.push((cached.scanned + off, object));
                    }
                    cached.ordinal += 1;
                }
            }
            cached.scanned = end;
        }
        let i = cached.objects.binary_search_by_key(&pos.off, |(off, _)| *off).ok()?;
        cached.objects.get(i).map(|(_, object)| *object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc4me_doc::{CharProps, Paragraph, para_block};

    #[test]
    fn lookups_out_of_order_match_object_at() {
        // Multi-byte text around the markers, offsets visited out of text order and repeated,
        // and offsets that aren't markers: every answer is the paragraph's own `object_at`.
        let mut d = Document::new();
        let mut p = Paragraph::new();
        p.text = format!("é{OBJ}{OBJ}x{OBJ}漢{OBJ}");
        p.runs = vec![doc4me_doc::para::Run { len: p.text.len(), props: CharProps::default() }];
        p.objects = (0..4).map(|i| InlineObject::BookmarkStart { name: format!("b{i}") }).collect();
        d.body = vec![para_block(p)];
        let para = d.para(StoryRef::Body, &Path::top(0)).unwrap();
        let mut objects = PageObjects::new(&d);
        let offs: Vec<usize> = (0..=para.text.len()).rev().chain(0..=para.text.len()).chain([usize::MAX]).collect();
        for off in offs {
            let pos = Pos::new(StoryRef::Body, Path::top(0), off);
            assert_eq!(objects.get(&pos), para.object_at(off), "offset {off}");
        }
    }
}
