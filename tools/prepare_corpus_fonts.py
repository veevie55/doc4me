#!/usr/bin/env python3
"""Add the OFL fonts needed by the Windows pagination corpus to craft-fonts."""

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
    ("montserrat-doc4me", "Montserrat-Italic[wght].ttf", "montserrat/Montserrat-Italic%5Bwght%5D.ttf", "51607f316bc020e59f03cbf51543eecffbea501c0b31d73e5b82927c5cca442c"),
    ("montserrat-doc4me", "Montserrat[wght].ttf", "montserrat/Montserrat%5Bwght%5D.ttf", "0f7b311b2f3279e4eef9b2f968bcdbab6e28f4daeb1f049f4f278a902bcd82f7"),
    ("montserrat-doc4me", "OFL.txt", "montserrat/OFL.txt", "8b7141c03fa4f8d44e6345d5d4931709290f0f67875e452e95ac1fd3a027802e"),
    ("source-sans-pro-original", "SourceSansPro-Regular.ttf", "https://raw.githubusercontent.com/adobe-fonts/source-sans/ce77773581f4d454f0fa985c073bb25c721bfcf5/TTF/SourceSansPro-Regular.ttf", "c6f835b6a6d942b196e2607bf73a7b086407ae8dbdc928c23a2fb4893ec383f6"),
    ("source-sans-pro-original", "SourceSansPro-Bold.ttf", "https://raw.githubusercontent.com/adobe-fonts/source-sans/ce77773581f4d454f0fa985c073bb25c721bfcf5/TTF/SourceSansPro-Bold.ttf", "8d869f2aeac0220984a430120af819fbcb12a88e8ea4ad72aae4c5f6d6a2b985"),
    ("source-sans-pro-original", "SourceSansPro-It.ttf", "https://raw.githubusercontent.com/adobe-fonts/source-sans/ce77773581f4d454f0fa985c073bb25c721bfcf5/TTF/SourceSansPro-It.ttf", "8fa3fe523cb385c2184e6691821c0431a5189a2a01a708a7b047847e15536b03"),
    ("source-sans-pro-original", "SourceSansPro-BoldIt.ttf", "https://raw.githubusercontent.com/adobe-fonts/source-sans/ce77773581f4d454f0fa985c073bb25c721bfcf5/TTF/SourceSansPro-BoldIt.ttf", "9cbd037fd721a3eea395675ecabaf10961d76198589a3e71a99a9c73e72ce384"),
    ("source-sans-pro-original", "OFL.txt", "https://raw.githubusercontent.com/adobe-fonts/source-sans/ce77773581f4d454f0fa985c073bb25c721bfcf5/LICENSE.txt", "e1d1fdcefb57985226c06fa9500d29caacfa2b1f8b8c74db911f13c6cad186a1"),
    ("roboto-doc4me", "Roboto[wdth,wght].ttf", "roboto/Roboto%5Bwdth%2Cwght%5D.ttf", "d7598e12c5dbef095ff8272cfc55da0250bd07fbdecbac8a530b9b277872a134"),
    ("roboto-doc4me", "Roboto-Italic[wdth,wght].ttf", "roboto/Roboto-Italic%5Bwdth%2Cwght%5D.ttf", "9725a847af6b460ffca162ae66d20dad48b01876137947180b42d7dcd7887182"),
    ("roboto-doc4me", "OFL.txt", "roboto/OFL.txt", "061402327a96aadb0bfb694a960ed289ecd38d383e396243831ab81feb109c41"),
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
    "Montserrat | Italic | fonts/montserrat-doc4me/Montserrat-Italic[wght].ttf | Latn | OFL-1.1 | fonts/montserrat-doc4me/OFL.txt | 51607f316bc020e59f03cbf51543eecffbea501c0b31d73e5b82927c5cca442c | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/montserrat",
    "Montserrat | Regular | fonts/montserrat-doc4me/Montserrat[wght].ttf | Latn | OFL-1.1 | fonts/montserrat-doc4me/OFL.txt | 0f7b311b2f3279e4eef9b2f968bcdbab6e28f4daeb1f049f4f278a902bcd82f7 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/montserrat",
    "Source Sans Pro | Regular | fonts/source-sans-pro-original/SourceSansPro-Regular.ttf | Latn | OFL-1.1 | fonts/source-sans-pro-original/OFL.txt | c6f835b6a6d942b196e2607bf73a7b086407ae8dbdc928c23a2fb4893ec383f6 | https://github.com/adobe-fonts/source-sans/tree/ce77773581f4d454f0fa985c073bb25c721bfcf5",
    "Source Sans Pro | Bold | fonts/source-sans-pro-original/SourceSansPro-Bold.ttf | Latn | OFL-1.1 | fonts/source-sans-pro-original/OFL.txt | 8d869f2aeac0220984a430120af819fbcb12a88e8ea4ad72aae4c5f6d6a2b985 | https://github.com/adobe-fonts/source-sans/tree/ce77773581f4d454f0fa985c073bb25c721bfcf5",
    "Source Sans Pro | Italic | fonts/source-sans-pro-original/SourceSansPro-It.ttf | Latn | OFL-1.1 | fonts/source-sans-pro-original/OFL.txt | 8fa3fe523cb385c2184e6691821c0431a5189a2a01a708a7b047847e15536b03 | https://github.com/adobe-fonts/source-sans/tree/ce77773581f4d454f0fa985c073bb25c721bfcf5",
    "Source Sans Pro | Bold Italic | fonts/source-sans-pro-original/SourceSansPro-BoldIt.ttf | Latn | OFL-1.1 | fonts/source-sans-pro-original/OFL.txt | 9cbd037fd721a3eea395675ecabaf10961d76198589a3e71a99a9c73e72ce384 | https://github.com/adobe-fonts/source-sans/tree/ce77773581f4d454f0fa985c073bb25c721bfcf5",
    "Roboto | Regular | fonts/roboto-doc4me/Roboto[wdth,wght].ttf | Latn | OFL-1.1 | fonts/roboto-doc4me/OFL.txt | d7598e12c5dbef095ff8272cfc55da0250bd07fbdecbac8a530b9b277872a134 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/roboto",
    "Roboto | Italic | fonts/roboto-doc4me/Roboto-Italic[wdth,wght].ttf | Latn | OFL-1.1 | fonts/roboto-doc4me/OFL.txt | 9725a847af6b460ffca162ae66d20dad48b01876137947180b42d7dcd7887182 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/roboto",
    "Overpass | Regular | fonts/overpass-doc4me/Overpass[wght].ttf | Latn | OFL-1.1 | fonts/overpass-doc4me/OFL.txt | 970717df17a7f9911dee45f60695d05bfa9d745fa0a11fc5c348371fa21f0073 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/overpass",
    "Overpass | Italic | fonts/overpass-doc4me/Overpass-Italic[wght].ttf | Latn | OFL-1.1 | fonts/overpass-doc4me/OFL.txt | 3b45ac4193930bbeb491b5c76227afd5991607712f0d16d6bc5d74db6bbd8172 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/overpass",
    "Noto Sans Ethiopic | Regular | fonts/noto-sans-ethiopic-doc4me/NotoSansEthiopic[wdth,wght].ttf | Ethi,Latn | OFL-1.1 | fonts/noto-sans-ethiopic-doc4me/OFL.txt | 0dbccc00b22d180ebd6a4bd8a733918a29e709fa6798023adc3e6cd40da65077 | https://github.com/google/fonts/tree/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/notosansethiopic",
)


