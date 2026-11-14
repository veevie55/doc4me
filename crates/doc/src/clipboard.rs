//! Resources owned by a rich clipboard fragment. Importing them must not overwrite an image,
//! style or list with the same id in the destination document.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::edit::Fragment;
use crate::graphic::{EMBEDDED_PARTS, EmbeddedManifest, MAX_EMBEDDED_PARTS};
use crate::props::CharProps;
use crate::styles::Style;
use crate::{Block, DocError, Document, InlineObject, Part, Result};

const MAX_DEFINITIONS: usize = 4096;
const MAX_MEDIA_BYTES: usize = 128 << 20;
const MAX_MANIFEST_BYTES: usize = 16 << 20;

#[derive(Default)]
struct References<'a> {
    styles: BTreeSet<&'a str>,
    nums: BTreeSet<u32>,
    images: BTreeSet<&'a str>,
    embedded: BTreeSet<&'a str>,
}

fn references<'a>(roots: impl IntoIterator<Item = &'a Block>) -> References<'a> {
    let mut out = References::default();
    let mut todo: Vec<_> = roots.into_iter().collect();
    while let Some(b) = todo.pop() {
        match b {
            Block::Table(t) => {
                if let Some(id) = &t.props.style {
                    out.styles.insert(id);
                }
                todo.extend(t.rows.iter().flat_map(|r| &r.cells).flat_map(|c| &c.blocks).map(|b| &**b));
            }
            Block::Para(p) => {
                out.styles.insert(p.props.style.as_deref().unwrap_or("Normal"));
                for c in std::iter::once(&p.mark).chain(p.runs.iter().map(|r| &r.props)) {
                    if let Some(id) = &c.style {
                        out.styles.insert(id);
                    }
                }
                if let Some(n) = p.props.numbering.filter(|n| n.num != 0) {
                    out.nums.insert(n.num);
                }
                let mut objects: Vec<_> = p.objects.iter().collect();
                while let Some(o) = objects.pop() {
                    match o {
                        InlineObject::Image { media, ole, .. } => {
                            out.images.insert(media);
                            if let Some(e) = ole {
                                out.embedded.extend(e.refs.iter().filter(|r| !r.external).map(|r| r.target.as_str()));
                            }
                        }
                        InlineObject::Graphic { graphic, .. } => {
                            out.images.extend(graphic.items.iter().filter_map(|item| match item {
                                crate::graphic::GraphicItem::Image { media, .. } => Some(media.as_str()),
                                _ => None,
                            }));
                            if let Some(e) = &graphic.source {
                                out.embedded.extend(e.refs.iter().filter(|r| !r.external).map(|r| r.target.as_str()));
                            }
                        }
                        InlineObject::Group { children, .. } => objects.extend(children.iter().map(|c| &c.obj)),
                        _ => {}
                    }
                }
            }
        }
    }
    out
}

pub(crate) fn copied_media(doc: &Document, blocks: &[Block], parts: &BTreeMap<u32, Part>) -> BTreeMap<String, Arc<Vec<u8>>> {
    let refs = references(blocks.iter().chain(parts.values().flat_map(|p| p.blocks.iter().map(|b| &**b))));
    refs.images.into_iter().filter_map(|key| doc.media.get(key).map(|data| (key.to_string(), data.clone()))).collect()
}

fn style_chars(st: &Style) -> Vec<&CharProps> {
    let mut out = vec![&st.chr];
    if let Some(t) = &st.table {
        out.extend([&t.header_chr, &t.band_chr, &t.first_col_chr, &t.total_chr, &t.last_col_chr, &t.col_band_chr]);
    }
    out
}

fn style_chars_mut(st: &mut Style) -> Vec<&mut CharProps> {
    let mut out = vec![&mut st.chr];
    if let Some(t) = &mut st.table {
        out.extend([&mut t.header_chr, &mut t.band_chr, &mut t.first_col_chr, &mut t.total_chr, &mut t.last_col_chr, &mut t.col_band_chr]);
    }
    out
}

fn map_style(id: &mut Option<String>, map: &BTreeMap<String, String>) {
    if let Some(new) = id.as_ref().and_then(|id| map.get(id)) {
        *id = Some(new.clone());
    }
}

fn map_num(n: &mut Option<crate::props::NumRef>, map: &BTreeMap<u32, u32>) {
    if let Some(n) = n
        && let Some(id) = map.get(&n.num)
    {
        n.num = *id;
    }
}

