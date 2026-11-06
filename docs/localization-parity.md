# Localization parity with Microsoft Word

> **Last reviewed:** 2026-11-06 · **Last updated:** 2026-11-06 · **Change:** minor (current registry has 69 interface choices including English; the latest Windows readiness report is 68/95 culture rows, while the current source maps 72/95 pending report refresh; key-row coverage does not measure translation accuracy; Hindi, Arabic and Korean remain unregistered among the twelve key languages; bundled Noto fallbacks cover their glyph scripts but RTL/bidi layout remains unsupported) · **Target:** Microsoft Word (Microsoft 365) for Windows

Interface languages, script support and proofing per language. How translation works (catalog
format, adding a language, clean-room rule): `crates/ui-egui/src/i18n/mod.rs`.

**Dimension: ~27% (estimated), 75–120 h to full for the twelve key languages**, plus work for the
other languages Word ships and native-speaker review (human). This estimate is separate from the
readiness check: the latest official Windows report (run 42211564189) records 68/95 culture rows
passing the catalog key-row coverage rule; the current source maps 72/95 rows, and the next report
has not yet verified that updated result. The check does not distinguish translated values from
English-identical values and does not assess meaning or native-speaker quality. Bundled OFL Noto
fallbacks now cover the missing scripts (including a Hangul subset); epaint's lack of bidi and RTL
layout still prevents claiming Arabic or Hebrew interface readiness.

## What Word ships (measured by listing the bundle)

- **30 interface languages** (31 `.lproj` folders incl. Base): ar, cs, da, de, el, en, en-GB, es,
  es-MX, fi, fr, fr-CA, he, hu, id, it, ja, ko, nl, no, pl, pt (BR), pt-PT, ru, sk, sv, th, tr,
  zh-CN, zh-TW. Arabic and Hebrew get a mirrored interface. No Hindi or Vietnamese interface on Mac.
- **120 proofing tools** (`SharedSupport/Proofing Tools`): spellers for ~50 languages, grammar
  checkers for ~20, hyphenators for ~16, thesauri for ~20.

## What Doc4Me ships (measured)

- **69 interface choices** are registered in `crates/ui-egui/src/i18n/mod.rs`: English source plus
  68 TSV catalogs. One TSV is the English UK catalog (`en-gb`); the other 67 are non-English
  language catalogs, including Spanish, French, Indonesian, Vietnamese, European Portuguese,
  Norwegian Bokmål and Nynorsk. Four registered drafts (Igbo, Setswana, Wolof and Yoruba) are
  AI-generated and require native-speaker review; key presence counts them without measuring their
  quality. The 95-row readiness catalog maps cultures to these files: the latest official result is
  68/95, while the current source maps 72/95 pending report refresh. `LANGUAGES` is authoritative for
  choices and locale matching; `languages.tsv` is the separate readiness denominator.
- `cargo xtask readiness` parses unique catalog keys and compares key sets, not non-English
  values. These key counts cannot be described as the share of strings
  translated. Some values remain English, and the readiness result is not a translation-quality
  score. Native-speaker review and script/UI verification remain open.
- `every_tab_and_command_is_translated` checks key presence for ribbon tabs, command labels and
  ribbon locations; it does not establish translation accuracy.
