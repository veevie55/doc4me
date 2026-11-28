//! Font files to embed in a saved document (File › Options › Save › Embed fonts in the file).
//!
//! [`prepare`] decides whether a face may be embedded, the way Word decides it, and returns a
//! standalone TrueType file for it:
//!
//! * The font's own licence flags (`OS/2.fsType`, OpenType spec "OS/2 — fsType") are obeyed:
//!   *Restricted License embedding* and *Bitmap embedding only* faces are not embedded.
//!   *Preview & Print*, *Editable* and *Installable* faces are.
//! * Only TrueType outlines (`glyf`) are embedded: Word does not use embedded CFF faces.
//!   Variable fonts are not embedded either: Word takes an embedded file as one static face.
//! * A face from a collection (`.ttc`) is written as a font file of its own.
//! * "Embed only the characters used": glyphs the document doesn't use are emptied, and every
//!   glyph keeps its number, so the character map, metrics and layout tables stay valid (a
//!   subsetter that renumbers glyphs, as PDF writers do, would break them). A face whose
//!   licence says *No subsetting* is embedded whole.
//!
//! Everything is read with bounds checks; a damaged face is refused, never a panic.

use std::collections::BTreeSet;

use skrifa::raw::TableProvider;
use skrifa::raw::types::Tag;

use crate::FontFace;

/// Largest face embedded (a whole CJK face is 20 MB or so).
pub const MAX_FACE_BYTES: usize = 64 * 1024 * 1024;
/// Most tables copied from one face.
const MAX_TABLES: usize = 256;
/// Most rounds of following composite glyphs' components (they nest a few deep at most).
const MAX_COMPONENT_DEPTH: usize = 16;

/// Why a face was not embedded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The licence forbids embedding (`fsType` *Restricted License embedding*).
    Restricted,
    /// The licence allows only bitmaps to be embedded, and the face is outlines.
    BitmapOnly,
    /// CFF (PostScript) outlines: Word doesn't use them embedded.
    NotTrueType,
    /// A variable font.
    Variable,
    /// Larger than [`MAX_FACE_BYTES`].
    TooLarge,
    /// The file couldn't be read.
    Damaged,
}

impl Refusal {
    /// Why, in a few words, for the status line.
    pub fn reason(self) -> &'static str {
        match self {
            Refusal::Restricted => "its licence doesn't allow embedding",
            Refusal::BitmapOnly => "its licence allows embedding bitmaps only",
            Refusal::NotTrueType => "it isn't a TrueType font",
            Refusal::Variable => "it's a variable font",
            Refusal::TooLarge => "it's too large",
            Refusal::Damaged => "the font file can't be read",
        }
    }
}

/// A face ready to embed.
#[derive(Clone, Debug)]
pub struct Prepared {
    /// A standalone TrueType file.
    pub data: Vec<u8>,
    /// Whether unused glyphs were emptied ([`prepare`]'s `keep`).
    pub subsetted: bool,
    /// The licence allows the document to be viewed and printed only, not edited (`fsType`
    /// *Preview & Print*); Word opens such a document read-only.
    pub preview_only: bool,
}

/// `fsType` bits (OpenType spec, OS/2 table).
const FS_RESTRICTED: u16 = 0x0002;
const FS_PREVIEW_PRINT: u16 = 0x0004;
const FS_EDITABLE: u16 = 0x0008;
const FS_NO_SUBSETTING: u16 = 0x0100;
const FS_BITMAP_ONLY: u16 = 0x0200;

/// Whether `face` may be embedded, and what its licence allows: `Ok(preview_only)`.
pub fn permission(face: &FontFace) -> Result<bool, Refusal> {
    let font = face.skrifa().ok_or(Refusal::Damaged)?;
    // A face without OS/2 (old Mac fonts) says nothing: Word embeds it.
    let fs = font.os2().map(|o| o.fs_type()).unwrap_or(0);
    // The usage bits are exclusive in OS/2 version 3 and later; in older faces several can be
    // set, and the least restrictive one counts.
    let usage = fs & 0x000F;
    if usage & FS_RESTRICTED != 0 && usage & (FS_PREVIEW_PRINT | FS_EDITABLE) == 0 {
        return Err(Refusal::Restricted);
    }
    if fs & FS_BITMAP_ONLY != 0 {
        return Err(Refusal::BitmapOnly);
    }
    if font.table_data(Tag::new(b"glyf")).is_none() || font.table_data(Tag::new(b"loca")).is_none() {
        return Err(Refusal::NotTrueType);
    }
    if face.is_variable() || font.table_data(Tag::new(b"fvar")).is_some() {
        return Err(Refusal::Variable);
    }
    Ok(usage & FS_PREVIEW_PRINT != 0 && usage & FS_EDITABLE == 0)
}