def fetch(root: Path, family_dir: str, name: str, source: str, expected: str) -> None:
    path = root / "fonts" / family_dir / name
    if path.is_file():
        data = path.read_bytes()
    else:
        url = source if source.startswith("https://") else f"{BASE}/{source}"
        request = urllib.request.Request(url, headers={"User-Agent": "doc4me-font-build"})
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


def verify_manifest_fonts(root: Path) -> None:
    """Verify every declared file before the build script embeds it."""
    for line in (root / "fonts" / "manifest.txt").read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        fields = [part.strip() for part in line.split("|")]
        if len(fields) < 8:
            raise RuntimeError(f"malformed craft-fonts manifest row: {line}")
        family, style, relative, _scripts, license_id, license_path, expected, _source = fields[:8]
        font_path = root / relative
        actual = hashlib.sha256(font_path.read_bytes()).hexdigest()
        if actual != expected:
            raise RuntimeError(f"SHA-256 mismatch for {family} {style}: expected {expected}, got {actual}")
        if license_id.startswith("OFL"):
            notice = root / license_path
            if not notice.is_file() or b"SIL OPEN FONT LICENSE" not in notice.read_bytes()[:4096]:
                raise RuntimeError(f"missing or invalid OFL notice for {family} {style}: {license_path}")


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
    verify_manifest_fonts(root)
    print(f"Prepared pinned OFL Overpass, Noto Sans Ethiopic, Roboto, Montserrat and original Source Sans Pro in {root}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # keep workflow diagnostics concise and actionable
        print(f"prepare_corpus_fonts: {exc}", file=sys.stderr)
        raise SystemExit(1)
