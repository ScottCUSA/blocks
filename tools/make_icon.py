#!/usr/bin/env python3
"""Render the app icon in the game's bevel block style to packaging/ and resources/."""

import pathlib

from PIL import Image, ImageDraw

ROOT = pathlib.Path(__file__).resolve().parent.parent
SIZE = 256
BLOCK = 40
GAP = 1
BEVEL = round(3.0 * BLOCK / 30.0)  # draw.rs BEVEL at BLOCK_SIZE 30
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

# rustomino.rs piece colors
COLORS = {
    "I": (0.0, 1.0, 1.0),
    "O": (1.0, 1.0, 0.0),
    "T": (0.72, 0.01, 0.99),
    "L": (1.0, 0.45, 0.03),
    "J": (0.09, 0.2, 1.0),
    "S": (0.4, 0.99, 0.0),
    "Z": (1.0, 0.06, 0.24),
}

LAYOUT = [
    "..L.S.",
    "..LSS.",
    "ILLST.",
    "IJZTTT",
    "IJZZOO",
    "IJJZOO",
]


def rgb(c):
    return tuple(round(v * 255) for v in c) + (255,)


def shade(c, amount):
    return tuple(v * amount for v in c)


def tint(c, amount):
    return tuple(v + (1.0 - v) * amount for v in c)


def rect(draw, x, y, w, h, color):
    draw.rectangle([x, y, x + w - 1, y + h - 1], fill=rgb(color))


def draw_block(draw, x, y, color):
    """draw.rs BlockStyle::Bevel"""
    rect(draw, x, y, BLOCK, BLOCK, shade(color, 0.55))
    rect(draw, x, y, BLOCK - BEVEL, BLOCK - BEVEL, tint(color, 0.45))
    face = BLOCK - 2 * BEVEL
    rect(draw, x + BEVEL, y + BEVEL, face, face, color)
    gloss = tint(color, 0.12)  # 12% white overlay
    rect(draw, x + BEVEL, y + BEVEL, face, round(face * 0.45), gloss)


def render():
    image = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    pitch = BLOCK + GAP
    span = len(LAYOUT[0]) * pitch - GAP
    left = (SIZE - span) // 2
    top = (SIZE - len(LAYOUT) * pitch + GAP) // 2
    for row, line in enumerate(LAYOUT):
        for col, piece in enumerate(line):
            if piece in COLORS:
                draw_block(draw, left + col * pitch, top + row * pitch, COLORS[piece])
    return image


def main():
    image = render()
    image.save(ROOT / "packaging" / "blocks.png", optimize=True)
    image.save(
        ROOT / "resources" / "blocks256x256.ico",
        sizes=[(s, s) for s in ICO_SIZES],
    )


if __name__ == "__main__":
    main()
