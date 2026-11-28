//! The faces laid-out text is drawn in, with the fonts the text asks for and the glyphs drawn
//! ([`DocLayout::used_faces`]), for embedding fonts in a saved document.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use doc4me_fonts::FaceRef;

use crate::para::ClKind;
use crate::{DocLayout, Placed};

/// Most faces [`DocLayout::used_faces`] returns.
pub const MAX_USED_FACES: usize = 256;
/// Most font names recorded per face.
const MAX_NAMES: usize = 16;

/// A face text is drawn in.
#[derive(Clone, Debug)]
pub struct UsedFace {
    pub face: FaceRef,
    /// The fonts the text drawn in it asks for (its runs' Latin, complex-script and East Asian
    /// fonts), as the document names them. A face none of them names is a substitute or a
    /// fallback for characters the font lacks.
    pub fonts: Vec<String>,
    /// The glyphs drawn.
    pub gids: BTreeSet<u32>,
}

impl DocLayout {
    /// The faces the text is drawn in, over every page's body, header, footer, notes, text boxes
    /// and table cells (the text this view lays out), in the order first met, at most
    /// [`MAX_USED_FACES`].
    pub fn used_faces(&self) -> Vec<UsedFace> {
        let mut w = Walk { paras: HashSet::new(), at: HashMap::new(), out: Vec::new() };
        for p in &self.pages {
            for items in [&p.items, &p.header, &p.footer] {
                w.walk(items);
            }
        }
        w.out
    }
}

struct Walk {
    paras: HashSet<usize>,
    /// Face id → index in `out`.
    at: HashMap<u32, usize>,
    out: Vec<UsedFace>,
}

impl Walk {
    fn walk(&mut self, items: &[Placed]) {
        for it in items {
            match it {
                Placed::Lines { para, .. } if self.paras.insert(Arc::as_ptr(para) as usize) => {
                    for c in para.clusters.iter().filter(|c| matches!(c.kind, ClKind::Text | ClKind::Space)) {
                        let Some(style) = para.styles.get(c.style as usize) else { continue };
                        let i = match self.at.get(&style.face.id()) {
                            Some(&i) => i,
                            None if self.out.len() < MAX_USED_FACES => {
                                self.at.insert(style.face.id(), self.out.len());
                                self.out.push(UsedFace { face: style.face, fonts: Vec::new(), gids: BTreeSet::new() });
                                self.out.len() - 1
                            }
                            None => continue,
                        };
                        let Some(used) = self.out.get_mut(i) else { continue };
                        let rc = &style.rc;
                        for name in std::iter::once(&rc.font).chain(std::iter::once(&rc.font_cs)).chain(rc.font_ea.iter()) {
                            let name = name.trim();
                            if !name.is_empty() && used.fonts.len() < MAX_NAMES && !used.fonts.iter().any(|f| f.eq_ignore_ascii_case(name)) {
                                used.fonts.push(name.to_string());
                            }
                        }
                        let glyphs = para.glyphs.get(c.g0 as usize..c.g1 as usize).unwrap_or_default();
                        used.gids.extend(glyphs.iter().map(|g| g.gid));
                    }
                }
                Placed::Clip { items, .. } => self.walk(items),
                _ => {}
            }
        }
    }
}
