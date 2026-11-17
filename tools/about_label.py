#!/usr/bin/env python3
"""Make an older Doc4Me source's About dialog show the version or the commit, as chosen.

    python3 tools/about_label.py SOURCE_DIR CATALOG_DIR

The build workflows (.github/workflows/build-*.yml) run this from their own revision on the
source they are about to build. Newer sources show the version or the commit natively
(`build_info::about_line` in crates/engine/src/build_info.rs, following DOC4ME_FILE_NAME); for them
this does nothing. For older ones it replaces the line(s) under the "Doc4Me" heading of the About
tab in crates/ui-egui/src/dialogs.rs with code that reads the same build variables
(DOC4ME_FILE_NAME, DOC4ME_VERSION, DOC4ME_BUILD_SHA), and adds the strings it uses to the source's
interface catalogs from CATALOG_DIR (the workflow revision's crates/ui-egui/src/i18n) where they
are missing, so the line is translated as in newer sources.

It stops with an error, rather than build an About dialog without the line, when the dialog code
does not look as expected.
"""

import pathlib
import re
import sys


# Bytes in and out: no newline translation on Windows, where a checkout may have CRLF lines.
def read(path: pathlib.Path) -> str:
    return path.read_bytes().decode("utf-8")


def write(path: pathlib.Path, text: str) -> None:
    path.write_bytes(text.encode("utf-8"))

# Present in sources that choose the About line themselves.
NATIVE = "pub fn about_line()"
KEYS = ("Version {version} ({build})", "Commit {commit}", "development build")
HEADING = re.compile(r'^(\s*)ui\.label\(egui::RichText::new\(tl!\("(?:Doc4Me|WordCraft)"\)\)\.font\(semibold\(22\.0\)\)\);\s*$')
BLURB = "A free, open-source word processor"
# What may stand between the heading and the blurb in older sources: the version line, the
# commit-only comment, or the `if let Some(version)` block around the version line.
REPLACEABLE = re.compile(r"^\s*(?://.*|\}|.*\b(?:build|version|Version)\b.*)?$")

BLOCK = """\
// Added by tools/about_label.py when this source was built: the version or the commit, following
// the build's file-name choice (DOC4ME_FILE_NAME), as newer sources do (build_info::about_line).
{
    let commit = option_env!("DOC4ME_BUILD_SHA")
        .map(str::trim)
        .filter(|s| (7..=64).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(|s| s.get(..8).unwrap_or(s));
    let version = option_env!("DOC4ME_VERSION").map(str::trim).map(|s| s.strip_prefix('v').unwrap_or(s)).filter(|s| !s.is_empty());
    match (option_env!("DOC4ME_FILE_NAME").map(str::trim), version, commit) {
        (Some("version"), Some(version), build) => {
            let build = build.unwrap_or(tl!("development build"));
            ui.label(crate::i18n::fmt(tl!("Version {version} ({build})"), &[("version", version), ("build", build)]));
        }
        (_, _, Some(commit)) => {
            ui.label(crate::i18n::fmt(tl!("Commit {commit}"), &[("commit", commit)]));
        }
        (_, _, None) => {
            ui.label(tl!("development build"));
        }
    }
}"""


def fail(message: str) -> None:
    print(f"::error::about_label.py: {message}", file=sys.stderr)
    sys.exit(1)


def patch_dialog(path: pathlib.Path) -> bool:
    text = read(path)
    if "Added by tools/about_label.py" in text:
        print(f"{path}: already patched")
        return False
    eol = "\r\n" if "\r\n" in text else "\n"
    lines = text.split("\n")
    headings = [i for i, line in enumerate(lines) if HEADING.match(line)]
    if len(headings) != 1:
        fail(f"{path}: expected one About heading, found {len(headings)}")
    start = headings[0]
    indent = HEADING.match(lines[start]).group(1)
    end = None
    for j in range(start + 1, min(start + 12, len(lines))):
        line = lines[j]
        if BLURB in line or (line.strip() == "ui.label(tl!(" and j + 1 < len(lines) and BLURB in lines[j + 1]):
            end = j
            break
    if end is None:
        fail(f"{path}: the About text after the heading was not found")
    old = lines[start + 1 : end]
    if len(old) > 8 or not all(REPLACEABLE.match(line.rstrip("\r")) for line in old):
        fail(f"{path}: unexpected code between the About heading and its text: {old!r}")
    cr = eol[:-1]
    block = [(indent + line if line else line) + cr for line in BLOCK.split("\n")]
    lines[start + 1 : end] = block
    write(path, "\n".join(lines))
    print(f"{path}: replaced {len(old)} line(s) under the About heading")
    return True


def source_of(line: str):
    """The (context, English source) of a catalog row, or None for comments and blank lines."""
    if not line or line.startswith("#"):
        return None
    cols = line.rstrip("\r").split("\t")
    return (cols[0], cols[1]) if len(cols) >= 3 else None


def add_strings(catalogs: pathlib.Path, reference: pathlib.Path) -> None:
    added = 0
    for path in sorted(catalogs.glob("*.tsv")):
        ref = reference / path.name
        if not ref.is_file():
            continue  # A language the workflow revision doesn't have: English is shown.
        text = read(path)
        eol = "\r\n" if "\r\n" in text else "\n"
        have = {source_of(line) for line in text.split("\n")}
        rows = {source_of(line): line.rstrip("\r") for line in read(ref).split("\n")}
        new = [rows[("", key)] for key in KEYS if ("", key) not in have and ("", key) in rows]
        if new:
            if text and not text.endswith("\n"):
                text += eol
            write(path, text + eol.join(new) + eol)
            added += len(new)
    print(f"{catalogs}: added {added} catalog row(s)")


def main() -> None:
    if len(sys.argv) != 3:
        fail("usage: about_label.py SOURCE_DIR CATALOG_DIR")
    source = pathlib.Path(sys.argv[1])
    reference = pathlib.Path(sys.argv[2])
    info = source / "crates/engine/src/build_info.rs"
    if info.is_file() and NATIVE in read(info):
        print("this source chooses the About line itself; nothing to change")
        return
    dialogs = source / "crates/ui-egui/src/dialogs.rs"
    if not dialogs.is_file():
        fail(f"{dialogs} not found")
    patch_dialog(dialogs)
    add_strings(source / "crates/ui-egui/src/i18n", reference)


if __name__ == "__main__":
    main()
