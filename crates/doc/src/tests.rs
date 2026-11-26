use super::*;

mod note_conversion;

#[test]
fn new_document_is_valid() {
    let d = Document::new();
    assert_eq!(d.body.len(), 1);
    assert_eq!(d.para_paths(StoryRef::Body).len(), 1);
    assert_eq!(d.plain_text(StoryRef::Body), "");
    assert_eq!(d.sections().len(), 1);
}

#[test]
fn word_count() {
    let d = Document::from_text("Hello, world!\nThis is — a test 42.\n\n");
    assert_eq!(d.word_count(), 7);
    assert_eq!(count_words(" -- "), 0);
    // Thai is written without spaces between words: each dictionary word counts, as in Word.
    assert_eq!(count_words("ภาษาไทย"), 2);
    assert_eq!(count_words("ภาษาไทย English"), 3);
    assert_eq!(count_words("๑๒๓"), 1);
}

#[test]
fn sections_and_section_mut() {
    let mut d = Document::from_text("a\nb\nc");
    if let Some(Block::Para(p)) = d.body.get_mut(0).map(Arc::make_mut) {
        p.section = Some(Box::new(SectionProps { landscape: true, ..Default::default() }));
    }
    let s = d.sections();
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].0, 0);
    assert_eq!(d.section_index_of(0), 0);
    assert_eq!(d.section_index_of(2), 1);
    d.section_mut(0).margin_left = 10.0;
    d.section_mut(2).margin_left = 20.0;
    assert_eq!(d.sections()[0].1.margin_left, 10.0);
    assert_eq!(d.last_section.margin_left, 20.0);
}

#[test]
fn parts_and_media() {
    let mut d = Document::new();
    let id = d.add_part(PartKind::Header, Vec::new());
    assert_eq!(d.story(StoryRef::Part(id)).map(|b| b.len()), Some(1));
    let k1 = d.add_media(vec![1, 2, 3], "png");
    let k2 = d.add_media(vec![1, 2, 3], "png");
    let k3 = d.add_media(vec![4], "png");
    assert_eq!(k1, k2);
    assert_ne!(k1, k3);
}

#[test]
fn clone_is_shallow() {
    let d = Document::from_text("a\nb");
    let mut e = d.clone();
    assert!(Arc::ptr_eq(&d.body[0], &e.body[0]));
    e.insert_text(&Pos::body(1, 0), "x", &CharProps::default()).unwrap();
    assert!(Arc::ptr_eq(&d.body[0], &e.body[0]));
    assert!(!Arc::ptr_eq(&d.body[1], &e.body[1]));
    assert_eq!(d.plain_text(StoryRef::Body), "a\nb");
}

#[test]
fn json_round_trip() {
    let mut d = Document::from_text("Hello\nWorld");
    d.format_range(&Pos::body(0, 0), &Pos::body(0, 5), &|c| c.bold = Some(true)).unwrap();
    let j = serde_json::to_string(&d).unwrap();
    let back: Document = serde_json::from_str(&j).unwrap();
    assert_eq!(back.plain_text(StoryRef::Body), "Hello\nWorld");
    assert_eq!(back.para(StoryRef::Body, &Path::top(0)).unwrap().runs[0].props.bold, Some(true));
}

#[test]
fn ensure_nonempty_repairs() {
    let mut d = Document::new();
    d.body.clear();
    d.body.push(Arc::new(Block::Table(Table::new(1, 1, 100.0))));
    d.ensure_nonempty();
    assert!(matches!(d.body.last().map(|b| &**b), Some(Block::Para(_))));
}

