# Line breaks ignore zero-width markers · 2026-11-18

Corpus-breaks follow-up. Criteria, accepted, excluded and deferred rows are unchanged.

## Finding

zenodo-174 page 1 ended one line early because two lines of a paragraph started with "," and ".":
"…(Suárez-Balseiro et al., 2020)" | ", se convierte…". The citation is a Zotero field
(`fldChar begin` … `separate` … `end`), and the field's end sits between the ")" and the ",". In
Doc4Me a field boundary is a zero-width object character (U+FFFC) in the paragraph text, which UAX
#14 treats as a break opportunity on both sides, so the comma, which may not start a line, could.
Word ignores the field characters when it breaks lines.

`crates/layout/src/para.rs` now leaves zero-width markers (field, bookmark, comment, permission
and content-control boundaries; `InlineObject::is_marker`) out of the text it finds break
opportunities in, as it already did for hidden text; break positions map back to the paragraph as
before. Pictures, shapes and other drawn objects still break as before.

## Results (Linux, local)

438 source hashes verified, pinned prepared craft-fonts, release CLI (`56416f75…dc434c9`),
`cargo xtask readiness --corpus --dry-run --diagnostics --jobs 2`, at `1168b3b` plus this change:
open/round-trip/stability 219/219, page counts 218/219 (zenodo-019, as before), boundaries
**133 → 135/219**: **zenodo-166** and **zenodo-174** pass; nothing that passed fails. Every row:
[`field-marker-breaks-comparison-2026-11-18.tsv`](field-marker-breaks-comparison-2026-11-18.tsv).

Test: `a_field_end_between_a_bracket_and_a_comma_is_no_line_break` (fails with the change removed).
