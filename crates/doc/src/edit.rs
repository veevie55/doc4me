//! Range editing on a [`Document`]: insert text, split paragraphs, delete ranges across
//! paragraphs and tables, apply character/paragraph formatting, copy a range to a fragment and
//! paste a fragment.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::para::InlineObject;
use crate::props::{CharProps, ParaProps};
use crate::{Block, Blocks, DocError, Document, Paragraph, Part, PartKind, Path, Pos, Result, Table, para_block, table::MAX_TABLE_DEPTH};

/// A copied piece of a document: blocks, where the first and last paragraphs may be partial.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Fragment {
    pub blocks: Vec<Block>,
    /// Copies of the text box stories its shapes show, by the source document's part id. Pasting
    /// gives each box a new story made from these.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parts: BTreeMap<u32, Part>,
    /// Referenced picture bytes survive closing the source document. Shared until edited.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub media: BTreeMap<String, Arc<Vec<u8>>>,
    /// Original package graphs used by copied charts, diagrams and OLE objects.
    #[serde(skip)]
    pub passthrough: BTreeMap<String, Arc<Vec<u8>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub styles: Option<Arc<crate::styles::StyleSheet>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbering: Option<Arc<crate::numbering::Numbering>>,
}

impl Fragment {
    /// Conservative retained-content estimate for bounded clipboard history. The live clipboard
    /// may hold a larger item; the history must also charge shared images and object stories.
    pub fn retained_bytes(&self, limit: usize) -> usize {
        let mut bytes = self.media.values().chain(self.passthrough.values()).fold(0usize, |n, b| n.saturating_add(b.len()));
        if bytes > limit {
            return bytes;
        }
        for b in self.blocks.iter().chain(self.parts.values().flat_map(|p| p.blocks.iter().map(|b| &**b))) {
            bytes = bytes.saturating_add(block_cost(b));
            if bytes > limit {
                return bytes;
            }
        }
        bytes.saturating_add(serialized_len(&self.styles)).saturating_add(serialized_len(&self.numbering)).saturating_mul(2)
    }

    pub fn plain_text(&self) -> String {
        let mut d = Document::new();
        d.body = self.blocks.iter().cloned().map(Arc::new).collect();
        d.plain_text(crate::StoryRef::Body)
    }
    pub fn from_text(s: &str) -> Fragment {
        let lines: Vec<&str> = s.split('\n').collect();
        Fragment {
            blocks: lines.iter().map(|l| Block::Para(Paragraph::with_text(l.trim_end_matches('\r'), CharProps::default()))).collect(),
            ..Default::default()
        }
    }
}

fn order(a: &Pos, b: &Pos) -> (Pos, Pos) {
    if a <= b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) }
}

/// The blocks in `t`'s cells, row by row.
fn cell_blocks(t: &Table) -> impl DoubleEndedIterator<Item = &Block> {
    t.rows.iter().flat_map(|r| r.cells.iter()).flat_map(|c| c.blocks.iter()).map(|b| &**b)
}

/// How many tables deep `t` goes, itself included: 1 when its cells hold no table (without
/// recursion: a pasted fragment may be deep).
fn table_height(t: &Table) -> usize {
    let mut most = 1;
    let mut stack: Vec<(&Block, usize)> = cell_blocks(t).map(|b| (b, 1)).collect();
    while let Some((b, d)) = stack.pop() {
        if let Block::Table(inner) = b {
            let d = d.saturating_add(1);
            most = most.max(d);
            stack.extend(cell_blocks(inner).map(|cb| (cb, d)));
        }
    }
    most
}

/// `b` as blocks nesting at most `room` tables deep: a table that would go deeper is replaced by
/// its paragraphs (tables in its cells too, document order), as the DOCX reader does.
fn fit_tables(b: Block, room: usize, out: &mut Vec<Block>) {
    match b {
        Block::Table(t) if room == 0 => {
            let mut stack: Vec<&Block> = cell_blocks(&t).rev().collect();
            while let Some(cb) = stack.pop() {
                match cb {
                    Block::Para(p) => out.push(Block::Para(p.clone())),
                    Block::Table(inner) => stack.extend(cell_blocks(inner).rev()),
                }
            }
        }
        Block::Table(mut t) if table_height(&t) > room => {
            for cell in t.rows.iter_mut().flat_map(|r| r.cells.iter_mut()) {
                let mut kept = Vec::new();
                for cb in std::mem::take(&mut cell.blocks) {
                    fit_tables(Arc::unwrap_or_clone(cb), room.saturating_sub(1), &mut kept);
                }
                cell.blocks = kept.into_iter().map(Arc::new).collect();
            }
            out.push(Block::Table(t));
        }
        b => out.push(b),
    }
}

/// Nesting limit when following text boxes inside text boxes.
const MAX_BOX_DEPTH: usize = 8;
/// Most text box stories one paste may create (hostile documents can fan out).
const MAX_PASTED_BOXES: usize = 4096;
/// Most text box content (estimated bytes, see [`block_cost`]) one paste may adopt: thousands of
/// boxes can show the same large story, and each needs a story of its own.
const MAX_PASTED_BOX_BYTES: usize = 32 << 20;

/// What a paste may still spend on text box stories.
struct BoxBudget {
    stories: usize,
    bytes: usize,
    /// The source stories being copied, innermost last: a box inside one of them showing it
    /// again is a cycle.
    active: Vec<u32>,
    /// Each source story's cost ([`block_cost`]), worked out once per paste: a story too big for
    /// what is left is not walked again for every box that shows it.
    costs: BTreeMap<u32, usize>,
    /// Boxes left without a story because the paste's budget couldn't hold theirs.
    skipped: usize,
}

impl BoxBudget {
    fn new() -> BoxBudget {
        BoxBudget { stories: MAX_PASTED_BOXES, bytes: MAX_PASTED_BOX_BYTES, active: Vec::new(), costs: BTreeMap::new(), skipped: 0 }
    }
}

/// Rough size of a block's content: its text, runs and object slots, plus what each object
/// carries (group members, equations, kept XML …), estimated by its serialized length.
fn block_cost(b: &Block) -> usize {
    let mut cost = std::mem::size_of::<Block>();
    each_para(b, 0, &mut |p| {
        let items = p
            .runs
            .len()
            .saturating_mul(std::mem::size_of::<crate::para::Run>())
            .saturating_add(p.objects.len().saturating_mul(std::mem::size_of::<InlineObject>()));
        let mut objects = p.objects.iter().fold(0usize, |n, o| n.saturating_add(serialized_len(o)));
        // Original package markup is intentionally skipped by JSON serialization.
        let mut todo: Vec<_> = p.objects.iter().collect();
        while let Some(o) = todo.pop() {
            let embedded = match o {
                InlineObject::Image { ole, .. } => ole.as_deref(),
                InlineObject::Graphic { graphic, .. } => graphic.source.as_deref(),
                InlineObject::Group { children, .. } => {
                    todo.extend(children.iter().map(|c| &c.obj));
                    None
                }
                _ => None,
            };
            if let Some(e) = embedded {
                objects = objects.saturating_add(e.xml.len());
                for r in &e.refs {
                    objects = objects.saturating_add(r.id.len()).saturating_add(r.kind.len()).saturating_add(r.target.len());
                }
            }
        }
        cost = cost.saturating_add(p.text.len()).saturating_add(items).saturating_add(objects);
    });
    cost
}