#[test]
fn text_box_anchor_finds_the_owning_shape() {
    let mut d = Document::from_text("Before after");
    let id = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("Inside", CharProps::default()))]);
    let shape = |story| InlineObject::Shape {
        kind: para::ShapeKind::TextBox,
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
    };
    // A plain shape before it doesn't count.
    d.insert_object(&Pos::body(0, 0), shape(None), &CharProps::default()).unwrap();
    let end = d.insert_object(&Pos::body(0, 10), shape(Some(id)), &CharProps::default()).unwrap();
    assert_eq!(d.text_box_anchor(id), Some(end));
    assert_eq!(d.text_box_anchor(id + 1), None);
}

#[test]
fn word_count_including_notes_counts_used_boxes_and_notes_only() {
    let mut d = Document::from_text("one two");
    let shape = |story| InlineObject::Shape {
        kind: para::ShapeKind::TextBox,
        w: 144.0,
        h: 72.0,
        fill: None,
        stroke: None,
        stroke_width: 0.75,
        float: Default::default(),
        story: Some(story),
        freeform: None,
        effects: Default::default(),
        extra: Default::default(),
    };
    let para_of = |t: &str| vec![para_block(Paragraph::with_text(t, CharProps::default()))];
    let boxed = d.add_part(PartKind::TextBox, para_of("three four"));
    let nested = d.add_part(PartKind::TextBox, para_of("five"));
    let note = d.add_part(PartKind::Footnote, para_of("six seven"));
    // Not counted: an orphaned box (deleted or cut), a header, a comment.
    d.add_part(PartKind::TextBox, para_of("orphan words here"));
    d.add_part(PartKind::Header, para_of("header words"));
    d.add_part(PartKind::Comment, para_of("comment words"));
    d.insert_object(&Pos::body(0, 3), shape(boxed), &CharProps::default()).unwrap();
    d.insert_object(&Pos { story: StoryRef::Part(boxed), path: Path::top(0), off: 0 }, shape(nested), &CharProps::default()).unwrap();
    d.insert_object(
        &Pos::body(0, 0),
        InlineObject::NoteRef { kind: para::NoteKind::Footnote, id: note, custom: String::new() },
        &CharProps::default(),
    )
    .unwrap();
    // A box pointing at a header doesn't make the header count.
    let hdr = d.parts.iter().find(|(_, p)| p.kind == PartKind::Header).map(|(k, _)| *k).unwrap();
    d.insert_object(&Pos::body(0, 0), shape(hdr), &CharProps::default()).unwrap();
    assert_eq!(d.word_count(), 2);
    assert_eq!(d.word_count_including_notes(), 7);
    assert_eq!(d.counted_stories(), vec![StoryRef::Body, StoryRef::Part(note), StoryRef::Part(boxed), StoryRef::Part(nested)]);
}

#[test]
fn counted_stories_survive_self_nested_boxes() {
    let mut d = Document::from_text("a");
    let id = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("b", CharProps::default()))]);
    let shape = InlineObject::Shape {
        kind: para::ShapeKind::TextBox,
        w: 10.0,
        h: 10.0,
        fill: None,
        stroke: None,
        stroke_width: 0.0,
        float: Default::default(),
        story: Some(id),
        freeform: None,
        effects: Default::default(),
        extra: Default::default(),
    };
    d.insert_object(&Pos::body(0, 0), shape.clone(), &CharProps::default()).unwrap();
    d.insert_object(&Pos { story: StoryRef::Part(id), path: Path::top(0), off: 0 }, shape, &CharProps::default()).unwrap();
    assert_eq!(d.word_count_including_notes(), 2);
}