- **Proofing: English, Polish and Thai** (Thai: spelling only), chosen per run from its language (`w:lang`, resolved through
  the styles; Review › Language and the status bar set it). Languages without proofing tools get
  no spelling or grammar marks (as in Word without the language's tools) instead of being flagged
  as English. Detail in [Proofing per language](#proofing-per-language).
- Document scripts: shaping via HarfRust (HarfBuzz port) for every script; bidi (UAX #9) and Arabic
  shaping tested (#207); IME on all desktop platforms (winit fixes pending, #155–#164).

## The twelve key languages

The table reports catalog key rows. “English-identical” counts values exactly equal to their
English source; this is mechanical, not a quality score (some shared terms and proper names may be
intentional). No non-English catalog has native-speaker review recorded here. `full` requires
translation, correct rendering/editing and review; `partial` means work remains. Hours are
estimates, excluding the shared items below.

| Language | Code | Parsed unique keys / values exactly equal to English source | Dialogs / tooltips / help | Script support | Native review | Status | Hours to full |
|---|---|---|---|---|---|---|---|
| English | en | English source (3,031 keys) | ✅ | Latin ✅ | yes | **full** | 0 |
| Simplified Chinese (Mandarin) | zh-hans | 3,031 keys; 76 values exactly equal English source | most; newer panes English | CJK ✅, IME ✅; Windows UI font fixed (#248); interface font chain covers every catalog character, installed CJK font after the embedded ones when it doesn't, PingFang found on macOS 12 (#486); font list by Chinese names (宋体, 微软雅黑, #505); no vertical text | no | partial | 3–5 |
| Spanish | es | 3,031 keys; 105 values exactly equal English source | most | Latin ✅ | no | partial | 1–2 |
| Hindi | hi | no catalog | ❌ | Devanagari shaping via HarfRust, untested; no hyphenation | no | none | 4–6 |
| Arabic | ar | no catalog | ❌ | shaping, joining, lam-alef, bidi ✅ (#207); no kashida; **no mirrored UI** | no | none | 6–8 |
| French | fr | 3,031 keys; 158 values exactly equal English source | most; newer dialogs partly English | Latin ✅ | no | partial | 1–2 |
| Portuguese | pt-br / pt-pt | 3,031 keys each; 113 / 110 values exactly equal English source | most | Latin ✅ | no | partial | 2–4 (terminology and review) |
| Indonesian | id | 3,031 keys; 149 values exactly equal English source | ❌ | Latin ✅ | no | partial | 3–4 (terminology and review) |
| Japanese | ja | 3,031 keys; 77 values exactly equal English source | most | CJK ✅, IME ✅; no vertical text, ruby or document grid | no | partial | 3–5 |
| German | de | 3,031 keys; 148 values exactly equal English source | most | Latin ✅ | no | partial | 1–2 |
| Korean | ko | no catalog | ❌ | Hangul ✅; macOS Korean IME fixes pending (#163, #164) | no | none | 3–5 |
| Vietnamese | vi | 3,031 keys; 101 values exactly equal English source | ❌ | Latin with stacked diacritics, untested | no | partial | 3–4 (script/UI verification and review) |

Outside those key-language rows, the registry has **59 catalog files**; this includes the English UK catalog. The eight non-English key-language rows that have catalogs use nine files because Portuguese has both `pt-br` and `pt-pt`. Registry membership and catalog key coverage do not guarantee translation quality; details should be checked against the source catalogs and human review.
Estonian uses the existing fonts, follows `et-EE` system locales, and is available in Options or
through `ui.language` with `{"value":"et"}`. The readiness parser sees 3,031 unique keys; translation accuracy
is not measured by readiness.
Estonian spelling, grammar, dates and document templates are not added by the interface catalog.
Turkish works the same way (`tr-TR` and `tr-CY` system locales, `{"value":"tr"}`) with 3,031
unique parsed keys, the same key set as the Russian catalog. Command search folds the Turkish dotted and dotless I (`italik` finds İtalik, `YAZI TİPİ`
finds Yazı Tipi) while English labels and ASCII command ids keep their usual matching. Turkish
proofing is not included; terminology and checks: [`localization-tr.md`](localization-tr.md).
Hungarian follows `hu-HU` system locales and `{"value":"hu"}`, with 3,031 unique parsed keys matching
the Russian catalog, including built-in style and citation style names. Hungarian proofing is not
included; translation accuracy is not measured by readiness. Terminology and conventions:
[`localization-hu.md`](localization-hu.md).

Polish and Thai are the only ones besides English with proofing tools (Thai: spelling only), and a
Polish interface writes new documents in Polish (`pl-PL`).

**Thai (`th`)**: an interface catalog (`th.tsv`, 3,031 unique keys as parsed by readiness across ribbon, dialogs, messages, built-in style names and equation gallery), chosen by `th`, `th-TH` and `th_TH.UTF-8` system locales or in File › Options; written from a 903-term glossary and a style guide (commands are verbs, labels nouns, Royal Society loanword spelling, ฟอนต์ for Font, one space between Thai and Latin or digits, Arabic digits in fields). New documents keep US English as their editing language, since Thai text is proofed whatever its language. Not yet reviewed by a native-speaking professional translator. Home › Font has Convert to Thai Digits and Convert to Arabic Digits (0–9 ↔ ๐–๙ in the selection; with Track Changes on, a revision). Thai documents: Sarabun (OFL, bundled) is the default
document and interface font, so Thai draws the same everywhere, shaped through HarfRust with its
mark stacking (`layout::tests_complex::thai_is_drawn_with_the_bundled_sarabun_by_default`). Thai,
Lao and Khmer lines wrap between dictionary words (ICU4X, `doc::cluster::dictionary_breaks`), and
double-click, Ctrl+arrows and the word count use the same word boundaries; tone marks and ์ stay
inside their word. TH Sarabun New and TH SarabunPSK are drawn with Sarabun when not
installed, scaled to their letter height and set on their own advance widths, kerning and 1.331 em
line spacing, so lines and pages break as with the real font (`fonts::thsarabun`, measured from
the font files); the other Thai document fonts are drawn with Sarabun at its own size, and when an
opened document uses TH SarabunIT๙ and the computer doesn't have it, a notice says the
document is shown with Sarabun (`cmd::file::unmatched_fonts`). Single line spacing follows Word
for Windows (OS/2 win metrics plus external leading: Sarabun 1.853 em). Thai digits, ฿ and
Thai/Khmer punctuation stay with the word before them (no line starts with ๒ or ។). A document made
here is saved to .docx with its complex-script font, size and weight spelled out (`w:cs`, `w:szCs`,
`w:bCs`, `w:iCs`), so Word shows its Thai text the same; a .docx from Word keeps its own. List
labels draw Thai digits (๑ ๒ ๓), letters (ก ข ค ง, without ฃ ฅ ฆ) and counting words (หนึ่ง สอง …
สิบเอ็ด ยี่สิบเอ็ด … หนึ่งร้อยเอ็ด), all three offered in the Numbering menu and Define New
Multilevel List.
Thai Distributed alignment spreads full lines between characters. Text boxes in the interface
leave room for tone marks raised over upper vowels (ที่, น้ำ). Thai proofing marks only what is
certainly wrong (see the table below).

## Proofing per language

| Language | Spelling | Suggestions | Grammar | Hyphenation |
|---|---|---|---|---|
| English (`en`, any region; text with no language) | Moby word list (public domain, ~160k words) with inflection rules | edit distance | repeated word, `a`/`an`, sentence capital, space before punctuation, double space | Moby dictionary, our Liang patterns, heuristic |
| Polish (`pl`) | The Hunspell `pl_PL` dictionary installed on the system (OS package, Homebrew, LibreOffice, or the `DOC4ME_DICTIONARIES` folder; not bundled, so no Polish spelling marks without it), read at runtime by a Hunspell-style affix engine (`crates/proof/src/affix.rs`); names capitalized, compounds part by part, `Kennedy'ego` | missing diacritics first (`zolw` → `żółw`), the dictionary's confusion table (`żeka` → `rzeka`, `chuśtawka` → `huśtawka`), missing space (`napewno` → `na pewno`), one typo, then a stem scan within two typos | Polish messages: repeated word, space before punctuation, double space, sentence capital (knows `np.`, `r.`, initials, ordinals, dialogue dashes), missing comma before `że`/`który`/`ponieważ`/`gdyż`/`aby`/`żeby`/`jeśli`/`jeżeli`/`gdyby`/`lecz` (not after `mimo`, `chyba`, `tak`, prepositions…), `we`/`ze` (`we wtorek`, `ze szkoły`), doubled punctuation | TeX Polish patterns (Kołodziejska, Jackowski, Ryćko; MIT option) with their 20 exception words; a Polish syllable heuristic for letters outside the alphabet |
| Thai (any Thai text, whatever the run's language) | Precision first: characters typed in an order no Thai word has (เเ for แ, ํา for ำ, a tone mark or ์ before the vowel, a tone mark after ำ, a doubled mark, two tone marks) and common misspellings marked only when ICU4X's dictionary sees them as a whole word (`assets/spelling/th-misspellings.tsv`: อนุญาติ, กระเพรา, อีเมล์…); unknown words are never marked | the word typed in order; the Royal Society spelling | — | — (Thai isn't hyphenated; lines wrap between words) |
| Any other | — (no marks) | — | — | the English patterns, as before |

Measured with the SJP.PL dictionary: every one of the ~4.5 million forms an independent expansion of its affix file
produces is accepted, and 200,000 one-edit mutations that aren't words are all rejected; a check
takes ~1 µs, a suggestion list 1–35 ms (release build). Not yet: other languages' dictionaries,
a thesaurus for Polish, Word's style and clarity refinements, Polish-specific AutoCorrect.

Italian (`it`, #594) follows `it`, `it-IT` and `it-CH` system locales. Its catalog has every key of
the others plus 51 strings that are wrapped in `tl!` but were in no catalog yet (File › Info
properties, the File › Home greeting, list levels, Equation tab gallery and tooltips). Ribbon tabs,
groups and command names use the names Italian Word users know (feature names, as `AGENTS.md`
allows); all other text is written from the meaning of the English. No Italian proofing yet.
## Shared work (not in the per-language hours)

| Item | Hours | Notes |
|---|---|---|
| Wrap the remaining English literals in `tl!` and translate them in every catalog | 4–6 | Makes the six partial languages full except script items |
| Mirrored (right-to-left) interface for Arabic and Hebrew | 15–25 | Ribbon, panes, dialogs, rulers |
| Vertical text, ruby, document grid (Chinese, Japanese, Korean) | counted in [layout-parity.md](layout-parity.md) | |
| Proofing dictionaries, grammar and hyphenation for the 11 non-English key languages | 20–40 | Open dictionaries only (licence check per dictionary); issues #25, #40, #100. Per-run languages, the Hunspell-style affix engine and the TeX pattern reader landed with Polish, so a language with an open Hunspell dictionary and hyphenation patterns is mostly data plus grammar rules |
| Native-speaker review of every catalog | human | None reviewed yet |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-25 | trivial | TH SarabunIT๙ no longer reuses TH Sarabun New's metrics: Sarabun draws it at its own size and opening the document says so |
| 2026-10-19 | minor | Thai interface catalog (2,433 entries, complete); Thai misspelling list grown to 105 pairs; review fixes (Thai/Latin words split for spelling, user dictionary clears typing-order marks) |
| 2026-10-16 | minor | Thai proofing (typing-order mistakes and common misspellings, Royal Society spellings suggested), Thai counting list numbers and Thai list kinds in the Numbering menu, TH Sarabun New/PSK/IT๙ drawn with Sarabun at their own size, widths and line spacing, Word for Windows single line spacing |
| 2026-10-15 | minor | Thai: digits and punctuation stay with their word, Thai letters skip ฆ (ก ข ค ง), complex-script values spelled out only when saving a document made here, interface text boxes keep raised tone marks |
| 2026-10-14 | minor | Thai documents: bundled Sarabun as the default font, dictionary word breaks for Thai, Lao and Khmer (lines, selection, word count), Thai list numbers, complex-script defaults for Word |
| 2026-10-11 | minor | Turkish interface catalog completed (2,325 entries, the same keys as Russian): equation gallery, built-in style names, watermarks, paper sizes and preview tiles; the interface audit checks Turkish too (#774) |
| 2026-10-11 | minor | Czech (`cs`) interface catalog added |
| 2026-10-11 | minor | Polish interface (13 languages) and Polish proofing: SJP.PL spelling with suggestions, Polish grammar rules, TeX Polish hyphenation; proofing language per run; dimension ~25% → ~27% |
| 2026-10-11 | minor | German interface catalog (#574): 2,021 entries covering the full zh-hans key set; locale, saved-preference and bundled-font tests; no German proofing |
| 2026-10-11 | minor | Italian interface catalog (#594), locale selection, saved preference and font coverage checks; count labels put the noun first so they read right at 1 |
| 2026-10-11 | minor | Font dialog sets the Asian text font (`w:eastAsia`) and the Latin text font separately; the Asian list holds the installed Chinese, Japanese and Korean fonts by their interface-language names (#505) |
| 2026-10-11 | minor | French interface catalog (2,033 entries, @pechnikov2026-star), `fr-*` system locales, saved preference and font coverage checks; no French proofing |
| 2026-10-11 | minor | Font list shows installed fonts by their names in the interface language (宋体 for SimSun in Chinese, Japanese and Korean names likewise) and finds them by either name; a Chinese, Japanese or Korean font is stored as `w:eastAsia` too (#505) |
| 2026-10-11 | minor | Chinese/Japanese interface fonts: a test checks every catalog character has a glyph; an installed CJK font follows the embedded faces whenever they miss one; macOS font asset folders searched for PingFang (#486) |
| 2026-10-10 | minor | Russian interface (#300; 1,524 entries, built-in style names shown translated; covers everything the interface audit finds) |
| 2026-10-11 | minor | Hungarian interface catalog (2,419 entries, built-in style names shown translated; covers everything the interface audit finds), locale selection, saved preference and font coverage checks |
| 2026-10-11 | minor | Turkish interface catalog (1,307 entries), locale selection, saved preference, Turkish casing in command search and font coverage checks; catalog sizes re-counted |
| 2026-10-11 | trivial | English spelling (#520): our supplement `assets/spelling/en-extra.txt` adds ~220 common modern words the Moby list lacks ("desktop", "download", "email"…) |
| 2026-10-10 | minor | Estonian interface catalog, locale selection, saved preference and font coverage checks; count labels put the noun first so they read right at 1 |
| 2026-10-10 | minor | Serbian (Cyrillic and Latin) interface merged (#250) |
| 2026-10-10 | major | First version: twelve-language table, catalog coverage measured, Word's 30 UI languages and 120 proofing tools listed |
