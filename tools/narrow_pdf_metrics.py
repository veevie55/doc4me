#!/usr/bin/env python3
"""Reproduce width facts from the pinned corpus PDF; never read embedded font programs.

Requires pypdf. This handles the simple-font bfchar CMaps actually present in this reference;
it rejects changed inputs and unsupported CMap forms instead of guessing character widths.
"""
from pathlib import Path
import hashlib
import re

from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'corpus/readiness/zenodo-149.pdf'
SHA256 = '852febe91286b70651dee95dfb3fa6e57397e7fa56688c528b8bd244fa514752'


def write_if_changed(path, text):
    if not path.exists() or path.read_text(encoding='utf-8') != text:
        path.write_text(text, encoding='utf-8')


def main():
    if hashlib.sha256(SOURCE.read_bytes()).hexdigest() != SHA256:
        raise ValueError('reference PDF does not match corpus manifest')
    widths = {'NOVA_COND_LIGHT_REGULAR': {}, 'NOVA_COND_LIGHT_ITALIC': {}, 'ARIAL_NARROW_REGULAR': {}, 'ARIAL_NARROW_BOLD': {}, 'TAHOMA_REGULAR': {}, 'TAHOMA_BOLD': {}, 'GOTHAM_BOOK': {}, 'GOTHAM_MEDIUM': {}, 'NYALA': {}}
    import csv
    manifest = {r['id']: r for r in csv.DictReader((line for line in (ROOT / 'docs/readiness/corpus.tsv').read_text().splitlines() if not line.startswith('#')), delimiter='\t')}
    sources = [SOURCE, ROOT / 'corpus/readiness/zenodo-095.pdf', ROOT / 'corpus/readiness/govuk-068.pdf'] + [ROOT / f'corpus/readiness/{id}.pdf' for id in ['govuk-038', 'govuk-043', 'govuk-067']]
    for source in sources:
        if hashlib.sha256(source.read_bytes()).hexdigest() != manifest[source.stem]['pdf_sha256']:
            raise ValueError('reference PDF does not match manifest')
    ambiguous = set()
    ambiguities = []
    for page in (page for source in sources for page in PdfReader(source).pages):
        for ref in page['/Resources'].get('/Font', {}).values():
            font = ref.get_object()
            name = str(font.get('/BaseFont'))
            if '/ToUnicode' not in font:
                continue
            if 'ArialNovaCond-Light' in name:
                key = 'NOVA_COND_LIGHT_ITALIC' if 'Italic' in name else 'NOVA_COND_LIGHT_REGULAR'
            elif 'ArialNarrow' in name and 'Italic' not in name:
                key = 'ARIAL_NARROW_BOLD' if 'Bold' in name else 'ARIAL_NARROW_REGULAR'
            elif 'Gotham-Book' in name:
                key = 'GOTHAM_BOOK'
            elif 'Gotham-Medium' in name:
                key = 'GOTHAM_MEDIUM'
            elif 'Nyala-Regular' in name:
                key = 'NYALA'
            elif 'Tahoma' in name and 'Italic' not in name:
                key = 'TAHOMA_BOLD' if 'Bold' in name else 'TAHOMA_REGULAR'
            else:
                continue
            target = widths[key]
            if '/Widths' in font:
                first, advances = int(font['/FirstChar']), font['/Widths']
                declared = {first + index: int(width) for index, width in enumerate(advances)}
            elif key == 'NYALA' and '/DescendantFonts' in font:
                if font.get('/Encoding') != '/Identity-H':
                    raise ValueError('unsupported CID encoding')
                descendant = font['/DescendantFonts'][0].get_object()
                entries, declared, index = descendant.get('/W', []), {}, 0
                while index < len(entries):
                    start, value = int(entries[index]), entries[index + 1]
                    if isinstance(value, list):
                        declared.update((start + offset, int(width)) for offset, width in enumerate(value))
                        index += 2
                    else:
                        end, width = int(value), int(entries[index + 2])
                        if not 0 <= start <= end <= 65535:
                            raise ValueError('CID range outside bounds')
                        declared.update((cid, width) for cid in range(start, end + 1))
                        index += 3
            else:
                continue
            cmap = font['/ToUnicode'].get_object().get_data().decode('ascii')
            if 'beginbfrange' in cmap:
                raise ValueError('unsupported CMap range')
            for block in re.findall(r'beginbfchar(.*?)endbfchar', cmap, re.S):
                for code, uni in re.findall(r'<([\da-fA-F]+)>\s*<([\da-fA-F]+)>', block):
                    char = bytes.fromhex(uni).decode('utf-16-be')
                    cid = int(code, 16)
                    if len(char) != 1 or cid not in declared:
                        continue
                    width = declared[cid]
                    if not 0 <= width <= 65535:
                        raise ValueError('width outside table bounds')
                    if (key, char) in ambiguous:
                        continue
                    if char in target and target[char] != width:
                        if key != 'NYALA':
                            raise ValueError(f'conflicting width measurements: {key} {char!r}: {target[char]} vs {width} in {name}')
                        ambiguities.append((key, f'U+{ord(char):04X}', target[char], width))
                        ambiguous.add((key, char))
                        del target[char]
                        continue
                    target[char] = width
    ambiguity_audit = 'family\tcodepoint\tfirst_width\tconflicting_width\n' + ''.join('\t'.join(map(str, row)) + '\n' for row in ambiguities)
    write_if_changed(ROOT / 'docs/readiness/font-width-ambiguities-2026-11-15.tsv', ambiguity_audit)
    output = (
        '//! Advance widths declared in zenodo-149 reference PDF /Widths + /ToUnicode.\n'
        f'//! Source SHA-256: {SHA256}\n'
        '//! Tahoma trial widths are recorded separately in docs/readiness/tahoma-width-audit-2026-11-15.tsv.\n'
        '//! Gotham Book/Medium width facts additionally use the manifest-pinned zenodo-095 PDF.\n'
        '//! Nyala advance facts additionally use the manifest-pinned govuk-068 PDF.\n'
        '//! Public PDF layout numbers (1000 units/em); no font program or outlines copied.\n'
        '//! Arial Nova Cond Light regular/italic and Arial Narrow regular/bold. Missing widths retain fallback advances.\n'
    )
    for name, chars in widths.items():
        if not chars:
            raise ValueError('reference has no matching widths')
        if name.startswith('TAHOMA'):
            continue
        output += f'pub(crate) const {name}: &[(char, u16)] = &[\n'
        for char, width in sorted(chars.items()):
            output += f"    ('\\u{{{ord(char):x}}}', {width}),\n"
        output += '];\n'
    write_if_changed(ROOT / 'crates/fonts/src/latin_metrics.rs', output)
    audit = 'style\tcodepoint\tadvance_per_1000em\n'
    for name, chars in widths.items():
        if name.startswith('TAHOMA'):
            audit += ''.join(f'{name}\tU+{ord(char):04X}\t{width}\n' for char, width in sorted(chars.items()))
    write_if_changed(ROOT / 'docs/readiness/tahoma-width-audit-2026-11-15.tsv', audit)


if __name__ == '__main__':
    main()