#[test]
fn prune_text_boxes_drops_only_unshown_ones() {
    let shape = |story| InlineObject::Shape {
        kind: para::ShapeKind::TextBox,
        w: 50.0,
        h: 20.0,
        fill: None,
        stroke: None,
        stroke_width: 0.0,
        float: Default::default(),
        story: Some(story),
        freeform: None,
        effects: Default::default(),
        extra: Default::default(),
    };
    let para_of = |t: &str| vec![para_block(Paragraph::with_text(t, CharProps::default()))];
    let mut d = Document::from_text("body");
    assert_eq!(d.prune_text_boxes(), 0, "nothing to do without text boxes");
    let shown = d.add_part(PartKind::TextBox, para_of("shown"));
    let nested = d.add_part(PartKind::TextBox, para_of("inside shown"));
    let orphan = d.add_part(PartKind::TextBox, para_of("deleted box"));
    let orphan_child = d.add_part(PartKind::TextBox, para_of("inside the deleted box"));
    let hdr = d.add_part(PartKind::Header, para_of("header"));
    let in_header = d.add_part(PartKind::TextBox, para_of("box in the header"));
    d.insert_object(&Pos::body(0, 0), shape(shown), &CharProps::default()).unwrap();
    let at = |id| Pos { story: StoryRef::Part(id), path: Path::top(0), off: 0 };
    d.insert_object(&at(shown), shape(nested), &CharProps::default()).unwrap();
    d.insert_object(&at(orphan), shape(orphan_child), &CharProps::default()).unwrap();
    d.insert_object(&at(hdr), shape(in_header), &CharProps::default()).unwrap();
    assert_eq!(d.prune_text_boxes(), 2);
    let kept: Vec<u32> = d.parts.keys().copied().collect();
    assert_eq!(kept, vec![shown, nested, hdr, in_header]);
    assert_eq!(d.prune_text_boxes(), 0);
}

/// `blocks` in the only cell of `levels` tables, each nested in the next.
fn nested_tables(levels: usize, blocks: Blocks) -> Blocks {
    (0..levels).fold(blocks, |inner, _| {
        let mut t = Table::new(1, 1, 100.0);
        t.rows[0].cells[0].blocks = inner;
        vec![Arc::new(Block::Table(t))]
    })
}

/// A paragraph showing text box story `story` (drawing id `id`).
fn box_para(story: u32, id: u32) -> Blocks {
    let shape = InlineObject::Shape {
        kind: para::ShapeKind::TextBox,
        w: 50.0,
        h: 20.0,
        fill: None,
        stroke: None,
        stroke_width: 0.0,
        float: Default::default(),
        story: Some(story),
        freeform: None,
        effects: Default::default(),
        extra: connector::ShapeExtra { id, ..Default::default() },
    };
    let mut p = Paragraph::with_text("cell", CharProps::default());
    p.insert_object(0, shape, &CharProps::default()).unwrap();
    vec![para_block(p)]
}

#[test]
fn text_boxes_shown_from_deeply_nested_tables_are_kept() {
    // #1339: the paragraph walk stopped 16 tables down, so a box shown only from deeper was taken
    // for unshown and the next edit pruned its story.
    let para_of = |t: &str| vec![para_block(Paragraph::with_text(t, CharProps::default()))];
    let mut d = Document::from_text("body");
    let at_17 = d.add_part(PartKind::TextBox, para_of("17 tables down"));
    let at_40 = d.add_part(PartKind::TextBox, para_of("40 tables down"));
    let inner = d.add_part(PartKind::TextBox, para_of("deep in a box"));
    let outer = d.add_part(PartKind::TextBox, nested_tables(18, box_para(inner, 0)));
    let in_header = d.add_part(PartKind::TextBox, para_of("deep in a header"));
    let hdr = d.add_part(PartKind::Header, nested_tables(20, box_para(in_header, 0)));
    let unshown = d.add_part(PartKind::TextBox, para_of("shown nowhere"));
    d.body.extend(nested_tables(17, box_para(at_17, 0)));
    d.body.extend(nested_tables(40, box_para(at_40, 7)));
    d.body.extend(box_para(outer, 0));
    assert_eq!(d.prune_text_boxes(), 1);
    assert!(!d.parts.contains_key(&unshown));
    let kept: Vec<u32> = d.parts.keys().copied().collect();
    assert_eq!(kept, vec![at_17, at_40, inner, outer, in_header, hdr]);
    assert_eq!(d.prune_text_boxes(), 0);
    // Counted in reading order, the nested box after its own; new drawing ids above the deep one.
    let part = StoryRef::Part;
    assert_eq!(d.counted_stories(), vec![StoryRef::Body, part(at_17), part(at_40), part(outer), part(inner)]);
    assert_eq!(d.next_shape_id(), 8);
}

