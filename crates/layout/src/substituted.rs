//! Fonts requested by shown text that aren't installed, and the faces drawn instead.

use std::collections::HashSet;
use std::sync::Arc;

use doc4me_fonts::FaceRef;

use crate::{DocLayout, Placed};

/// Bound the list when a file names many missing fonts.
pub const MAX_SUBSTITUTED: usize = 1024;

impl DocLayout {
    /// Fonts requested by paragraphs in the current layout, paired with the actual drawing face.
    /// Body, headers, footers, notes, text boxes and table cells are included once each.
    pub fn substituted_fonts(&self) -> Vec<(String, FaceRef)> {
        let mut walk = Walk { paras: HashSet::new(), pairs: HashSet::new(), out: Vec::new() };
        for page in &self.pages {
            for items in [&page.items, &page.header, &page.footer] {
                walk.walk(items);
            }
        }
        walk.out
    }
}

struct Walk<'a> {
    paras: HashSet<usize>,
    pairs: HashSet<(&'a str, u32)>,
    out: Vec<(String, FaceRef)>,
}

impl<'a> Walk<'a> {
    fn walk(&mut self, items: &'a [Placed]) {
        for item in items {
            match item {
                Placed::Lines { para, .. } if self.paras.insert(Arc::as_ptr(para) as usize) => {
                    for (font, face) in &para.substituted {
                        if self.out.len() < MAX_SUBSTITUTED && self.pairs.insert((font.as_str(), face.id())) {
                            self.out.push((font.clone(), *face));
                        }
                    }
                }
                Placed::Clip { items, .. } => self.walk(items),
                _ => {}
            }
        }
    }
}