/// The length of `v` as JSON, counted without building the text.
fn serialized_len<T: serde::Serialize>(v: &T) -> usize {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(b.len());
            if self.0 > MAX_PASTED_BOX_BYTES {
                return Err(std::io::Error::other("clipboard content exceeds the size budget"));
            }
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Count(0);
    if serde_json::to_writer(&mut c, v).is_err() { usize::MAX } else { c.0 }
}

/// Call `f` on every paragraph of a block, in document order, however deep its tables nest. The
/// walk keeps its own stack instead of recursing, so it is complete: a text box it doesn't reach
/// is shown nowhere, which pruning relies on (#1339). `_depth` is unused (callers keep passing it).
pub(crate) fn each_para<'a>(b: &'a Block, _depth: usize, f: &mut dyn FnMut(&'a Paragraph)) {
    // Most blocks are paragraphs, which need no stack.
    if let Block::Para(p) = b {
        return f(p);
    }
    let mut todo = vec![b];
    while let Some(b) = todo.pop() {
        match b {
            Block::Para(p) => f(p),
            Block::Table(t) => {
                // Last first, so they come off the stack in document order.
                let cells = t.rows.iter().rev().flat_map(|r| r.cells.iter().rev());
                todo.extend(cells.flat_map(|c| c.blocks.iter().rev()).map(|cb| &**cb));
            }
        }
    }
}

/// [`each_para`], changing the paragraphs.
pub(crate) fn each_para_mut(b: &mut Block, _depth: usize, f: &mut dyn FnMut(&mut Paragraph)) {
    if let Block::Para(p) = b {
        return f(p);
    }
    let mut todo = vec![b];
    while let Some(b) = todo.pop() {
        match b {
            Block::Para(p) => f(p),
            Block::Table(t) => {
                let cells = t.rows.iter_mut().rev().flat_map(|r| r.cells.iter_mut().rev());
                todo.extend(cells.flat_map(|c| c.blocks.iter_mut().rev()).map(Arc::make_mut));
            }
        }
    }
}

/// Whether a paragraph anywhere in `b` (also inside tables) uses paragraph or character style
/// `id` (tables to 16 levels deep). Table styles are the engine's `table.deleteStyle`'s job.
fn uses_style(b: &Block, id: &str, depth: usize) -> bool {
    let is = |s: &Option<String>| s.as_deref() == Some(id);
    match b {
        Block::Para(p) => is(&p.props.style) || is(&p.mark.style) || p.runs.iter().any(|r| is(&r.props.style)),
        Block::Table(t) if depth < 16 => {
            t.rows.iter().flat_map(|r| r.cells.iter()).flat_map(|c| c.blocks.iter()).any(|cb| uses_style(cb, id, depth + 1))
        }
        Block::Table(_) => false,
    }
}

/// Point every paragraph and run in `b` that uses style `from` at `to`; returns how many
/// paragraphs changed.
fn restyle_block(b: &mut Block, from: &str, to: &Option<String>, depth: usize) -> usize {
    let fix = |s: &mut Option<String>| {
        if s.as_deref() == Some(from) {
            s.clone_from(to);
            true
        } else {
            false
        }
    };
    match b {
        Block::Para(p) => {
            let mut changed = fix(&mut p.props.style);
            changed |= fix(&mut p.mark.style);
            for r in &mut p.runs {
                changed |= fix(&mut r.props.style);
            }
            if changed {
                p.normalize();
                p.touch();
            }
            usize::from(changed)
        }
        Block::Table(t) => {
            let mut n = 0;
            if depth < 16 {
                for cell in t.rows.iter_mut().flat_map(|r| r.cells.iter_mut()) {
                    for cb in &mut cell.blocks {
                        if uses_style(cb, from, depth + 1) {
                            n += restyle_block(Arc::make_mut(cb), from, to, depth + 1);
                        }
                    }
                }
            }
            n
        }
    }
}

/// Text box story ids of a paragraph's shapes.
fn box_ids(p: &Paragraph) -> impl Iterator<Item = u32> + '_ {
    p.objects.iter().flat_map(InlineObject::text_boxes)
}

fn referenced_parts(p: &Paragraph) -> impl Iterator<Item = u32> + '_ {
    box_ids(p).chain(p.objects.iter().filter_map(|o| match o {
        InlineObject::NoteRef { id, .. } => Some(*id),
        _ => None,
    }))
}

impl Document {
    /// Convert notes between footnotes and endnotes: footnotes (`foot`) become endnotes and
    /// endnotes (`end`) footnotes, both at once to swap them. Each note's kind changes, as do its
    /// reference marks wherever they are (the body, text boxes, the note's own mark) and the
    /// note styles (Footnote Text and Footnote Reference to their endnote twins and back).
    /// Returns how many notes changed.
    pub fn convert_notes(&mut self, foot: bool, end: bool) -> usize {
        use crate::para::NoteKind;
        let flips: std::collections::HashSet<u32> = self
            .parts
            .iter()
            .filter(|(_, p)| (foot && p.kind == PartKind::Footnote) || (end && p.kind == PartKind::Endnote))
            .map(|(id, _)| *id)
            .collect();
        if flips.is_empty() {
            return 0;
        }
        let swap = |s: &mut Option<String>| {
            let to = match s.as_deref() {
                Some("FootnoteText") => "EndnoteText",
                Some("EndnoteText") => "FootnoteText",
                Some("FootnoteReference") => "EndnoteReference",
                Some("EndnoteReference") => "FootnoteReference",
                _ => return,
            };
            *s = Some(to.to_string());
        };
        let refers = |p: &Paragraph| p.objects.iter().any(|o| matches!(o, InlineObject::NoteRef { id, .. } if flips.contains(id)));
        // In a paragraph: flip the converted notes' marks and their runs' reference style; in a
        // converted note's own story also the paragraph style.
        let convert = |p: &mut Paragraph, in_note: bool| {
            if !in_note && !refers(p) {
                return;
            }
            let offs = p.object_offsets();
            let mut at = Vec::new();
            for (k, o) in p.objects.iter_mut().enumerate() {
                if let InlineObject::NoteRef { kind, id, .. } = o
                    && flips.contains(id)
                {
                    *kind = if *kind == NoteKind::Footnote { NoteKind::Endnote } else { NoteKind::Footnote };
                    at.extend(offs.get(k).copied());
                }
            }
            // Reference offsets and run boundaries are ordered; do not restart at the first
            // reference for every run, including runs after the last reference.
            let mut at = at.into_iter().peekable();
            let mut start = 0usize;
            for r in &mut p.runs {
                let end = start.saturating_add(r.len);
                while at.peek().is_some_and(|&o| o < start) {
                    at.next();
                }
                if at.peek().is_some_and(|&o| o < end) {
                    swap(&mut r.props.style);
                }
                start = end;
            }
            if in_note {
                swap(&mut p.props.style);
                swap(&mut p.mark.style);
            }
            p.touch();
        };
        let touches = |b: &Block| {
            let mut any = false;
            each_para(b, 0, &mut |p| any |= refers(p));
            any
        };
        for b in self.body.iter_mut() {
            if touches(b) {
                each_para_mut(Arc::make_mut(b), 0, &mut |p| convert(p, false));
            }
        }
        for (id, part) in self.parts.iter_mut() {
            let own = flips.contains(id);
            if own {
                part.kind = if part.kind == PartKind::Footnote { PartKind::Endnote } else { PartKind::Footnote };
            }
            for b in part.blocks.iter_mut() {
                if own || touches(b) {
                    each_para_mut(Arc::make_mut(b), 0, &mut |p| convert(p, own));
                }
            }
        }
        flips.len()
    }

