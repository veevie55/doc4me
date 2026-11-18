# A section's vertical alignment on its last page · 2026-11-18

Corpus-breaks follow-up. Criteria, accepted, excluded and deferred rows are unchanged.

zenodo-081's first section is a title page with `w:vAlign w:val="bottom"`: one paragraph, and two
text boxes positioned 240.9 pt and 258.1 pt above it ("SÉRIE BALADO • ÉPISODE 3" and the title).
Word puts the paragraph at the foot of the page, so the boxes sit in its lower half (title lines
at y = 448–672 pt). Doc4Me already moved a page's content for its section's vertical alignment
when the page ended, but a page ended by the next section's page break was aligned with the
**next** section's setting (top here): the layout switched to the new section before closing the
page. The paragraph stayed at the top and the boxes ran off the page above it; only "Ressources
en appui" showed.

`crates/layout/src/lib.rs`: when a section starts a new page, the page the previous section ends
on is aligned with that section's setting first (and not again when the new page opens).

Test: `a_bottom_aligned_section_keeps_its_alignment_when_the_next_starts_a_page` (fails without the
change).

Linux corpus (438 hashes verified, pinned craft-fonts, release CLI): open/round-trip/stability
219/219, pages 218/219, boundaries 138 → **139/219** (zenodo-081), no other row changes; matching
lines 106,603 → 106,611. Rows: [`section-valign-comparison-2026-11-18.tsv`](section-valign-comparison-2026-11-18.tsv).
