#!/usr/bin/env python3
"""Add the two OFL fonts needed by the Windows pagination corpus to craft-fonts."""

from __future__ import annotations

import hashlib
import subprocess
import sys
import urllib.request
from pathlib import Path


CRAFT_FONTS_COMMIT = "c1db6d1a8ed772191241c3d185e2104be84d8691"
GOOGLE_FONTS_COMMIT = "cc39c4c2c304bc368c9b7c311ea2bcc6013299fe"
BASE = f"https://raw.githubusercontent.com/google/fonts/{GOOGLE_FONTS_COMMIT}/ofl"

FILES = (
    (
        "overpass-doc4me",
        "Overpass[wght].ttf",
        "overpass/Overpass%5Bwght%5D.ttf",
        "970717df17a7f9911dee45f60695d05bfa9d745fa0a11fc5c348371fa21f0073",
    ),
    (
        "overpass-doc4me",
        "Overpass-Italic[wght].ttf",
        "overpass/Overpass-Italic%5Bwght%5D.ttf",
        "3b45ac4193930bbeb491b5c76227afd5991607712f0d16d6bc5d74db6bbd8172",
    ),
    (
        "overpass-doc4me",
        "OFL.txt",
        "overpass/OFL.txt",
        "86e5ff25c701ec446d20b1a85b02ee6d36de8503a7288a4c948f5459809af1f0",
    ),
    (
        "noto-sans-ethiopic-doc4me",
        "NotoSansEthiopic[wdth,wght].ttf",
        "notosansethiopic/NotoSansEthiopic%5Bwdth%2Cwght%5D.ttf",
        "0dbccc00b22d180ebd6a4bd8a733918a29e709fa6798023adc3e6cd40da65077",
    ),
    (
        "noto-sans-ethiopic-doc4me",
        "OFL.txt",
        "notosansethiopic/OFL.txt",
        "72606b23f312cb25973958f2892d4d2c2012deabadbf0f763232624a9649fc69",
    ),
)

MANIFEST_ROWS = (
    "Overpass | Regular | fonts/overpass-doc4me/Overpass[wght].ttf | Latn | OFL-1.1 | fonts/overpass-doc4me/OFL.txt | 970717df17a7f9911dee45f60695d05bfa9d745fa0a11fc5c348371fa21f0073 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/overpass",
    "Overpass | Italic | fonts/overpass-doc4me/Overpass-Italic[wght].ttf | Latn | OFL-1.1 | fonts/overpass-doc4me/OFL.txt | 3b45ac4193930bbeb491b5c76227afd5991607712f0d16d6bc5d74db6bbd8172 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/overpass",
    "Noto Sans Ethiopic | Regular | fonts/noto-sans-ethiopic-doc4me/NotoSansEthiopic[wdth,wght].ttf | Ethi,Latn | OFL-1.1 | fonts/noto-sans-ethiopic-doc4me/OFL.txt | 0dbccc00b22d180ebd6a4bd8a733918a29e709fa6798023adc3e6cd40da65077 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/notosansethiopic",
)


def fetch(root: Path, family_dir: str, name: str, source: str, expected: str) -> None:
    path = root / "fonts" / family_dir / name
    if path.is_file():
        data = path.read_bytes()
    else:
        request = urllib.request.Request(f"{BASE}/{source}", headers={"User-Agent": "doc4me-font-build"})
        with urllib.request.urlopen(request, timeout=60) as response:
            data = response.read(8 * 1024 * 1024 + 1)
        if len(data) > 8 * 1024 * 1024:
            raise RuntimeError(f"download too large: {source}")
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        raise RuntimeError(f"SHA-256 mismatch for {source}: expected {expected}, got {actual}")
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.is_file():
        path.write_bytes(data)


def main() -> int:
    if len(sys.argv) != 2:
        raise RuntimeError("usage: prepare_corpus_fonts.py <craft-fonts-checkout>")
    root = Path(sys.argv[1]).resolve()
    manifest = root / "fonts" / "manifest.txt"
    if not manifest.is_file():
        raise RuntimeError(f"not a craft-fonts checkout: {root}")
    commit = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
    if commit != CRAFT_FONTS_COMMIT:
        raise RuntimeError(f"craft-fonts revision mismatch: expected {CRAFT_FONTS_COMMIT}, got {commit}")

    for family_dir, name, source, sha256 in FILES:
        fetch(root, family_dir, name, source, sha256)

    existing = manifest.read_text(encoding="utf-8").splitlines()
    for row in MANIFEST_ROWS:
        if row not in existing:
            existing.append(row)
    manifest.write_text("\n".join(existing) + "\n", encoding="utf-8", newline="\n")
    print(f"Prepared pinned OFL Overpass and Noto Sans Ethiopic in {root}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # keep workflow diagnostics concise and actionable
        print(f"prepare_corpus_fonts: {exc}", file=sys.stderr)
        raise SystemExit(1)