fn free_id(used: &BTreeSet<u32>, start: u32) -> Result<u32> {
    for i in 0..=used.len() {
        let Some(id) = u32::try_from(i).ok().and_then(|i| i.checked_add(start)) else { break };
        if !used.contains(&id) {
            return Ok(id);
        }
    }
    Err(DocError::Invalid("no free clipboard list id".into()))
}

fn unique_name(used: &BTreeSet<String>, prefix: &str, suffix: &str) -> String {
    // At most `used.len()` candidates can be occupied.
    for i in 0..=used.len() {
        let key = format!("{prefix}Paste{i}{suffix}");
        if !used.contains(&key) && !used.contains(&key.to_ascii_lowercase()) {
            return key;
        }
    }
    // The loop above always has a free candidate; this fallback is finite for hostile input.
    format!("{prefix}Paste{}_{suffix}", used.len())
}

#[derive(Default)]
struct Maps {
    styles: BTreeMap<String, String>,
    nums: BTreeMap<u32, u32>,
    images: BTreeMap<String, String>,
    embedded: BTreeMap<String, String>,
}

fn remap_embedded(e: &mut Arc<crate::graphic::Embedded>, map: &BTreeMap<String, String>) {
    if e.refs.iter().any(|r| !r.external && map.contains_key(&r.target)) {
        for r in &mut Arc::make_mut(e).refs {
            if !r.external
                && let Some(target) = map.get(&r.target)
            {
                r.target = target.clone();
            }
        }
    }
}

fn remap(block: &mut Block, maps: &Maps) {
    let mut todo = vec![block];
    while let Some(b) = todo.pop() {
        match b {
            Block::Table(t) => {
                map_style(&mut t.props.style, &maps.styles);
                todo.extend(t.rows.iter_mut().flat_map(|r| &mut r.cells).flat_map(|c| &mut c.blocks).map(Arc::make_mut));
            }
            Block::Para(p) => {
                if p.props.style.is_none()
                    && let Some(id) = maps.styles.get("Normal")
                {
                    p.props.style = Some(id.clone());
                }
                map_style(&mut p.props.style, &maps.styles);
                map_num(&mut p.props.numbering, &maps.nums);
                map_style(&mut p.mark.style, &maps.styles);
                for r in &mut p.runs {
                    map_style(&mut r.props.style, &maps.styles);
                }
                let mut objects: Vec<_> = p.objects.iter_mut().collect();
                while let Some(o) = objects.pop() {
                    match o {
                        InlineObject::Image { media, ole, .. } => {
                            if let Some(key) = maps.images.get(media) {
                                *media = key.clone();
                            }
                            if let Some(e) = ole {
                                remap_embedded(e, &maps.embedded);
                            }
                        }
                        InlineObject::Graphic { graphic, .. } => {
                            let images = graphic
                                .items
                                .iter()
                                .any(|item| matches!(item, crate::graphic::GraphicItem::Image { media, .. } if maps.images.contains_key(media)));
                            if images
                                || graphic
                                    .source
                                    .as_ref()
                                    .is_some_and(|e| e.refs.iter().any(|r| !r.external && maps.embedded.contains_key(&r.target)))
                            {
                                let g = Arc::make_mut(graphic);
                                if let Some(e) = &mut g.source {
                                    remap_embedded(e, &maps.embedded);
                                }
                                for item in &mut g.items {
                                    if let crate::graphic::GraphicItem::Image { media, .. } = item
                                        && let Some(new) = maps.images.get(media)
                                    {
                                        *media = new.clone();
                                    }
                                }
                            }
                        }
                        InlineObject::Group { children, .. } => objects.extend(children.iter_mut().map(|c| &mut c.obj)),
                        _ => {}
                    }
                }
                p.touch();
            }
        }
    }
}

fn equivalent_style(src: &BTreeMap<&str, &Style>, dst: &BTreeMap<&str, &Style>, id: &str, defaults_same: bool, numbering_same: bool) -> bool {
    if !defaults_same {
        return false;
    }
    let mut todo = vec![id.to_string()];
    let mut seen = BTreeSet::new();
    while let Some(id) = todo.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if seen.len() > MAX_DEFINITIONS {
            return false;
        }
        match (src.get(id.as_str()), dst.get(id.as_str())) {
            (None, None) => {}
            (Some(a), Some(b)) if a == b => {
                if !numbering_same && a.para.numbering.is_some_and(|n| n.num != 0) {
                    return false;
                }
                todo.extend(a.based_on.iter().chain(&a.next).chain(&a.linked).chain(&a.para.style).cloned());
                todo.extend(style_chars(a).into_iter().filter_map(|c| c.style.clone()));
            }
            _ => return false,
        }
    }
    true
}