/// The font file to embed for `face`: whole, or with only the glyphs in `keep` (glyph ids the
/// document uses; glyph 0 and composite glyphs' parts are kept as well).
pub fn prepare(face: &FontFace, keep: Option<&BTreeSet<u32>>) -> Result<Prepared, Refusal> {
    let preview_only = permission(face)?;
    let font = face.skrifa().ok_or(Refusal::Damaged)?;
    let no_subsetting = font.os2().map(|o| o.fs_type() & FS_NO_SUBSETTING != 0).unwrap_or(false);
    let mut tables: Vec<([u8; 4], Vec<u8>)> = Vec::new();
    let mut total = 0usize;
    for record in font.table_directory().table_records().iter().take(MAX_TABLES) {
        let tag = record.tag();
        let Some(data) = font.table_data(tag) else { continue };
        let bytes = data.as_bytes();
        total = total.saturating_add(bytes.len());
        if total > MAX_FACE_BYTES {
            return Err(Refusal::TooLarge);
        }
        tables.push((tag.to_be_bytes(), bytes.to_vec()));
    }
    let subsetted = match keep {
        Some(keep) if !no_subsetting => {
            subset(&mut tables, keep)?;
            true
        }
        _ => false,
    };
    Ok(Prepared { data: write_sfnt(tables)?, subsetted, preview_only })
}

fn table<'a>(tables: &'a [([u8; 4], Vec<u8>)], tag: &[u8; 4]) -> Option<&'a [u8]> {
    tables.iter().find(|(t, _)| t == tag).map(|(_, d)| d.as_slice())
}

fn table_mut<'a>(tables: &'a mut [([u8; 4], Vec<u8>)], tag: &[u8; 4]) -> Option<&'a mut Vec<u8>> {
    tables.iter_mut().find(|(t, _)| t == tag).map(|(_, d)| d)
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*b.get(at)?, *b.get(at + 1)?]))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes([*b.get(at)?, *b.get(at + 1)?, *b.get(at + 2)?, *b.get(at + 3)?]))
}

/// Glyph `gid`'s bytes in `glyf`, through `loca` in its format (`long`: 32-bit offsets).
fn glyph<'a>(glyf: &'a [u8], loca: &[u8], long: bool, gid: usize) -> Option<&'a [u8]> {
    let (start, end) = if long {
        (u32_at(loca, gid * 4)? as usize, u32_at(loca, gid * 4 + 4)? as usize)
    } else {
        (usize::from(u16_at(loca, gid * 2)?) * 2, usize::from(u16_at(loca, gid * 2 + 2)?) * 2)
    };
    if end < start {
        return None;
    }
    glyf.get(start..end)
}

/// The glyphs a composite glyph is built from (TrueType spec, "glyf" composite descriptions).
fn components(g: &[u8]) -> Vec<u32> {
    const ARGS_ARE_WORDS: u16 = 0x0001;
    const HAVE_SCALE: u16 = 0x0008;
    const MORE: u16 = 0x0020;
    const HAVE_XY_SCALE: u16 = 0x0040;
    const HAVE_2X2: u16 = 0x0080;
    let mut out = Vec::new();
    // A simple glyph (or an empty one) has no components.
    let Some(contours) = u16_at(g, 0).map(|c| c as i16) else { return out };
    if contours >= 0 {
        return out;
    }
    let mut at = 10;
    // Every component takes at least 6 bytes, which bounds the loop.
    while let (Some(flags), Some(gid)) = (u16_at(g, at), u16_at(g, at + 2)) {
        out.push(u32::from(gid));
        at += 4 + if flags & ARGS_ARE_WORDS != 0 { 4 } else { 2 };
        at += if flags & HAVE_SCALE != 0 {
            2
        } else if flags & HAVE_XY_SCALE != 0 {
            4
        } else if flags & HAVE_2X2 != 0 {
            8
        } else {
            0
        };
        if flags & MORE == 0 {
            break;
        }
    }
    out
}