#[test]
fn a_deeply_nested_table_is_copied_with_its_text_boxes() {
    // A copy takes the box 17 tables down along, so pasting it into another document shows its
    // text, not the other document's box that happens to have the same id.
    let para_of = |t: &str| vec![para_block(Paragraph::with_text(t, CharProps::default()))];
    let mut d = Document::from_text("body");
    let deep = d.add_part(PartKind::TextBox, para_of("deep box"));
    d.body.extend(nested_tables(17, box_para(deep, 0)));
    let frag = d.body_fragment();
    assert_eq!(frag.parts.keys().copied().collect::<Vec<_>>(), vec![deep]);
    let mut e = Document::new();
    let own = e.add_part(PartKind::TextBox, para_of("its own box"));
    assert_eq!(own, deep, "the same id in both documents");
    e.body = box_para(own, 0);
    let end = e.end_of(StoryRef::Body);
    e.insert_fragment(&end, &frag).unwrap();
    let mut shown = Vec::new();
    e.objects_in_reading_order(&mut |o| shown.extend(o.text_box()));
    let texts: Vec<String> = shown.iter().map(|id| e.plain_text(StoryRef::Part(*id))).collect();
    assert_eq!(texts, vec!["its own box", "deep box"]);
}

#[test]
fn notes_in_text_box_follow_nested_boxes_once() {
    let shape = |story| InlineObject::Shape {
        kind: para::ShapeKind::TextBox,
        w: 50.0,
        h: 20.0,
        fill: None,
        stroke: None,
        stroke_width: 0.0,
        float: Default::default(),
        story: Some(story),
        freeform: None,
        effects: Default::default(),
        extra: Default::default(),
    };
    let note = |id| InlineObject::NoteRef { kind: para::NoteKind::Footnote, id, custom: String::new() };
    let mut d = Document::new();
    let outer = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::new())]);
    let inner = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::new())]);
    let (n1, n2) = (d.add_part(PartKind::Footnote, Vec::new()), d.add_part(PartKind::Footnote, Vec::new()));
    let at = |id| Pos { story: StoryRef::Part(id), path: Path::top(0), off: 0 };
    // outer: [note n1][box inner][box outer (itself)]; inner: [note n2]
    d.insert_object(&at(outer), shape(outer), &CharProps::default()).unwrap();
    d.insert_object(&at(outer), shape(inner), &CharProps::default()).unwrap();
    d.insert_object(&at(outer), note(n1), &CharProps::default()).unwrap();
    d.insert_object(&at(inner), note(n2), &CharProps::default()).unwrap();
    assert_eq!(d.notes_in_text_box(outer), vec![n1, n2]);
    assert_eq!(d.notes_in_text_box(inner), vec![n2]);
    assert!(d.notes_in_text_box(n1).is_empty(), "not a text box");
}

fn fs(instr: &str) -> InlineObject {
    InlineObject::FieldStart { instr: instr.into(), locked: false }
}