/// Plan the import in separate maps first. Validation errors leave the destination resources
/// unchanged, and equal resources in the same document keep their ids and shared blocks.
pub(crate) fn adopt(doc: &mut Document, blocks: &mut [Block], frag: &Fragment) -> Result<BTreeMap<u32, Part>> {
    let refs = references(blocks.iter().chain(frag.parts.values().flat_map(|p| p.blocks.iter().map(|b| &**b))));
    if frag.styles.is_some() && refs.images.iter().any(|key| !frag.media.contains_key(*key)) {
        return Err(DocError::Invalid("a copied picture's bytes are missing".into()));
    }
    let mut maps = Maps::default();
    let mut media = doc.media.clone();
    let mut keys: BTreeSet<_> = media.keys().map(|key| key.to_ascii_lowercase()).collect();
    let mut bytes = 0usize;
    for (key, data) in &frag.media {
        if media.get(key).is_some_and(|d| d == data) {
            continue;
        }
        bytes = bytes.saturating_add(data.len());
        if bytes > MAX_MEDIA_BYTES {
            return Err(DocError::Invalid("copied pictures exceed the 128 MiB paste limit".into()));
        }
        let ext = key
            .rsplit_once('.')
            .map(|(_, e)| e)
            .filter(|e| !e.is_empty() && e.len() <= 8 && e.bytes().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("bin");
        let new = unique_name(&keys, "image", &format!(".{ext}"));
        keys.insert(new.to_ascii_lowercase());
        media.insert(new.clone(), data.clone());
        maps.images.insert(key.clone(), new);
    }
    let (passthrough, embedded) = adopt_embedded(doc, frag, &refs.embedded, &mut bytes)?;
    maps.embedded = embedded;
    let numbering_same = frag.numbering.as_ref().is_none_or(|n| **n == *doc.numbering);
    let import_styles = frag.styles.as_ref().is_some_and(|s| **s != *doc.styles || !numbering_same);
    let source_styles: BTreeMap<_, _> = frag.styles.as_ref().map(|s| s.styles.iter().map(|st| (st.id.as_str(), st)).collect()).unwrap_or_default();
    enum Work {
        Style(String),
        Num(u32),
    }
    let mut work: Vec<_> = refs.styles.iter().map(|id| Work::Style((*id).to_string())).chain(refs.nums.iter().copied().map(Work::Num)).collect();
    let mut styles = BTreeSet::new();
    let mut nums = BTreeSet::new();
    while let Some(next) = work.pop() {
        match next {
            Work::Style(id) => {
                if !styles.insert(id.clone()) {
                    continue;
                }
                if styles.len() > MAX_DEFINITIONS {
                    return Err(DocError::Invalid("too many copied styles".into()));
                }
                let Some(st) = source_styles.get(id.as_str()) else { continue };
                work.extend(st.based_on.iter().chain(&st.linked).chain(&st.next).chain(&st.para.style).map(|id| Work::Style(id.clone())));
                work.extend(style_chars(st).into_iter().filter_map(|c| c.style.as_ref()).map(|id| Work::Style(id.clone())));
                if let Some(n) = st.para.numbering.filter(|n| n.num != 0) {
                    work.push(Work::Num(n.num));
                }
            }
            Work::Num(id) => {
                if !nums.insert(id) {
                    continue;
                }
                if nums.len() > MAX_DEFINITIONS {
                    return Err(DocError::Invalid("too many copied lists".into()));
                }
                let Some(src) = &frag.numbering else { continue };
                let Some(n) = src.num(id) else { continue };
                let a = src.abstracts.iter().find(|a| a.id == n.abstract_id);
                for level in a.into_iter().flat_map(|a| &a.levels).chain(n.level_overrides.iter().map(|(_, l)| l)) {
                    work.extend(level.style.iter().chain(&level.chr.style).map(|id| Work::Style(id.clone())));
                }
            }
        }
    }
    let mut sheet = doc.styles.clone();
    let mut pending = Vec::new();
    let destination_styles: BTreeMap<_, _> = doc.styles.styles.iter().map(|s| (s.id.as_str(), s)).collect();
    if import_styles && let Some(src) = &frag.styles {
        let mut reserved: BTreeSet<_> = sheet.styles.iter().map(|s| s.id.clone()).chain(styles.iter().cloned()).collect();
        for id in &styles {
            let Some(st) = source_styles.get(id.as_str()) else { continue };
            let defaults_same = src.default_chr == doc.styles.default_chr && src.default_para == doc.styles.default_para;
            if equivalent_style(&source_styles, &destination_styles, id, defaults_same, numbering_same) {
                continue;
            }
            let new = if sheet.get(id).is_some() { unique_name(&reserved, id, "") } else { id.clone() };
            reserved.insert(new.clone());
            if *id != new {
                maps.styles.insert(id.clone(), new.clone());
            }
            let mut st = (**st).clone();
            st.id = new;
            // Freeze the source's document defaults at the roots of its imported style chains.
            if st.based_on.is_none() && !defaults_same {
                st.para = src.default_para.clone().overlaid(&st.para);
                let mut chr = src.default_chr.clone();
                chr.overlay(&st.chr);
                st.chr = chr;
            }
            pending.push(st);
        }
    }
    let mut numbering = doc.numbering.clone();
    if (!numbering_same || !maps.styles.is_empty())
        && let Some(src) = &frag.numbering
    {
        let mut used_abs: BTreeSet<_> = numbering.abstracts.iter().map(|a| a.id).collect();
        let mut used_num: BTreeSet<_> = numbering.nums.iter().map(|n| n.id).collect();
        let mut abs_map = BTreeMap::new();
        for id in nums {
            let Some(n) = src.num(id) else {
                maps.nums.insert(id, 0);
                continue;
            };
            let Some(a) = src.abstracts.iter().find(|a| a.id == n.abstract_id) else {
                maps.nums.insert(id, 0);
                continue;
            };
            let abstract_id = if let Some(id) = abs_map.get(&a.id) {
                *id
            } else {
                let id = free_id(&used_abs, 0)?;
                used_abs.insert(id);
                abs_map.insert(a.id, id);
                let mut a = a.clone();
                a.id = id;
                for l in &mut a.levels {
                    map_style(&mut l.style, &maps.styles);
                    map_style(&mut l.chr.style, &maps.styles);
                }
                Arc::make_mut(&mut numbering).abstracts.push(a);
                id
            };
            let new = free_id(&used_num, 1)?;
            used_num.insert(new);
            maps.nums.insert(id, new);
            let mut n = n.clone();
            n.id = new;
            n.abstract_id = abstract_id;
            for (_, l) in &mut n.level_overrides {
                map_style(&mut l.style, &maps.styles);
                map_style(&mut l.chr.style, &maps.styles);
            }
            Arc::make_mut(&mut numbering).nums.push(n);
        }
    }
    for mut st in pending {
        for id in [&mut st.based_on, &mut st.next, &mut st.linked, &mut st.para.style] {
            map_style(id, &maps.styles);
        }
        map_num(&mut st.para.numbering, &maps.nums);
        for c in style_chars_mut(&mut st) {
            map_style(&mut c.style, &maps.styles);
        }
        Arc::make_mut(&mut sheet).upsert(st);
    }
    let changed = |b: &Block| {
        let r = references(std::iter::once(b));
        r.styles.iter().any(|id| maps.styles.contains_key(*id))
            || r.nums.iter().any(|id| maps.nums.contains_key(id))
            || r.images.iter().any(|id| maps.images.contains_key(*id))
            || r.embedded.iter().any(|id| maps.embedded.contains_key(*id))
    };
    for b in blocks {
        if changed(b) {
            remap(b, &maps);
        }
    }
    let mut parts = frag.parts.clone();
    for part in parts.values_mut() {
        for b in &mut part.blocks {
            if changed(b) {
                remap(Arc::make_mut(b), &maps);
            }
        }
    }
    doc.passthrough = passthrough;
    doc.media = media;
    doc.styles = sheet;
    doc.numbering = numbering;
    Ok(parts)
}

/// Keep only the dependency graph of copied charts/OLE/diagrams, without unrelated custom XML,
/// macros or settings from the source package.
pub(crate) fn copied_embedded(doc: &Document, blocks: &[Block], parts: &BTreeMap<u32, Part>) -> BTreeMap<String, Arc<Vec<u8>>> {
    let refs = references(blocks.iter().chain(parts.values().flat_map(|p| p.blocks.iter().map(|b| &**b))));
    if refs.embedded.is_empty() {
        return BTreeMap::new();
    }
    let mut out = BTreeMap::new();
    let Some(raw) = doc.passthrough.get(EMBEDDED_PARTS) else {
        out.insert(EMBEDDED_PARTS.into(), Arc::new(Vec::new()));
        return out;
    };
    if raw.len() > MAX_MANIFEST_BYTES {
        out.insert(EMBEDDED_PARTS.into(), raw.clone());
        return out;
    }
    let src = EmbeddedManifest::parse(raw);
    let mut kept = EmbeddedManifest::default();
    let mut todo: Vec<_> = refs.embedded.into_iter().map(str::to_string).collect();
    let mut seen = BTreeSet::new();
    while let Some(key) = todo.pop() {
        if !seen.insert(key.clone()) || seen.len() > MAX_EMBEDDED_PARTS {
            continue;
        }
        let (Some(part), Some(data)) = (src.parts.get(&key), doc.passthrough.get(&key)) else { continue };
        todo.extend(part.rels.iter().filter(|r| !r.external).map(|r| r.target.clone()));
        kept.parts.insert(key.clone(), part.clone());
        out.insert(key, data.clone());
    }
    out.insert(EMBEDDED_PARTS.into(), Arc::new(kept.to_bytes()));
    out
}

type KeptParts = BTreeMap<String, Arc<Vec<u8>>>;

fn adopt_embedded(doc: &Document, frag: &Fragment, roots: &BTreeSet<&str>, bytes: &mut usize) -> Result<(KeptParts, BTreeMap<String, String>)> {
    let mut out = doc.passthrough.clone();
    let Some(raw) = frag.passthrough.get(EMBEDDED_PARTS) else { return Ok((out, BTreeMap::new())) };
    if raw.len() > MAX_MANIFEST_BYTES || doc.passthrough.get(EMBEDDED_PARTS).is_some_and(|m| m.len() > MAX_MANIFEST_BYTES) {
        return Err(DocError::Invalid("copied object metadata exceeds 16 MiB".into()));
    }
    let src = EmbeddedManifest::parse(raw);
    let mut dest = doc.passthrough.get(EMBEDDED_PARTS).map(|m| EmbeddedManifest::parse(m)).unwrap_or_default();
    let mut todo: Vec<_> = roots.iter().map(|k| (*k).to_string()).collect();
    let mut nodes = BTreeSet::new();
    while let Some(key) = todo.pop() {
        if !nodes.insert(key.clone()) {
            continue;
        }
        if nodes.len() > MAX_EMBEDDED_PARTS {
            return Err(DocError::Invalid("too many copied object parts".into()));
        }
        let part = src
            .parts
            .get(&key)
            .filter(|_| frag.passthrough.contains_key(&key))
            .ok_or_else(|| DocError::Invalid("a copied object's original part is missing".into()))?;
        todo.extend(part.rels.iter().filter(|r| !r.external).map(|r| r.target.clone()));
    }
    let rename = nodes.iter().any(|key| {
        out.keys().any(|k| k != key && k.eq_ignore_ascii_case(key))
            || out.get(key).is_some_and(|d| Some(d) != frag.passthrough.get(key))
            || dest.parts.get(key).is_some_and(|p| Some(p) != src.parts.get(key))
    });
    let mut reserved: BTreeSet<_> = out.keys().chain(nodes.iter()).map(|k| k.to_ascii_lowercase()).collect();
    let mut map = BTreeMap::new();
    if rename {
        for key in &nodes {
            let ext = key
                .rsplit_once('.')
                .map(|(_, e)| e)
                .filter(|e| !e.is_empty() && e.len() <= 8 && e.bytes().all(|b| b.is_ascii_alphanumeric()))
                .unwrap_or("bin");
            let new = unique_name(&reserved, "word/embeddings/part", &format!(".{ext}"));
            reserved.insert(new.to_ascii_lowercase());
            map.insert(key.clone(), new);
        }
    }
    for key in nodes {
        let Some(data) = frag.passthrough.get(&key) else { continue };
        let Some(part) = src.parts.get(&key) else { continue };
        let target = map.get(&key).cloned().unwrap_or_else(|| key.clone());
        if !out.get(&target).is_some_and(|d| d == data) {
            *bytes = bytes.saturating_add(data.len());
            if *bytes > MAX_MEDIA_BYTES {
                return Err(DocError::Invalid("copied objects exceed the 128 MiB paste limit".into()));
            }
        }
        let mut part = part.clone();
        for r in &mut part.rels {
            if !r.external
                && let Some(target) = map.get(&r.target)
            {
                r.target = target.clone();
            }
        }
        dest.parts.insert(target.clone(), part);
        out.insert(target, data.clone());
    }
    if dest.parts.len() > MAX_EMBEDDED_PARTS {
        return Err(DocError::Invalid("too many embedded parts in the destination document".into()));
    }
    let manifest = dest.to_bytes();
    if manifest.len() > MAX_MANIFEST_BYTES {
        return Err(DocError::Invalid("combined object metadata exceeds 16 MiB".into()));
    }
    out.insert(EMBEDDED_PARTS.into(), Arc::new(manifest));
    Ok((out, map))
}