    /// Point every paragraph and run that uses paragraph or character style `from` at `to`
    /// (`None`: the default style), in every story, tables included. Returns how many paragraphs
    /// changed. (Tables' own table styles are retargeted by the engine's table style deletion.)
    pub fn restyle(&mut self, from: &str, to: Option<String>) -> usize {
        let stories: Vec<crate::StoryRef> =
            std::iter::once(crate::StoryRef::Body).chain(self.parts.keys().map(|k| crate::StoryRef::Part(*k))).collect();
        let mut n = 0;
        for s in stories {
            let Ok(bl) = self.story_mut(s) else { continue };
            for b in bl.iter_mut() {
                if uses_style(b, from, 0) {
                    n += restyle_block(Arc::make_mut(b), from, &to, 0);
                }
            }
        }
        n
    }

    /// Insert plain text at `pos` (`\n` inside `s` is a manual line break; use
    /// [`Document::split_paragraph`] for paragraph breaks). Returns the position after it.
    pub fn insert_text(&mut self, pos: &Pos, s: &str, props: &CharProps) -> Result<Pos> {
        let p = self.para_mut(pos.story, &pos.path)?;
        let n = p.insert_text(pos.off, s, props)?;
        Ok(Pos { off: pos.off + n, ..pos.clone() })
    }

    /// Insert an inline object at `pos`; returns the position after it.
    pub fn insert_object(&mut self, pos: &Pos, obj: InlineObject, props: &CharProps) -> Result<Pos> {
        let p = self.para_mut(pos.story, &pos.path)?;
        p.insert_object(pos.off, obj, props)?;
        Ok(Pos { off: pos.off + crate::para::OBJ.len_utf8(), ..pos.clone() })
    }

    /// Split the paragraph at `pos`; returns the start of the new (second) paragraph.
    pub fn split_paragraph(&mut self, pos: &Pos) -> Result<Pos> {
        let i = pos.path.last();
        let tail = self.para_mut(pos.story, &pos.path)?.split_off(pos.off)?;
        let bl = self.container_mut(pos.story, &pos.path)?;
        bl.insert((i + 1).min(bl.len()), para_block(tail));
        Ok(Pos { story: pos.story, path: pos.path.with_last(i + 1), off: 0 })
    }

    /// Insert a block before the block at `path` (same container). A table that would nest
    /// deeper than [`MAX_TABLE_DEPTH`] is refused.
    pub fn insert_block(&mut self, story: crate::StoryRef, path: &Path, block: Block) -> Result<()> {
        if let Block::Table(t) = &block
            && path.depth().saturating_add(table_height(t)) > MAX_TABLE_DEPTH
        {
            return Err(DocError::Invalid(format!("tables can't nest more than {MAX_TABLE_DEPTH} deep")));
        }
        let i = path.last();
        let bl = self.container_mut(story, path)?;
        if i > bl.len() {
            return Err(DocError::BadPath(path.to_string()));
        }
        bl.insert(i, Arc::new(block));
        Ok(())
    }

    /// Remove the block at `path` (keeps at least one paragraph in the container).
    pub fn remove_block(&mut self, story: crate::StoryRef, path: &Path) -> Result<Block> {
        let i = path.last();
        let bl = self.container_mut(story, path)?;
        if i >= bl.len() {
            return Err(DocError::BadPath(path.to_string()));
        }
        let b = bl.remove(i);
        if bl.is_empty() || matches!(bl.last().map(|b| &**b), Some(Block::Table(_))) {
            bl.push(para_block(Paragraph::new()));
        }
        Ok(Arc::try_unwrap(b).unwrap_or_else(|a| (*a).clone()))
    }

    /// Paragraph paths between `a` and `b` inclusive (same story), document order.
    pub fn paths_between(&self, a: &Pos, b: &Pos) -> Vec<Path> {
        let (a, b) = order(a, b);
        if a.story != b.story {
            return Vec::new();
        }
        self.para_paths(a.story).into_iter().filter(|p| *p >= a.path && *p <= b.path).collect()
    }

    /// Delete `a..b`. Returns the position where the deletion collapsed to. On error the document
    /// is unchanged: both ends are checked before anything is deleted.
    pub fn delete_range(&mut self, a: &Pos, b: &Pos) -> Result<Pos> {
        let (a, b) = order(a, b);
        if a == b {
            return Ok(a);
        }
        if a.story != b.story {
            return Err(DocError::Invalid("range spans stories".into()));
        }
        // Check both ends before the first edit, so an error never leaves the document half-deleted.
        for p in [&a, &b] {
            match self.para(p.story, &p.path) {
                Some(para) if para.clamp(p.off) == p.off => {}
                Some(_) => return Err(DocError::BadOffset(p.off)),
                None => return Err(DocError::BadPath(p.path.to_string())),
            }
        }
        // Content control markers whose partner is outside the range stay (the control is
        // emptied, not broken).
        let kept = self.kept_control_markers(&a, &b);
        let kept_in = |path: &Path| -> Vec<usize> { kept.iter().filter(|(p, _)| p == path).map(|(_, o)| *o).collect() };
        if a.path == b.path {
            self.para_mut(a.story, &a.path)?.delete_except(a.off, b.off, &kept_in(&a.path))?;
            return Ok(a);
        }
        if a.path.parent() == b.path.parent() {
            // Same container: trim the ends, drop the blocks between, join.
            let (ia, ib) = (a.path.last(), b.path.last());
            // Kept markers of the paragraphs in between move to the join.
            let mut moved: Vec<(InlineObject, CharProps)> = Vec::new();
            // `kept` is sorted: one walk of each paragraph's objects and runs (#3614).
            for group in kept.chunk_by(|x, y| x.0 == y.0) {
                let Some((path, _)) = group.first() else { continue };
                if *path == a.path || *path == b.path {
                    continue;
                }
                let Some(p) = self.para(a.story, path) else { continue };
                let mut runs = p.run_ranges().peekable();
                for (off, o) in p.objects_with_offsets() {
                    if group.binary_search_by_key(&off, |(_, k)| *k).is_err() {
                        continue;
                    }
                    while runs.next_if(|(r, _)| r.end <= off).is_some() {}
                    let props = runs.peek().filter(|(r, _)| r.contains(&off)).map_or(&p.mark, |(_, c)| *c);
                    moved.push((o.clone(), props.clone()));
                }
            }
            {
                let pa = self.para_mut(a.story, &a.path)?;
                let end = pa.len();
                pa.delete_except(a.off, end, &kept_in(&a.path))?;
                for (o, props) in moved {
                    let at = pa.len();
                    pa.insert_object(at, o, &props)?;
                }
            }
            let mut tail = {
                let pb = self.para_mut(b.story, &b.path)?;
                pb.delete_except(0, b.off, &kept_in(&b.path))?;
                pb.clone()
            };
            let bl = self.container_mut(a.story, &a.path)?;
            if ib < bl.len() && ia < ib {
                bl.drain(ia + 1..=ib);
            }
            tail.props = ParaProps::default();
            let pa = self.para_mut(a.story, &a.path)?;
            let keep_props = pa.props.clone();
            pa.append(tail);
            pa.props = keep_props;
            return Ok(a);
        }
        // Different containers (range enters or leaves a table): clear text in every paragraph
        // inside; remove whole tables that are entirely covered when at the same level as `a`.
        let paths = self.paths_between(&a, &b);
        for p in paths.iter().rev() {
            let (from, to) = {
                let Some(para) = self.para(a.story, p) else { continue };
                let from = if *p == a.path { a.off } else { 0 };
                let to = if *p == b.path { b.off } else { para.len() };
                (from, to)
            };
            self.para_mut(a.story, p)?.delete_except(from, to, &kept_in(p))?;
        }
        // Drop top-level tables strictly between a and b at a's level.
        if a.path.depth() == 0 && b.path.depth() > 0 {
            let first_b = b.path.0.first().copied().unwrap_or(0) as usize;
            let ia = a.path.last();
            let bl = self.container_mut(a.story, &a.path)?;
            let mut i = first_b;
            while i > ia + 1 {
                i -= 1;
                if matches!(bl.get(i).map(|b| &**b), Some(Block::Table(_))) {
                    bl.remove(i);
                }
            }
        }
        Ok(a)
    }

