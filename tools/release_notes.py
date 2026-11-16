#!/usr/bin/env python3
"""Write cumulative GitHub release notes from the released CHANGELOG sections.

Doc4Me names releases by the commit they were built from: a release's section is headed
`## [<commit>] - YYYY-MM-DD` (short commit). Sections released before that change keep their
version headings (`## [1.1.4] - ...`); they are earlier releases all the same. The notes are the
chosen release's section followed by every earlier released section, newest first, in the order
CHANGELOG.md lists them (`[Unreleased]` is never included).

Usage: release_notes.py <commit> <output> --repo owner/name --commit <full sha>
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path

LIMIT = 125_000
SECTION = re.compile(r"^## \[([^\]]+)\](?: - .*)?$", re.MULTILINE)
COMMIT = re.compile(r"[0-9a-f]{7,40}")


def sections(changelog: str) -> list[tuple[str, str]]:
    """Every released section as (heading id, body), in file order (newest first)."""
    matches = list(SECTION.finditer(changelog))
    result = []
    for index, match in enumerate(matches):
        if match.group(1).lower() == "unreleased":
            continue
        end = matches[index + 1].start() if index + 1 < len(matches) else len(changelog)
        body = changelog[match.start() : end]
        body = "\n".join(line for line in body.splitlines() if not re.match(r"\[[^\]]+\]: ", line)).strip()
        result.append((match.group(1), body))
    return result


def matches(heading: str, commit: str) -> bool:
    """A heading names this release when one commit is a prefix of the other (short or full)."""
    heading = heading.lower()
    return bool(COMMIT.fullmatch(heading)) and (heading.startswith(commit) or commit.startswith(heading))


def compact(heading: str, body: str, detail_url: str) -> str:
    """Keep all release headings and top-level entry summaries when notes need shortening."""
    lines = body.splitlines()
    header = next((line for line in lines if line.startswith("## ")), f"## [{heading}]")
    kept = [f"{header} · [รายละเอียดทั้งหมดใน CHANGELOG]({detail_url})", ""]
    for line in lines[1:]:
        if line.startswith("### ") or (line.startswith("- ") and not line.startswith("-  ")):
            kept.append(line[:160])
    return "\n".join(kept).rstrip()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("release", help="the release's commit (short or full), as in its CHANGELOG heading")
    parser.add_argument("output", type=Path)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--server", default="https://github.com")
    parser.add_argument("--commit", required=True, help="full commit SHA the CHANGELOG links point at")
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    args = parser.parse_args()

    release = args.release.lower()
    if not COMMIT.fullmatch(release):
        parser.error("release must be a commit SHA (7 to 40 hex digits)")
    if not COMMIT.fullmatch(args.commit.lower()):
        parser.error("--commit must be a commit SHA")
    found = sections(args.changelog.read_text(encoding="utf-8"))
    start = next((i for i, (heading, _) in enumerate(found) if matches(heading, release)), None)
    if start is None:
        parser.error(f"CHANGELOG.md has no section headed [{release}]")
    found = found[start:]

    full = "\n\n---\n\n".join(body for _, body in found)
    if len(full.encode("utf-8")) <= LIMIT:
        result = full
    else:
        result = "\n\n---\n\n".join(
            compact(heading, body, f"{args.server}/{args.repo}/blob/{args.commit}/CHANGELOG.md#{anchor(body)}") for heading, body in found
        )
        if len(result.encode("utf-8")) > LIMIT:
            raise SystemExit("cumulative release summaries exceed GitHub's 125,000-byte limit")
    args.output.write_text(result.rstrip() + "\n", encoding="utf-8")


def anchor(body: str) -> str:
    header = next((line[3:] for line in body.splitlines() if line.startswith("## ")), "")
    return re.sub(r"[^\w -]", "", header.lower()).strip().replace(" ", "-")


if __name__ == "__main__":
    main()