/// Empty every glyph not in `keep` (nor glyph 0, nor a part of a kept composite glyph). Glyph
/// numbers stay; `loca` is rewritten in the long format.
fn subset(tables: &mut [([u8; 4], Vec<u8>)], keep: &BTreeSet<u32>) -> Result<(), Refusal> {
    let head = table(tables, b"head").ok_or(Refusal::Damaged)?;
    let long = u16_at(head, 50).ok_or(Refusal::Damaged)? != 0;
    let glyphs = usize::from(u16_at(table(tables, b"maxp").ok_or(Refusal::Damaged)?, 4).ok_or(Refusal::Damaged)?);
    let glyf = table(tables, b"glyf").ok_or(Refusal::Damaged)?;
    let loca = table(tables, b"loca").ok_or(Refusal::Damaged)?;
    // What to keep: the asked-for glyphs that exist, glyph 0 (.notdef), and composite glyphs'
    // parts, followed a bounded number of levels.
    let mut kept: BTreeSet<u32> = keep.iter().copied().filter(|g| (*g as usize) < glyphs).collect();
    kept.insert(0);
    let mut fresh: Vec<u32> = kept.iter().copied().collect();
    for _ in 0..MAX_COMPONENT_DEPTH {
        let mut next = Vec::new();
        for gid in fresh {
            for part in glyph(glyf, loca, long, gid as usize).map(components).unwrap_or_default() {
                if (part as usize) < glyphs && kept.insert(part) {
                    next.push(part);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        fresh = next;
    }
    let mut new_glyf = Vec::new();
    let mut new_loca = Vec::with_capacity((glyphs + 1) * 4);
    for gid in 0..glyphs {
        let offset = u32::try_from(new_glyf.len()).map_err(|_| Refusal::TooLarge)?;
        new_loca.extend_from_slice(&offset.to_be_bytes());
        if kept.contains(&(gid as u32)) {
            new_glyf.extend_from_slice(glyph(glyf, loca, long, gid).ok_or(Refusal::Damaged)?);
            // Long offsets need no padding, but 4-byte alignment is what Word itself writes.
            while new_glyf.len() % 4 != 0 {
                new_glyf.push(0);
            }
        }
    }
    let end = u32::try_from(new_glyf.len()).map_err(|_| Refusal::TooLarge)?;
    new_loca.extend_from_slice(&end.to_be_bytes());
    *table_mut(tables, b"glyf").ok_or(Refusal::Damaged)? = new_glyf;
    *table_mut(tables, b"loca").ok_or(Refusal::Damaged)? = new_loca;
    let head = table_mut(tables, b"head").ok_or(Refusal::Damaged)?;
    let format = head.get_mut(50..52).ok_or(Refusal::Damaged)?;
    format.copy_from_slice(&1u16.to_be_bytes());
    Ok(())
}

/// The OpenType table checksum: the sum of the table's big-endian 32-bit words, zero-padded.
fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, c| {
        let mut word = [0u8; 4];
        word[..c.len()].copy_from_slice(c);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

/// A standalone TrueType file from `tables`: the table directory sorted by tag, each table
/// 4-byte aligned with its checksum, and `head.checksumAdjustment` set.
fn write_sfnt(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Result<Vec<u8>, Refusal> {
    tables.sort_by_key(|t| t.0);
    let n = u16::try_from(tables.len()).map_err(|_| Refusal::Damaged)?;
    if n == 0 {
        return Err(Refusal::Damaged);
    }
    // head.checksumAdjustment is computed with the field zeroed.
    if let Some(adj) = table_mut(&mut tables, b"head").and_then(|h| h.get_mut(8..12)) {
        adj.copy_from_slice(&[0; 4]);
    }
    let pow = 1u16 << (15 - n.leading_zeros().min(15));
    let search_range = pow.saturating_mul(16);
    let entry_selector = 15u16.saturating_sub(pow.leading_zeros() as u16);
    let range_shift = n.saturating_mul(16).saturating_sub(search_range);
    let mut out = Vec::new();
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    for v in [n, search_range, entry_selector, range_shift] {
        out.extend_from_slice(&v.to_be_bytes());
    }
    let mut offset = 12 + 16 * tables.len();
    let mut head_at = None;
    for (tag, data) in &tables {
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(data).to_be_bytes());
        out.extend_from_slice(&u32::try_from(offset).map_err(|_| Refusal::TooLarge)?.to_be_bytes());
        out.extend_from_slice(&u32::try_from(data.len()).map_err(|_| Refusal::TooLarge)?.to_be_bytes());
        if tag == b"head" {
            head_at = Some(offset);
        }
        offset += data.len().div_ceil(4) * 4;
    }
    for (_, data) in &tables {
        out.extend_from_slice(data);
        out.resize(out.len().div_ceil(4) * 4, 0);
    }
    if let Some(at) = head_at {
        let adjust = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        if let Some(field) = out.get_mut(at + 8..at + 12) {
            field.copy_from_slice(&adjust.to_be_bytes());
        }
    }
    if out.len() > MAX_FACE_BYTES {
        return Err(Refusal::TooLarge);
    }
    Ok(out)
}