    /// Apply `f` to the character formatting of `a..b`. A collapsed range changes nothing (the
    /// engine keeps "pending" formatting for the caret instead).
    pub fn format_range(&mut self, a: &Pos, b: &Pos, f: &dyn Fn(&mut CharProps)) -> Result<()> {
        let (a, b) = order(a, b);
        for p in self.paths_between(&a, &b) {
            let para = self.para_mut(a.story, &p)?;
            let from = if p == a.path { a.off } else { 0 };
            let to = if p == b.path { b.off } else { para.len() };
            para.format(from, to, f)?;
            // A fully selected paragraph also gets the mark formatted (Word does), but a hyperlink
            // ends before the mark: a linked mark would carry the link into every paragraph Enter
            // makes from it (issue #413).
            if to == para.len() && (from == 0 || p != a.path) {
                let style = para.mark.style.clone();
                f(&mut para.mark);
                if para.mark.link.take().is_some() {
                    para.mark.style = style.filter(|s| s != "Hyperlink");
                }
            }
        }
        Ok(())
    }

    /// The paragraphs paragraph formatting of `a..b` applies to: [`Self::paths_between`] without
    /// the last paragraph when a non-empty range ends at its start (Select Paragraph ends there
    /// to include the paragraph mark). A caret applies to its own paragraph (Word, #418).
    pub fn para_paths_in(&self, a: &Pos, b: &Pos) -> Vec<Path> {
        let (a, b) = order(a, b);
        let mut paths = self.paths_between(&a, &b);
        if b.off == 0 && a.path != b.path && paths.len() > 1 && paths.last() == Some(&b.path) {
            paths.pop();
        }
        paths
    }

    /// Apply `f` to the paragraph properties of every paragraph `a..b` covers ([`Self::para_paths_in`]).
    pub fn format_paragraphs(&mut self, a: &Pos, b: &Pos, f: &dyn Fn(&mut ParaProps)) -> Result<()> {
        for p in self.para_paths_in(a, b) {
            let para = self.para_mut(a.story, &p)?;
            f(&mut para.props);
            para.touch();
        }
        Ok(())
    }

    /// Copy `a..b` into a fragment (with copies of the text boxes in it).
    pub fn copy_range(&self, a: &Pos, b: &Pos) -> Fragment {
        self.blocks_fragment(self.copy_blocks(a, b))
    }

    /// Copy complete blocks with the text-box stories they reference, including nested boxes.
    /// This also supports rectangular cell selections and separately selected drawings.
    pub fn blocks_fragment(&self, mut blocks: Vec<Block>) -> Fragment {
        // Half a content control (one marker without the other) isn't copied.
        crate::control::balance_blocks(&mut blocks);
        let parts = self.fragment_parts(&blocks);
        let media = crate::clipboard::copied_media(self, &blocks, &parts);
        let passthrough = crate::clipboard::copied_embedded(self, &blocks, &parts);
        Fragment { blocks, parts, media, passthrough, styles: Some(self.styles.clone()), numbering: Some(self.numbering.clone()) }
    }

    /// The whole body as a fragment (with its text boxes), to insert into another document.
    pub fn body_fragment(&self) -> Fragment {
        let blocks: Vec<Block> = self.body.iter().map(|b| (**b).clone()).collect();
        self.blocks_fragment(blocks)
    }

    /// Copies of referenced text boxes and notes, including their nested stories.
    fn fragment_parts(&self, blocks: &[Block]) -> BTreeMap<u32, Part> {
        let mut out = BTreeMap::new();
        let mut todo = Vec::new();
        for b in blocks {
            each_para(b, 0, &mut |p| todo.extend(referenced_parts(p)));
        }
        while let Some(id) = todo.pop() {
            if out.contains_key(&id) {
                continue;
            }
            let Some(part) = self.parts.get(&id).filter(|p| matches!(p.kind, PartKind::TextBox | PartKind::Footnote | PartKind::Endnote)) else {
                continue;
            };
            for b in &part.blocks {
                each_para(b, 0, &mut |p| todo.extend(referenced_parts(p)));
            }
            out.insert(id, part.clone());
        }
        out
    }

    fn copy_blocks(&self, a: &Pos, b: &Pos) -> Vec<Block> {
        let (a, b) = order(a, b);
        let mut out = Vec::new();
        if a.story != b.story {
            return out;
        }
        if a.path.parent() == b.path.parent() {
            let Some(bl) = self.container(a.story, &a.path) else { return out };
            for i in a.path.last()..=b.path.last() {
                let Some(blk) = bl.get(i) else { break };
                match &**blk {
                    Block::Para(p) => {
                        let mut p = p.clone();
                        let to = if i == b.path.last() { b.off } else { p.len() };
                        let from = if i == a.path.last() { a.off } else { 0 };
                        let _ = p.delete(to.min(p.len()), p.len());
                        let _ = p.delete(0, from.min(p.len()));
                        if i == b.path.last() && to < self.para(a.story, &b.path).map(Paragraph::len).unwrap_or(0) {
                            p.section = None;
                        }
                        out.push(Block::Para(p));
                    }
                    Block::Table(t) => out.push(Block::Table(t.clone())),
                }
            }
            return out;
        }
        for p in self.paths_between(&a, &b) {
            if let Some(para) = self.para(a.story, &p) {
                let mut q = para.clone();
                let to = if p == b.path { b.off } else { q.len() };
                let from = if p == a.path { a.off } else { 0 };
                let _ = q.delete(to.min(q.len()), q.len());
                let _ = q.delete(0, from.min(q.len()));
                q.section = None;
                out.push(Block::Para(q));
            }
        }
        out
    }

    /// Give copied boxes and notes independent stories. Captured fragments never fall back to
    /// an unrelated story with the same id in the destination.
    fn adopt_parts(&mut self, blocks: &mut [Block], parts: &BTreeMap<u32, Part>, fallback: bool, depth: usize, budget: &mut BoxBudget) -> Result<()> {
        let mut any = false;
        for b in blocks.iter() {
            each_para(b, 0, &mut |p| any |= referenced_parts(p).next().is_some());
        }
        if !any {
            return Ok(());
        }
        let mut error = None;
        for b in blocks.iter_mut() {
            each_para_mut(b, 0, &mut |p| {
                if error.is_some() {
                    return;
                }
                let mut changed = false;
                for o in &mut p.objects {
                    for story in o.text_box_slots() {
                        if let Some(old) = *story {
                            match self.adopt_part(old, PartKind::TextBox, parts, fallback, depth, budget) {
                                Ok(id) => {
                                    *story = id;
                                    changed = true;
                                }
                                Err(e) => {
                                    error = Some(e);
                                    return;
                                }
                            }
                        }
                    }
                    if let InlineObject::NoteRef { id, kind, .. } = o {
                        let part_kind = match kind {
                            crate::para::NoteKind::Footnote => PartKind::Footnote,
                            crate::para::NoteKind::Endnote => PartKind::Endnote,
                        };
                        match self.adopt_part(*id, part_kind, parts, fallback, depth, budget) {
                            Ok(Some(new)) => {
                                *id = new;
                                changed = true;
                            }
                            Ok(None) => {
                                error = Some(DocError::Invalid("a copied note is missing or exceeds the paste limit".into()));
                                return;
                            }
                            Err(e) => {
                                error = Some(e);
                                return;
                            }
                        }
                    }
                }
                if changed {
                    p.touch();
                }
            });
            if let Some(e) = error.take() {
                return Err(e);
            }
        }
        Ok(())
    }