#[test]
fn field_ranges_pair_across_paragraphs_and_nest() {
    let mut d = Document::from_text("See .\nBib one\nBib two");
    let c = CharProps::default();
    // Citation in paragraph 0: "See ␣[cite](Smith 2020)␣." with a nested field inside.
    let p0 = d.para_mut(StoryRef::Body, &Path::top(0)).unwrap();
    p0.insert_object(4, fs("ADDIN ZOTERO_ITEM CSL_CITATION {}"), &c).unwrap();
    let n = p0.insert_text(7, "(Smith 2020)", &c).unwrap();
    p0.insert_object(7 + n, InlineObject::FieldEnd, &c).unwrap();
    p0.insert_object(7, fs("ADDIN inner"), &c).unwrap();
    p0.insert_object(10, InlineObject::FieldEnd, &c).unwrap();
    // Bibliography from paragraph 1 to the end of paragraph 2.
    d.para_mut(StoryRef::Body, &Path::top(1)).unwrap().insert_object(0, fs("ADDIN ZOTERO_BIBL {} CSL_BIBLIOGRAPHY"), &c).unwrap();
    let p2 = d.para_mut(StoryRef::Body, &Path::top(2)).unwrap();
    let end = p2.len();
    p2.insert_object(end, InlineObject::FieldEnd, &c).unwrap();

    let r = d.field_ranges(StoryRef::Body);
    assert_eq!(r.len(), 3);
    assert!(r[0].instr.starts_with("ADDIN ZOTERO_ITEM"));
    assert_eq!((r[0].start.off, r[0].depth), (4, 0));
    assert_eq!((r[1].instr.as_str(), r[1].depth), ("ADDIN inner", 1));
    assert_eq!(r[2].start.path, Path::top(1));
    assert_eq!(r[2].end.path, Path::top(2));
    assert!(!d.has_unbalanced_field_ranges());
    assert_eq!(d.balance_field_ranges(), 0);
    // Markers contribute no text.
    assert_eq!(d.plain_text(StoryRef::Body), "See (Smith 2020).\nBib one\nBib two");
}

#[test]
fn orphan_field_markers_are_removed() {
    let mut d = Document::from_text("ab\ncd");
    let c = CharProps::default();
    let p0 = d.para_mut(StoryRef::Body, &Path::top(0)).unwrap();
    p0.insert_object(0, InlineObject::FieldEnd, &c).unwrap();
    // OBJ is three bytes: "␣a[x]b]" → start after "␣a", end after "b".
    p0.insert_object(4, fs("ADDIN x"), &c).unwrap();
    p0.insert_object(8, InlineObject::FieldEnd, &c).unwrap();
    d.para_mut(StoryRef::Body, &Path::top(1)).unwrap().insert_object(1, fs("ADDIN open"), &c).unwrap();
    assert!(d.has_unbalanced_field_ranges());
    assert_eq!(d.balance_field_ranges(), 2);
    assert!(!d.has_unbalanced_field_ranges());
    let r = d.field_ranges(StoryRef::Body);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].instr, "ADDIN x");
    assert_eq!(d.plain_text(StoryRef::Body), "ab\ncd");
}

#[test]
fn custom_props_set_get_remove() {
    let mut d = Document::new();
    assert_eq!(d.custom_prop("ZOTERO_PREF_1"), None);
    d.set_custom_prop("ZOTERO_PREF_1", "<data/>");
    d.set_custom_prop("Other", "x");
    d.set_custom_prop("zotero_pref_1", "<data2/>");
    assert_eq!(d.custom_props.len(), 2);
    assert_eq!(d.custom_props[0].name, "ZOTERO_PREF_1");
    assert_eq!(d.custom_prop("Zotero_Pref_1"), Some("<data2/>"));
    assert!(d.remove_custom_prop("other"));
    assert!(!d.remove_custom_prop("other"));
    assert_eq!(d.custom_props.len(), 1);
}

