#!/usr/bin/env python3
"""Write cumulative GitHub release notes from the versioned CHANGELOG sections."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

LIMIT = 125_000
SECTION = re.compile(r"^## \[(\d+)\.(\d+)\.(\d+)\](?: - .*)?$", re.MULTILINE)


def sections(changelog: str, current: tuple[int, int, int]) -> list[tuple[tuple[int, int, int], str]]:
    matches = list(SECTION.finditer(changelog))
    result = []
    for index, match in enumerate(matches):
        version = tuple(map(int, match.groups()))
        if version > current:
            continue
        end = matches[index + 1].start() if index + 1 < len(matches) else len(changelog)
        body = changelog[match.start() : end]
        body = "\n".join(line for line in body.splitlines() if not re.match(r"\[[^\]]+\]: ", line)).strip()
        result.append((version, body))
    return sorted(result, key=lambda row: row[0], reverse=True)


def compact(version: tuple[int, int, int], body: str, detail_url: str) -> str:
    """Keep all release headings and top-level entry summaries when notes need shortening."""
    lines = body.splitlines()
    header = next((line for line in lines if line.startswith("## ")), f"## [{'.'.join(map(str, version))}]")
    kept = [f"{header} · [รายละเอียดทั้งหมดใน CHANGELOG]({detail_url})", ""]
    for line in lines[1:]:
        if line.startswith("### ") or (line.startswith("- ") and not line.startswith("-  ")):
            kept.append(line[:160])
    return "\n".join(kept).rstrip()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("version", help="release version or tag, e.g. v1.2.0")
    parser.add_argument("output", type=Path)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--server", default="https://github.com")
    parser.add_argument("--commit", required=True)
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    args = parser.parse_args()

    version_text = args.version.removeprefix("v")
    current = tuple(int(part) for part in version_text.split("."))
    if len(current) != 3:
        parser.error("version must be X.Y.Z or vX.Y.Z")
    found = sections(args.changelog.read_text(encoding="utf-8"), current)
    if not found or found[0][0] != current:
        parser.error(f"CHANGELOG.md has no section for {version_text}")

    full = "\n\n---\n\n".join(body for _, body in found)
    if len(full.encode("utf-8")) <= LIMIT:
        result = full
    else:
        result = "\n\n---\n\n".join(
            compact(version, body, f"{args.server}/{args.repo}/blob/{args.commit}/CHANGELOG.md#{anchor(body)}") for version, body in found
        )
        if len(result.encode("utf-8")) > LIMIT:
            raise SystemExit("cumulative release summaries exceed GitHub's 125,000-byte limit")
    args.output.write_text(result.rstrip() + "\n", encoding="utf-8")


def anchor(body: str) -> str:
    header = next((line[3:] for line in body.splitlines() if line.startswith("## ")), "")
    return re.sub(r"[^\w -]", "", header.lower()).strip().replace(" ", "-")


if __name__ == "__main__":
    main()