    fn adopt_part(
        &mut self,
        old: u32,
        kind: PartKind,
        parts: &BTreeMap<u32, Part>,
        fallback: bool,
        depth: usize,
        budget: &mut BoxBudget,
    ) -> Result<Option<u32>> {
        if depth >= MAX_BOX_DEPTH || budget.active.contains(&old) {
            return Ok(None);
        }
        if budget.stories == 0 {
            budget.skipped = budget.skipped.saturating_add(1);
            return Ok(None);
        }
        let src = parts.get(&old).or_else(|| if fallback { self.parts.get(&old) } else { None }).filter(|p| p.kind == kind);
        let Some(src) = src else { return Ok(None) };
        let cost = *budget.costs.entry(old).or_insert_with(|| src.blocks.iter().fold(0usize, |a, b| a.saturating_add(block_cost(b))));
        let Some(left) = budget.bytes.checked_sub(cost) else {
            budget.skipped = budget.skipped.saturating_add(1);
            return Ok(None);
        };
        budget.bytes = left;
        budget.stories -= 1;
        let src_blocks = src.blocks.clone();
        let body = crate::TextBody { next: None, ..src.body };
        budget.active.push(old);
        let result = (|| -> Result<Blocks> {
            let mut blocks = Vec::with_capacity(src_blocks.len());
            for b in src_blocks {
                let mut refs = false;
                each_para(&b, 0, &mut |p| refs |= referenced_parts(p).next().is_some());
                if refs {
                    let mut copy = (*b).clone();
                    self.adopt_parts(std::slice::from_mut(&mut copy), parts, fallback, depth + 1, budget)?;
                    blocks.push(Arc::new(copy));
                } else {
                    blocks.push(b);
                }
            }
            Ok(blocks)
        })();
        budget.active.pop();
        let id = self.add_part(kind, result?);
        if let Some(p) = self.parts.get_mut(&id) {
            p.body = body;
        }
        Ok(Some(id))
    }

    /// Insert a fragment at `pos`; returns the position after the pasted content.
    pub fn insert_fragment(&mut self, pos: &Pos, frag: &Fragment) -> Result<Pos> {
        // Tables that would nest deeper than the limit here become their paragraphs.
        let room = MAX_TABLE_DEPTH.saturating_sub(pos.path.depth());
        let mut blocks = Vec::with_capacity(frag.blocks.len());
        for b in frag.blocks.iter().cloned() {
            fit_tables(b, room, &mut blocks);
        }
        if blocks.is_empty() {
            return Ok(pos.clone());
        }
        if self.para_at(pos).is_none() {
            return Err(DocError::BadPath(pos.path.to_string()));
        }
        if self.para_at(pos).is_some_and(|p| pos.off > p.len() || !p.text.is_char_boundary(pos.off)) {
            return Err(DocError::BadOffset(pos.off));
        }
        // Pasted text boxes get stories of their own (never shared with the copy's source).
        let mut budget = BoxBudget::new();
        let resources = (self.media.clone(), self.passthrough.clone(), self.styles.clone(), self.numbering.clone(), self.parts.clone());
        let parts = crate::clipboard::adopt(self, &mut blocks, frag)?;
        if let Err(e) = self.adopt_parts(&mut blocks, &parts, frag.styles.is_none(), 0, &mut budget) {
            (self.media, self.passthrough, self.styles, self.numbering, self.parts) = resources;
            return Err(e);
        }
        if budget.skipped > 0 {
            log::warn!("paste: {} text boxes were left empty: their stories were past the paste's limit", budget.skipped);
        }
        // A table pasted at a paragraph's start stands before it. Splitting at offset zero
        // would leave a new blank paragraph above the table (also in an empty document).
        if pos.off == 0
            && blocks.len() == 1
            && matches!(blocks.first(), Some(Block::Table(_)))
            && let Some(table) = blocks.pop()
        {
            self.insert_block(pos.story, &pos.path, table)?;
            return Ok(Pos { path: pos.path.with_last(pos.path.last().saturating_add(1)), off: 0, ..pos.clone() });
        }
        // Single paragraph: inline insert keeping runs.
        if blocks.len() == 1
            && let Some(Block::Para(src)) = blocks.first()
        {
            let src = src.clone();
            let target = self.para_mut(pos.story, &pos.path)?;
            let tail = target.split_off(pos.off)?;
            let added = src.len();
            let keep = target.props.clone();
            let sect = tail.section.clone();
            let mut src = src;
            src.section = None;
            target.append(src);
            target.append(tail);
            target.props = keep;
            target.section = sect;
            return Ok(Pos { off: pos.off + added, ..pos.clone() });
        }
        // Multi-block: split, merge first into left part, last into right part.
        let after = self.split_paragraph(pos)?;
        let left = pos.path.clone();
        let first = blocks.remove(0);
        let last = blocks.pop();
        let mut idx = left.last() + 1;
        match first {
            Block::Para(p) => {
                let l = self.para_mut(pos.story, &left)?;
                let keep = l.props.clone();
                let mut p2 = p;
                let pprops = p2.props.clone();
                p2.section = None;
                l.append(p2);
                l.props = if keep.is_empty() { pprops } else { keep };
            }
            t @ Block::Table(_) => {
                self.insert_block(pos.story, &left.with_last(idx), t)?;
                idx += 1;
            }
        }
        for b in blocks {
            self.insert_block(pos.story, &left.with_last(idx), b)?;
            idx += 1;
        }
        let right = after.path.with_last(idx);
        match last {
            Some(Block::Para(p)) => {
                let r = self.para_mut(pos.story, &right)?;
                let tail = std::mem::replace(r, p.clone());
                let off = r.len();
                let sect = tail.section.clone();
                r.append(tail);
                r.section = sect;
                Ok(Pos { story: pos.story, path: right, off })
            }
            Some(t @ Block::Table(_)) => {
                self.insert_block(pos.story, &right, t)?;
                Ok(Pos { story: pos.story, path: right.with_last(idx + 1), off: 0 })
            }
            None => Ok(Pos { story: pos.story, path: right, off: 0 }),
        }
    }

