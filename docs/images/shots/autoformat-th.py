#!/usr/bin/env python3
"""Compose docs/images/autoformat-th.png: the AutoFormat switch in three steps, with Thai captions.

Run from the repository root after the screenshots of autoformat-th.jsonl:

    cargo run -p doc4me-ui-egui --example ui_shot -- docs/images/shots/autoformat-th.jsonl
    python3 docs/images/shots/autoformat-th.py

It crops the title bar's switch and the page from target/autoformat-th-{1,2,3}.png (1440×900 pt
windows at 2×) and lays them out beside the captions, set in Sarabun from assets/fonts. Needs
Pillow with Raqm, which shapes the Thai text.
"""

import pathlib
import sys

from PIL import Image, ImageDraw, ImageFont, features

ROOT = pathlib.Path(__file__).resolve().parents[3]
SHOTS = [ROOT / f"target/autoformat-th-{i}.png" for i in (1, 2, 3)]
OUT = ROOT / "docs/images/autoformat-th.png"
FONTS = ROOT / "assets/fonts"

# Crops in screenshot pixels: the title bar from AutoSave to the switch, and the top of the page.
SWITCH = (140, 16, 620, 72)
KNOB = (528, 24, 592, 62)  # the switch itself, outlined in the picture
PAGE = (632, 560, 2264, 1060)
PAGE_SCALE = 0.62

STEPS = [
    (
        "ปิดสวิตช์ (ค่าเริ่มต้น)",
        [
            "ตัวเลขที่พิมพ์เป็นเลขอารบิกตามปกติ",
            "รายการลำดับนับ 1. 2. แบบเดิม",
        ],
    ),
    (
        "คลิกสวิตช์ “รูปแบบอัตโนมัติ” ให้เปิด",
        [
            "เลขอารบิกทั้งเอกสารเปลี่ยนเป็นเลขไทย",
            "ในครั้งเดียว รวมเลขรายการลำดับ เลขหน้า",
            "หัวกระดาษและท้ายกระดาษ",
            "เลิกทำ (Ctrl+Z) ได้ในขั้นเดียว",
        ],
    ),
    (
        "พิมพ์ต่อขณะเปิดสวิตช์",
        [
            "เลข 0–9 ที่พิมพ์กลายเป็น ๐–๙ ทันที",
            "รายการลำดับและเลขหน้าที่สร้างใหม่",
            "ก็ใช้เลขไทยด้วย",
        ],
    ),
]
TITLE = "สวิตช์รูปแบบอัตโนมัติ (AutoFormat) บนแถบชื่อเรื่อง"
SUBTITLE = "เปลี่ยนเลขอารบิกเป็นเลขไทย ทั้งตัวเลขที่มีอยู่ในเอกสารและตัวเลขที่พิมพ์ต่อจากนี้"
FOOTER = "ปิดสวิตช์อีกครั้ง: ไม่แปลงเลขกลับ ตัวเลขที่พิมพ์หลังจากนั้นเป็นเลขอารบิก · คีย์ลัด Alt, Z, F"

INK = (31, 35, 41)
DIM = (92, 99, 112)
ACCENT = (37, 99, 235)
MARK = (220, 38, 38)
CARD = (246, 247, 249)
LINE = (214, 218, 225)


def font(name: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(FONTS / name), size, layout_engine=ImageFont.Layout.RAQM)


def main() -> None:
    if not features.check("raqm"):
        sys.exit("Pillow was built without Raqm; it cannot shape the Thai captions")
    for shot in SHOTS:
        if not shot.is_file():
            sys.exit(f"{shot} is missing: run ui_shot on docs/images/shots/autoformat-th.jsonl first")
    title, sub, head, body, badge = (
        font("Sarabun-Bold.ttf", 46),
        font("Sarabun-Regular.ttf", 30),
        font("Sarabun-SemiBold.ttf", 34),
        font("Sarabun-Regular.ttf", 28),
        font("Sarabun-Bold.ttf", 30),
    )

    margin, gap, text_w = 56, 40, 640
    page_w = round((PAGE[2] - PAGE[0]) * PAGE_SCALE)
    page_h = round((PAGE[3] - PAGE[1]) * PAGE_SCALE)
    switch_w, switch_h = SWITCH[2] - SWITCH[0], SWITCH[3] - SWITCH[1]
    pad = 20
    card_w = page_w + 2 * pad
    card_h = switch_h + 16 + page_h + 2 * pad
    width = margin * 2 + text_w + gap + card_w
    header_h = 170
    row_h = card_h + 36
    footer_h = 90
    height = header_h + row_h * len(STEPS) + footer_h

    img = Image.new("RGB", (width, height), "white")
    d = ImageDraw.Draw(img)
    d.text((margin, 40), TITLE, font=title, fill=INK)
    d.text((margin, 104), SUBTITLE, font=sub, fill=DIM)

    for i, (shot, (heading, lines)) in enumerate(zip(SHOTS, STEPS)):
        top = header_h + i * row_h
        src = Image.open(shot).convert("RGB")
        # Caption: a numbered badge, the step and what it does.
        d.ellipse((margin, top + 6, margin + 48, top + 54), fill=ACCENT)
        d.text((margin + 24, top + 30), str(i + 1), font=badge, fill="white", anchor="mm")
        d.text((margin + 68, top + 8), heading, font=head, fill=INK)
        y = top + 68
        for line in lines:
            d.text((margin + 68, y), line, font=body, fill=DIM)
            y += 44
        # The picture: the switch above the page, on a light card.
        x0 = margin + text_w + gap
        d.rounded_rectangle((x0, top, x0 + card_w, top + card_h), radius=18, fill=CARD, outline=LINE, width=2)
        sx, sy = x0 + pad, top + pad
        img.paste(src.crop(SWITCH), (sx, sy))
        kx, ky = sx + KNOB[0] - SWITCH[0], sy + KNOB[1] - SWITCH[1]
        d.rounded_rectangle((kx - 6, ky - 6, kx + KNOB[2] - KNOB[0] + 6, ky + KNOB[3] - KNOB[1] + 6), radius=24, outline=MARK, width=4)
        d.text((sx + switch_w + 24, sy + switch_h // 2), "เปิดอยู่" if i else "ปิดอยู่", font=body, fill=MARK, anchor="lm")
        page = src.crop(PAGE).resize((page_w, page_h), Image.LANCZOS)
        py = sy + switch_h + 16
        img.paste(page, (sx, py))
        d.rectangle((sx, py, sx + page_w - 1, py + page_h - 1), outline=LINE, width=2)
        if i + 1 < len(STEPS):
            ax = margin + 24
            d.line((ax, y + 8, ax, top + row_h - 4), fill=LINE, width=4)

    d.text((margin, height - footer_h + 24), FOOTER, font=body, fill=DIM)
    img.save(OUT, optimize=True)
    print(OUT)


if __name__ == "__main__":
    main()