#[test]
fn complex_script_values_follow_the_latin_ones_until_export() {
    // Built-in styles set no complex-script values: Thai (and Arabic …) text takes the Latin
    // font and size, so a run made 36 pt by any command shows its Thai at 36 pt too.
    let doc = Document::new();
    assert!(doc.styles.default_chr.font_cs.is_none() && doc.styles.styles.iter().all(|s| s.chr.size_cs.is_none()));
    let run = CharProps { size: Some(36.0), ..Default::default() };
    let rc = doc.styles.resolve_char(Some("Heading1"), &run);
    assert_eq!((rc.size, rc.size_cs), (36.0, 36.0));
    assert_eq!(rc.font_cs, "Sarabun");
    // For Word, which inherits them on their own, export spells them out where the Latin ones are.
    assert!(doc.complex_script_falls_back());
    let mut out = Document::from_text("ภาษาไทย");
    if let Some(Block::Para(p)) = out.body.get_mut(0).map(std::sync::Arc::make_mut) {
        p.runs = vec![crate::para::Run { len: p.text.len(), props: run.clone() }];
    }
    out.make_complex_script_explicit();
    assert_eq!(out.styles.default_chr.font_cs.as_deref(), Some("Sarabun"));
    assert_eq!(out.styles.default_chr.size_cs, Some(12.0));
    let h1 = out.styles.get("Heading1").map(|s| s.chr.clone()).unwrap_or_default();
    assert_eq!((h1.size_cs, h1.font_cs.as_deref()), (h1.size, h1.font.as_deref()));
    assert_eq!(out.para(StoryRef::Body, &Path(vec![0])).map(|p| p.runs[0].props.size_cs), Some(Some(36.0)));
    assert!(!out.complex_script_falls_back());
    // A document read from a .docx keeps the file's, even where they fall back.
    let mut read = Document::new();
    read.complex_script_from_file = true;
    assert!(!read.complex_script_falls_back());
    let before = read.clone();
    read.make_complex_script_explicit();
    assert_eq!(read, before);
    // So does one whose styles set them.
    let mut word = Document::new();
    let chr = &mut Arc::make_mut(&mut word.styles).default_chr;
    chr.font_cs = Some("TH SarabunPSK".into());
    chr.size_cs = Some(16.0);
    chr.bold_cs = Some(false);
    chr.italic_cs = Some(false);
    let before = word.clone();
    assert!(!word.complex_script_falls_back());
    word.make_complex_script_explicit();
    assert_eq!(word, before);
}

#[test]
fn many_distinct_media_add_in_linear_time() {
    // An import with tens of thousands of small distinct images (same length, differing only at
    // the end) used to compare each against every earlier one.
    let mut d = Document::new();
    let t = std::time::Instant::now();
    let n = 40_000u32;
    for i in 0..n {
        let mut bytes = vec![7u8; 256];
        bytes.extend(i.to_le_bytes());
        d.add_media(bytes, "png");
    }
    assert_eq!(d.media.len(), n as usize);
    // Duplicates still find their first copy, including media inserted directly.
    let mut first = vec![7u8; 256];
    first.extend(0u32.to_le_bytes());
    assert_eq!(d.add_media(first, "png"), "image1.png");
    d.media.insert("direct.png".into(), Arc::new(vec![9, 9]));
    assert_eq!(d.add_media(vec![9, 9], "png"), "direct.png");
    assert!(t.elapsed().as_secs_f32() < 2.0, "{:?}", t.elapsed());
    // The lookup cache isn't part of the document.
    let mut e = d.clone();
    e.media_index = MediaIndex::default();
    assert_eq!(d, e);
}

#[test]
fn media_keys_skip_taken_names_once() {
    let mut d = Document::new();
    // A file whose media already use the names add_media would pick next.
    for i in 2..2002u32 {
        d.media.insert(format!("image{i}.png"), Arc::new(i.to_le_bytes().to_vec()));
    }
    assert_eq!(d.add_media(vec![0], "png"), "image2002.png");
    assert_eq!(d.add_media(vec![1], "png"), "image2003.png");
    assert_eq!(d.add_media(vec![2], "png"), "image2004.png");
}

/// A paragraph that is nothing but `objs` (built directly: inserting one at a time is slow too).
fn para_of_objects(objs: Vec<InlineObject>) -> Paragraph {
    let mut p = Paragraph::new();
    p.text = para::OBJ.to_string().repeat(objs.len());
    p.runs = vec![Run { len: p.text.len(), props: CharProps::default() }];
    p.objects = objs;
    p
}