    /// Replace every block of a story.
    pub fn set_story(&mut self, story: crate::StoryRef, blocks: Blocks) -> Result<()> {
        let s = self.story_mut(story)?;
        *s = blocks;
        if s.is_empty() {
            s.push(para_block(Paragraph::new()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StoryRef, Table};

    fn doc(lines: &[&str]) -> Document {
        Document::from_text(&lines.join("\n"))
    }

    #[test]
    fn split_and_join() {
        let mut d = doc(&["Hello world"]);
        let p = d.split_paragraph(&Pos::body(0, 5)).unwrap();
        assert_eq!(p, Pos::body(1, 0));
        assert_eq!(d.plain_text(StoryRef::Body), "Hello\n world");
        let at = d.delete_range(&Pos::body(0, 5), &Pos::body(1, 0)).unwrap();
        assert_eq!(at, Pos::body(0, 5));
        assert_eq!(d.plain_text(StoryRef::Body), "Hello world");
    }

    #[test]
    fn delete_range_with_bad_end_leaves_document_unchanged() {
        let mut d = doc(&["one", "two", "three"]);
        let before = d.plain_text(StoryRef::Body);
        // Missing paragraph as the second end.
        assert!(d.delete_range(&Pos::body(0, 1), &Pos::body(9, 0)).is_err());
        assert_eq!(d.plain_text(StoryRef::Body), before);
        // Offset past the end of the second paragraph.
        assert!(d.delete_range(&Pos::body(0, 1), &Pos::body(1, 99)).is_err());
        assert_eq!(d.plain_text(StoryRef::Body), before);
        // Offset inside a multi-byte character.
        let mut d = doc(&["abc", "\u{e9}x"]);
        assert!(d.delete_range(&Pos::body(0, 1), &Pos::body(1, 1)).is_err());
        assert_eq!(d.plain_text(StoryRef::Body), "abc\n\u{e9}x");
        // Range entering a table with a bad offset at its start: the cell text stays.
        let mut d = doc(&["before", "after"]);
        d.insert_block(StoryRef::Body, &Path::top(1), Block::Table(Table::new(2, 1, 200.0))).unwrap();
        d.insert_text(&Pos { story: StoryRef::Body, path: Path(vec![1, 0, 0, 0]), off: 0 }, "x", &CharProps::default()).unwrap();
        let before = d.plain_text(StoryRef::Body);
        let cell = Pos { story: StoryRef::Body, path: Path(vec![1, 1, 0, 0]), off: 0 };
        assert!(d.delete_range(&Pos::body(0, 99), &cell).is_err());
        assert_eq!(d.plain_text(StoryRef::Body), before);
    }

    #[test]
    fn delete_multi() {
        let mut d = doc(&["one", "two", "three"]);
        d.delete_range(&Pos::body(2, 2), &Pos::body(0, 1)).unwrap();
        assert_eq!(d.plain_text(StoryRef::Body), "oree");
        assert_eq!(d.body.len(), 1);
    }

    #[test]
    fn format_across() {
        let mut d = doc(&["abc", "def"]);
        d.format_range(&Pos::body(0, 1), &Pos::body(1, 2), &|c| c.bold = Some(true)).unwrap();
        let p0 = d.para(StoryRef::Body, &Path::top(0)).unwrap();
        assert_eq!(p0.props_of_char(0).bold, None);
        assert_eq!(p0.props_of_char(1).bold, Some(true));
        let p1 = d.para(StoryRef::Body, &Path::top(1)).unwrap();
        assert_eq!(p1.props_of_char(1).bold, Some(true));
        assert_eq!(p1.props_of_char(2).bold, None);
    }

    #[test]
    fn copy_paste_round_trip() {
        let mut d = doc(&["alpha", "beta", "gamma"]);
        let f = d.copy_range(&Pos::body(0, 2), &Pos::body(2, 3));
        assert_eq!(f.plain_text(), "pha\nbeta\ngam");
        let end = d.insert_fragment(&Pos::body(2, 5), &f).unwrap();
        assert_eq!(d.plain_text(StoryRef::Body), "alpha\nbeta\ngammapha\nbeta\ngam");
        assert_eq!(end, Pos::body(4, 3));
        let one = Fragment::from_text("XY");
        let e = d.insert_fragment(&Pos::body(0, 0), &one).unwrap();
        assert_eq!(e, Pos::body(0, 2));
        assert!(d.plain_text(StoryRef::Body).starts_with("XYalpha"));
    }

    fn text_box(d: &mut Document, pos: &Pos, text: &str) -> u32 {
        let id = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text(text, CharProps::default()))]);
        d.insert_object(pos, box_shape(Some(id)), &CharProps::default()).unwrap();
        id
    }

    fn box_shape(story: Option<u32>) -> InlineObject {
        InlineObject::Shape {
            kind: crate::para::ShapeKind::TextBox,
            w: 144.0,
            h: 72.0,
            fill: None,
            stroke: None,
            stroke_width: 0.75,
            float: Default::default(),
            story,
            freeform: None,
            effects: Default::default(),
            extra: Default::default(),
        }
    }

    /// Story ids of the boxes in a body paragraph, in order.
    fn boxes_in(d: &Document, path: &Path) -> Vec<Option<u32>> {
        let p = d.para(StoryRef::Body, path).unwrap();
        p.objects.iter().filter_map(|o| if let InlineObject::Shape { story, .. } = o { Some(*story) } else { None }).collect()
    }

    #[test]
    fn pasted_text_box_gets_its_own_story() {
        let mut d = doc(&["ab", "cd"]);
        let id = text_box(&mut d, &Pos::body(0, 1), "Inside");
        let f = d.copy_range(&Pos::body(0, 0), &Pos::body(0, 5));
        assert_eq!(f.parts.keys().copied().collect::<Vec<_>>(), vec![id]);
        // Edit the original after copying: the paste keeps the copied text.
        d.insert_text(&Pos { story: StoryRef::Part(id), path: Path::top(0), off: 6 }, " later", &CharProps::default()).unwrap();
        d.insert_fragment(&Pos::body(1, 2), &f).unwrap();
        let pasted = boxes_in(&d, &Path::top(1));
        let [Some(new)] = pasted[..] else { panic!("{pasted:?}") };
        assert_ne!(new, id);
        assert_eq!(d.parts.get(&new).map(|p| p.kind), Some(PartKind::TextBox));
        assert_eq!(d.plain_text(StoryRef::Part(new)), "Inside");
        assert_eq!(d.plain_text(StoryRef::Part(id)), "Inside later");
        // Editing the copy leaves the original alone; a second paste is a third story.
        d.insert_text(&Pos { story: StoryRef::Part(new), path: Path::top(0), off: 0 }, "Copy: ", &CharProps::default()).unwrap();
        assert_eq!(d.plain_text(StoryRef::Part(id)), "Inside later");
        d.insert_fragment(&Pos::body(1, 0), &f).unwrap();
        let again = boxes_in(&d, &Path::top(1));
        assert_eq!(again.len(), 2);
        assert!(again.iter().all(|s| s.is_some_and(|s| s != id)) && again[0] != again[1], "{again:?}");
        // Multi-paragraph pastes too.
        let f2 = d.copy_range(&Pos::body(0, 0), &Pos::body(1, 1));
        let before = d.parts.len();
        d.insert_fragment(&Pos::body(1, 0), &f2).unwrap();
        assert_eq!(d.parts.len(), before + 1);
    }

    #[test]
    fn text_box_pasted_into_another_document_brings_its_text() {
        let mut src = doc(&["x"]);
        let id = text_box(&mut src, &Pos::body(0, 1), "From the source");
        let f = src.copy_range(&Pos::body(0, 0), &Pos::body(0, 1 + 3));
        // The target already has a header with the same part id.
        let mut dst = doc(&["target"]);
        let hdr = dst.add_part(PartKind::Header, vec![para_block(Paragraph::with_text("Header text", CharProps::default()))]);
        assert_eq!(hdr, id);
        dst.insert_fragment(&Pos::body(0, 6), &f).unwrap();
        let [Some(new)] = boxes_in(&dst, &Path::top(0))[..] else { panic!() };
        assert_ne!(new, hdr);
        assert_eq!(dst.plain_text(StoryRef::Part(new)), "From the source");
        assert_eq!(dst.plain_text(StoryRef::Part(hdr)), "Header text");
        // The whole body of a document, as Insert › Text from File uses it.
        let mut dst2 = doc(&["t"]);
        dst2.insert_fragment(&Pos::body(0, 1), &src.body_fragment()).unwrap();
        let [Some(n2)] = boxes_in(&dst2, &Path::top(0))[..] else { panic!() };
        assert_eq!(dst2.plain_text(StoryRef::Part(n2)), "From the source");
    }

    #[test]
    fn text_box_in_a_copied_table_gets_its_own_story() {
        let mut d = doc(&["before", "after"]);
        d.insert_block(StoryRef::Body, &Path::top(1), Block::Table(Table::new(1, 1, 200.0))).unwrap();
        let cell = Pos { story: StoryRef::Body, path: Path(vec![1, 0, 0, 0]), off: 0 };
        let id = text_box(&mut d, &cell, "In a cell");
        let f = d.copy_range(&Pos::body(0, 0), &Pos::body(2, 5));
        assert!(f.parts.contains_key(&id));
        d.insert_fragment(&Pos::body(2, 5), &f).unwrap();
        let ids: Vec<u32> = d
            .para_paths(StoryRef::Body)
            .iter()
            .filter_map(|p| d.para(StoryRef::Body, p))
            .flat_map(|p| p.objects.iter())
            .filter_map(|o| if let InlineObject::Shape { story, .. } = o { *story } else { None })
            .collect();
        assert_eq!(ids.len(), 2, "{ids:?}");
        assert_ne!(ids[0], ids[1]);
        assert!(ids.iter().all(|i| d.plain_text(StoryRef::Part(*i)) == "In a cell"));
    }

    #[test]
    fn fragment_without_parts_never_shows_an_unrelated_story() {
        let mut d = doc(&["x"]);
        let hdr = d.add_part(PartKind::Header, Vec::new());
        let own = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("Mine", CharProps::default()))]);
        let mut p = Paragraph::new();
        p.insert_object(0, box_shape(Some(hdr)), &CharProps::default()).unwrap();
        p.insert_object(0, box_shape(Some(own)), &CharProps::default()).unwrap();
        p.insert_object(0, box_shape(Some(999)), &CharProps::default()).unwrap();
        let f = Fragment { blocks: vec![Block::Para(p)], ..Default::default() };
        d.insert_fragment(&Pos::body(0, 1), &f).unwrap();
        let got = boxes_in(&d, &Path::top(0));
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], None, "missing story");
        let Some(copy) = got[1] else { panic!("{got:?}") };
        assert_ne!(copy, own);
        assert_eq!(d.plain_text(StoryRef::Part(copy)), "Mine");
        assert_eq!(got[2], None, "a header isn't a text box");
    }

    #[test]
    fn self_nested_text_box_paste_is_bounded() {
        let mut d = doc(&["x"]);
        let id = text_box(&mut d, &Pos::body(0, 1), "loop");
        // The box contains a box showing itself.
        d.insert_object(&Pos { story: StoryRef::Part(id), path: Path::top(0), off: 0 }, box_shape(Some(id)), &CharProps::default()).unwrap();
        let f = d.copy_range(&Pos::body(0, 0), &Pos::body(0, 1 + 3));
        let before = d.parts.len();
        d.insert_fragment(&Pos::body(0, 0), &f).unwrap();
        let added = d.parts.len() - before;
        // The copy's inner box would show the story being copied: a cycle, so it gets none.
        assert_eq!(added, 1);
    }

    /// A fragment whose body is one paragraph of `refs` text boxes all showing story 1.
    fn fan_out(story: Vec<Arc<Block>>, refs: usize) -> Fragment {
        let mut p = Paragraph::new();
        for _ in 0..refs {
            p.insert_object(0, box_shape(Some(1)), &CharProps::default()).unwrap();
        }
        let mut parts = BTreeMap::new();
        parts.insert(1, crate::Part { kind: PartKind::TextBox, blocks: story, body: Default::default() });
        Fragment { blocks: vec![Block::Para(p)], parts, ..Default::default() }
    }

    #[test]
    fn pasted_boxes_share_the_blocks_they_need_not_copy() {
        let big = para_block(Paragraph::with_text(&"x".repeat(100_000), CharProps::default()));
        let f = fan_out(vec![big.clone()], 3);
        let mut d = doc(&["x"]);
        d.insert_fragment(&Pos::body(0, 1), &f).unwrap();
        let got = boxes_in(&d, &Path::top(0));
        assert_eq!(got.len(), 3);
        let ids: Vec<u32> = got.into_iter().flatten().collect();
        assert_eq!(ids.len(), 3, "each box has a story");
        assert!(ids.windows(2).all(|w| w[0] != w[1]), "stories of their own: {ids:?}");
        for id in ids {
            let b = d.parts[&id].blocks.first().unwrap();
            assert!(Arc::ptr_eq(b, &big), "no text box inside: shared, not copied");
        }
    }

    #[test]
    fn pasted_box_copies_are_capped_in_bytes() {
        // Story 1 is a 1 MiB paragraph that holds a text box itself, so each box showing it needs
        // its own copy; 64 of them used to copy 64 MiB.
        let mut p = Paragraph::with_text(&"x".repeat(1 << 20), CharProps::default());
        p.insert_object(0, box_shape(Some(999)), &CharProps::default()).unwrap();
        let f = fan_out(vec![para_block(p)], 64);
        let mut d = doc(&["x"]);
        let before = d.parts.len();
        d.insert_fragment(&Pos::body(0, 1), &f).unwrap();
        let added = d.parts.len() - before;
        let copied: usize = d.parts.values().flat_map(|p| p.blocks.iter()).map(|b| if let Block::Para(p) = &**b { p.text.len() } else { 0 }).sum();
        assert!((1..64).contains(&added), "{added}");
        assert!(copied <= MAX_PASTED_BOX_BYTES, "{copied}");
        // The boxes past the cap keep no story rather than a shared one.
        let got = boxes_in(&d, &Path::top(0));
        assert_eq!(got.iter().flatten().count(), added);
    }

    #[test]
    fn pasted_box_copies_count_the_objects_they_copy() {
        // A story holding a text box and a 1 MiB kept-for-round-trip object costs the object's
        // XML too, not only its slot: 64 copies used to copy 64 MiB of it.
        let mut p = Paragraph::with_text("x", CharProps::default());
        p.insert_object(0, box_shape(Some(999)), &CharProps::default()).unwrap();
        let opaque = InlineObject::Opaque { format: "docx".into(), xml: "x".repeat(1 << 20), text: String::new() };
        p.insert_object(0, opaque, &CharProps::default()).unwrap();
        let f = fan_out(vec![para_block(p)], 64);
        let mut d = doc(&["x"]);
        let before = d.parts.len();
        d.insert_fragment(&Pos::body(0, 1), &f).unwrap();
        let added = d.parts.len() - before;
        let xml = |o: &InlineObject| if let InlineObject::Opaque { xml, .. } = o { xml.len() } else { 0 };
        let copied: usize =
            d.parts.values().flat_map(|p| p.blocks.iter()).map(|b| if let Block::Para(p) = &**b { p.objects.iter().map(xml).sum() } else { 0 }).sum();
        assert!((1..64).contains(&added), "{added}");
        assert!(copied <= MAX_PASTED_BOX_BYTES, "{copied}");
    }

    #[test]
    fn a_box_over_the_budget_leaves_later_smaller_boxes_their_stories() {
        // Once one story didn't fit, every later box would be left empty however small its
        // story; only the box that doesn't fit is.
        let big = para_block(Paragraph::with_text(&"x".repeat(100_000), CharProps::default()));
        let small = para_block(Paragraph::with_text("small", CharProps::default()));
        let mut p = Paragraph::new();
        for story in [2, 1, 2, 1] {
            p.insert_object(0, box_shape(Some(story)), &CharProps::default()).unwrap();
        }
        let mut parts = BTreeMap::new();
        parts.insert(1, crate::Part { kind: PartKind::TextBox, blocks: vec![big], body: Default::default() });
        parts.insert(2, crate::Part { kind: PartKind::TextBox, blocks: vec![small], body: Default::default() });
        let mut blocks = vec![Block::Para(p)];
        let mut d = doc(&["x"]);
        let mut budget = BoxBudget { bytes: 50_000, ..BoxBudget::new() };
        d.adopt_parts(&mut blocks, &parts, true, 0, &mut budget).unwrap();
        let Some(Block::Para(p)) = blocks.first() else { panic!() };
        let got: Vec<Option<u32>> = p.objects.iter().map(|o| if let InlineObject::Shape { story, .. } = o { *story } else { None }).collect();
        assert_eq!(got.iter().map(Option::is_some).collect::<Vec<_>>(), [false, true, false, true], "{got:?}");
        assert_eq!(budget.skipped, 2);
        assert_eq!(budget.costs.len(), 2, "each story costed once");
    }

    #[test]
    fn tables_in_paths() {
        let mut d = doc(&["before", "after"]);
        d.insert_block(StoryRef::Body, &Path::top(1), Block::Table(Table::new(2, 2, 200.0))).unwrap();
        let paths = d.para_paths(StoryRef::Body);
        assert_eq!(paths.len(), 6);
        assert_eq!(paths[1], Path(vec![1, 0, 0, 0]));
        let cell = Pos { story: StoryRef::Body, path: Path(vec![1, 0, 1, 0]), off: 0 };
        let e = d.insert_text(&cell, "x", &CharProps::default()).unwrap();
        assert_eq!(e.off, 1);
        assert_eq!(d.next_para(StoryRef::Body, &Path(vec![1, 0, 1, 0])), Some(Path(vec![1, 1, 0, 0])));
        assert!(d.plain_text(StoryRef::Body).contains("\tx"));
        // Delete from before the table into a cell: text cleared, table kept (partially covered).
        d.delete_range(&Pos::body(0, 2), &cell).unwrap();
        assert_eq!(d.para(StoryRef::Body, &Path::top(0)).unwrap().text, "be");
        assert!(d.container(StoryRef::Body, &Path(vec![1, 0, 0, 0])).is_some());
        assert_eq!(Path(vec![1, 0, 1, 0]).cell(), Some((Path::top(1), 0, 1)));
    }

    #[test]
    fn bad_positions_error() {
        let mut d = doc(&["é"]);
        assert!(d.insert_text(&Pos::body(0, 1), "x", &CharProps::default()).is_err());
        assert!(d.insert_text(&Pos::body(5, 0), "x", &CharProps::default()).is_err());
        assert!(d.split_paragraph(&Pos::body(0, 99)).is_err());
        let c = d.clamp(&Pos::body(9, 9));
        assert_eq!(c, Pos::body(0, 2));
        assert!(d.delete_range(&Pos { story: StoryRef::Part(3), path: Path::top(0), off: 0 }, &Pos::body(0, 0)).is_err());
    }

    /// The path of the first paragraph of the deepest table: each table sits in the first cell of
    /// the one around it, ahead of that cell's paragraph.
    fn deepest(d: &Document) -> Path {
        let mut path = vec![];
        let mut blocks: &[Arc<Block>] = &d.body;
        while let Some((i, Block::Table(t))) = blocks.iter().enumerate().find_map(|(i, b)| matches!(&**b, Block::Table(_)).then_some((i, &**b))) {
            path.extend([i as u32, 0, 0]);
            blocks = &t.rows[0].cells[0].blocks;
        }
        path.push(0);
        Path(path)
    }

    fn body_height(d: &Document) -> usize {
        d.body.iter().filter_map(|b| if let Block::Table(t) = &**b { Some(table_height(t)) } else { None }).max().unwrap_or(0)
    }

    /// How many times `c` is in `blocks`, the deepest cells too.
    fn count(blocks: &[Arc<Block>], c: char) -> usize {
        blocks
            .iter()
            .map(|b| match &**b {
                Block::Para(p) => p.text.matches(c).count(),
                Block::Table(t) => t.rows.iter().flat_map(|r| &r.cells).map(|cell| count(&cell.blocks, c)).sum(),
            })
            .sum()
    }

    #[test]
    fn tables_nest_at_most_max_table_depth() {
        let mut d = doc(&["x", ""]);
        for _ in 0..MAX_TABLE_DEPTH {
            let at = deepest(&d);
            d.insert_block(StoryRef::Body, &at, Block::Table(Table::new(1, 1, 100.0))).unwrap();
        }
        assert_eq!(body_height(&d), MAX_TABLE_DEPTH);
        // One more in the deepest cell is refused and changes nothing; a paragraph is fine.
        let (at, before) = (deepest(&d), d.clone());
        assert_eq!(at.depth(), MAX_TABLE_DEPTH);
        assert!(d.insert_block(StoryRef::Body, &at, Block::Table(Table::new(1, 1, 100.0))).is_err());
        assert_eq!(d, before);
        d.insert_block(StoryRef::Body, &at, Block::Para(Paragraph::new())).unwrap();
        // One level up, a table fits but a table holding a table doesn't.
        let up = Path(at.0[..at.0.len() - 3].to_vec());
        let mut two = Table::new(1, 1, 100.0);
        two.rows[0].cells[0].blocks.insert(0, Arc::new(Block::Table(Table::new(1, 1, 100.0))));
        assert!(d.insert_block(StoryRef::Body, &up, Block::Table(two)).is_err());
        d.insert_block(StoryRef::Body, &up, Block::Table(Table::new(1, 1, 100.0))).unwrap();
        assert_eq!(body_height(&d), MAX_TABLE_DEPTH);
    }

    #[test]
    fn pasting_a_table_into_its_own_cell_stops_at_max_table_depth() {
        // Copy everything and paste it into the deepest cell, again and again: each round
        // doubled the nesting, until saving, copying or closing the document could overflow
        // the stack.
        let mut d = doc(&["x", ""]);
        d.insert_block(StoryRef::Body, &Path::top(1), Block::Table(Table::new(1, 1, 100.0))).unwrap();
        d.insert_text(&Pos { story: StoryRef::Body, path: Path(vec![1, 0, 0, 0]), off: 0 }, "a", &CharProps::default()).unwrap();
        for _ in 0..8 {
            let all = d.copy_range(&Pos::body(0, 0), &Pos::body(d.body.len() - 1, 0));
            let at = deepest(&d);
            let off = d.para(StoryRef::Body, &at).map_or(0, Paragraph::len);
            d.insert_fragment(&Pos { story: StoryRef::Body, path: at, off }, &all).unwrap();
        }
        assert_eq!(body_height(&d), MAX_TABLE_DEPTH);
        // Nothing is lost: every copy's text is there, below the limit as paragraphs.
        assert_eq!((count(&d.body, 'x'), count(&d.body, 'a')), (256, 256));
    }
}