#[test]
fn object_walks_are_linear_in_the_paragraph() {
    // #3614: loops of object_offsets + object_at recounted the objects before each one, so a
    // paragraph of 40,000 markers took seconds per walk.
    let n = 40_000u32;
    let w = para::OBJ.len_utf8();
    let mut objs: Vec<InlineObject> = Vec::new();
    for i in 0..n {
        objs.push(InlineObject::BookmarkStart { name: format!("b{i}") });
        objs.push(fs("ADDIN x"));
        objs.push(InlineObject::PermStart { id: i, group: "everyone".into(), editor: String::new() });
    }
    // Ranges close in the order they opened (the worst case for a backwards search).
    for i in 0..n {
        objs.push(InlineObject::FieldEnd);
        objs.push(InlineObject::PermEnd { id: i });
    }
    let mut d = Document::new();
    d.body = vec![Arc::new(Block::Para(para_of_objects(objs)))];
    let t = std::time::Instant::now();
    assert_eq!(d.bookmarks().len(), n as usize);
    assert_eq!(d.field_ranges(StoryRef::Body).len(), n as usize);
    let perms = d.perm_ranges();
    assert_eq!(perms.len(), n as usize);
    assert!(perms.iter().all(|r| r.end.off > r.start.off), "every range closed by its own end");
    assert_eq!(d.text_box_anchor(7), None);
    // A caret before a long row of field ends moves past all of them.
    let first_end = 3 * n as usize * w;
    let p = d.past_field_ends(&Pos::body(0, first_end));
    assert_eq!(p.off, first_end + w);
    assert!(t.elapsed().as_secs_f32() < 2.0, "{:?}", t.elapsed());

    let ends: Vec<InlineObject> = (0..n).map(|_| InlineObject::FieldEnd).collect();
    d.body = vec![Arc::new(Block::Para(para_of_objects(ends)))];
    let t = std::time::Instant::now();
    assert_eq!(d.past_field_ends(&Pos::body(0, 0)).off, n as usize * w);
    // Typing at the end of a row of block-level control starts.
    let block = ContentControl { block: true, ..Default::default() };
    let starts: Vec<InlineObject> = (0..n).map(|_| InlineObject::ControlStart { control: Box::new(block.clone()) }).collect();
    d.body = vec![Arc::new(Block::Para(para_of_objects(starts)))];
    let end = Pos::body(0, n as usize * w);
    assert_eq!(d.typing_pos(&end), end);
    assert!(t.elapsed().as_secs_f32() < 2.0, "{:?}", t.elapsed());
}

#[test]
fn deleting_across_many_lone_control_markers_is_not_quadratic() {
    let n = 1_500;
    let starts: Vec<InlineObject> = (0..n).map(|_| InlineObject::ControlStart { control: Box::default() }).collect();
    let mut d = Document::from_text("before\n\nafter");
    d.body[1] = Arc::new(Block::Para(para_of_objects(starts)));
    let t = std::time::Instant::now();
    let at = d.delete_range(&Pos::body(0, 3), &Pos::body(2, 2)).unwrap();
    assert_eq!(at, Pos::body(0, 3));
    assert_eq!(d.plain_text(StoryRef::Body), "befter");
    assert!(t.elapsed().as_secs_f32() < 2.0, "{:?}", t.elapsed());
}
#[test]
fn drawing_stacking_order_survives_json_and_defaults_for_older_documents() {
    use crate::para::{Float, Wrap};
    let old: Float = serde_json::from_str(r#"{"wrap":"inFrontOfText"}"#).unwrap();
    assert_eq!(old.z_order, 0);
    for z_order in [0, 251_660_288, u32::MAX] {
        let original = Float { z_order, wrap: Wrap::BehindText, ..Default::default() };
        let restored: Float = serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
        assert_eq!(restored, original);
    }
}
