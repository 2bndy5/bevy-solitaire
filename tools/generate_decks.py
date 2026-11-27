#!/usr/bin/env python3
"""Generates the extra card decks and card backs in `assets/decks` and `assets/backs`.

Everything is drawn from vector shapes (suits are parametric curves, not font glyphs), so the only
input is the Ubuntu font already in `assets/fonts`. Cards are drawn at twice their final size and
scaled down for smooth edges.

Usage (from the repository root; needs Pillow):

    python3 tools/generate_decks.py

The output is committed, so this only needs re-running after changing it.
"""

from __future__ import annotations

import math
import random
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FONT = ROOT / "assets" / "fonts" / "UbuntuNerdFont-Medium.ttf"

# Final card size: the same 7:10 shape as the card meshes.
WIDTH, HEIGHT = 420, 600
SCALE = 2  # drawn at this multiple, then scaled down
W, H = WIDTH * SCALE, HEIGHT * SCALE
RADIUS = 30 * SCALE  # corner radius
PAPER = (251, 250, 246, 255)
EDGE = (190, 186, 176, 255)

RANKS = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"]
SUITS = {"c": "Clubs", "d": "Diamonds", "h": "Hearts", "s": "Spades"}

RED = (200, 16, 46)
BLACK = (26, 26, 30)
# For dark cards: light enough to read on charcoal, still clearly two different colors.
CORAL = (255, 112, 104)
ICE = (222, 230, 240)
DARK_PAPER = (38, 40, 46, 255)
DARK_EDGE = (78, 82, 92, 255)


@dataclass
class Deck:
    name: str
    # Two colors, as in a traditional deck: one for hearts and diamonds, one for clubs and spades.
    red: tuple[int, int, int]
    black: tuple[int, int, int]
    large_print: bool = False
    paper: tuple[int, int, int, int] = PAPER
    edge: tuple[int, int, int, int] = EDGE

    def color(self, suit: str) -> tuple[int, int, int]:
        return self.red if suit in "hd" else self.black


DECKS = [
    Deck("modern", RED, BLACK),
    Deck("large-print", RED, BLACK, large_print=True),
    Deck("modern-dark", CORAL, ICE, paper=DARK_PAPER, edge=DARK_EDGE),
    Deck("large-print-dark", CORAL, ICE, large_print=True, paper=DARK_PAPER, edge=DARK_EDGE),
]


# --- Suit shapes ---------------------------------------------------------------------------------
# Each returns a list of polygons (lists of points) in a unit box: x and y in [-1, 1], y down.


def heart(steps: int = 120) -> list[list[tuple[float, float]]]:
    points = []
    for i in range(steps):
        t = 2 * math.pi * i / steps
        x = 16 * math.sin(t) ** 3
        y = 13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t)
        points.append((x / 17, -(y + 2.5) / 15))
    return [points]


def diamond(steps: int = 30) -> list[list[tuple[float, float]]]:
    # A rhombus with gently inward-curved sides.
    corners = [(0, -1), (0.78, 0), (0, 1), (-0.78, 0)]
    points = []
    for (x0, y0), (x1, y1) in zip(corners, corners[1:] + corners[:1]):
        for i in range(steps):
            t = i / steps
            bow = 0.08 * math.sin(math.pi * t)
            x = x0 + (x1 - x0) * t
            y = y0 + (y1 - y0) * t
            # Pull toward the center for the curve.
            x -= bow * (1 if x > 0 else -1) * (abs(x) > 1e-6)
            y -= bow * (1 if y > 0 else -1) * (abs(y) > 1e-6)
            points.append((x, y))
    return [points]


def circle(cx: float, cy: float, r: float, steps: int = 60) -> list[tuple[float, float]]:
    return [
        (cx + r * math.cos(2 * math.pi * i / steps), cy + r * math.sin(2 * math.pi * i / steps))
        for i in range(steps)
    ]


def stem(top: float) -> list[tuple[float, float]]:
    # A flared stem from y = top down to the bottom of the box.
    return [(-0.07, top), (0.07, top), (0.16, 0.8), (0.42, 1.0), (-0.42, 1.0), (-0.16, 0.8)]


def spade() -> list[list[tuple[float, float]]]:
    body = [(x * 1.0, -y * 0.92 - 0.12) for x, y in heart()[0]]
    return [body, stem(0.2)]


def club() -> list[list[tuple[float, float]]]:
    r = 0.36
    return [
        circle(0, -0.5, r),
        circle(-0.46, 0.08, r),
        circle(0.46, 0.08, r),
        circle(0, -0.05, 0.22),
        stem(0.0),
    ]


SHAPES = {"h": heart, "d": diamond, "s": spade, "c": club}


def draw_suit(draw: ImageDraw.ImageDraw, suit: str, cx: float, cy: float, size: float,
              color, flip: bool = False) -> None:
    """Draws `suit` centered on (cx, cy), `size` pixels tall, upside down if `flip`."""
    half = size / 2
    for polygon in SHAPES[suit]():
        draw.polygon(
            [(cx + x * half, cy + (-y if flip else y) * half) for x, y in polygon],
            fill=color,
        )


# --- Card faces ----------------------------------------------------------------------------------


def blank_card(paper=PAPER, edge=EDGE) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    draw.rounded_rectangle((0, 0, W - 1, H - 1), RADIUS, fill=edge)
    border = 3 * SCALE
    draw.rounded_rectangle((border, border, W - 1 - border, H - 1 - border), RADIUS - border,
                           fill=paper)
    return image, draw


def font(size: float) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(FONT), int(size))


def draw_index(image: Image.Image, rank: str, suit: str, color, large: bool,
               mirrored: bool = True) -> None:
    """Draws the rank and suit in the top-left corner and, if `mirrored`, upside down in the
    bottom-right."""
    corner = Image.new("RGBA", (W // 2, H // 2), (0, 0, 0, 0))
    draw = ImageDraw.Draw(corner)
    rank_size = (0.25 if large else 0.135) * H
    suit_size = (0.17 if large else 0.095) * H
    left = 0.085 * W if large else 0.07 * W
    text_font = font(rank_size)
    # "10" is squeezed a little so it is no wider than other ranks.
    if rank == "10":
        text_font = font(rank_size * 0.82)
    box = draw.textbbox((0, 0), rank, font=text_font, anchor="mt")
    rank_width = box[2] - box[0]
    center_x = left + max(rank_width, suit_size) / 2
    top = 0.04 * H
    draw.text((center_x, top), rank, font=text_font, fill=color, anchor="mt")
    rank_bottom = top + rank_size * 0.78
    draw_suit(draw, suit, center_x, rank_bottom + suit_size * 0.6, suit_size, color)
    image.alpha_composite(corner, (0, 0))
    if mirrored:
        image.alpha_composite(corner.rotate(180), (W - W // 2, H - H // 2))


def draw_small_bottom_index(image: Image.Image, rank: str, suit: str, color) -> None:
    """Just the bottom-right half of a regular-size index."""
    corner = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw_index(corner, rank, suit, color, large=False, mirrored=False)
    image.alpha_composite(corner.rotate(180).crop((W // 2, H // 2, W, H)), (W // 2, H // 2))


# Pip positions as (column, row) fractions of the card; rows below the middle are drawn flipped.
L, C, R = 0.345, 0.5, 0.655
T, B = 0.215, 0.785
PIPS = {
    "2": [(C, T), (C, B)],
    "3": [(C, T), (C, 0.5), (C, B)],
    "4": [(L, T), (R, T), (L, B), (R, B)],
    "5": [(L, T), (R, T), (C, 0.5), (L, B), (R, B)],
    "6": [(L, T), (R, T), (L, 0.5), (R, 0.5), (L, B), (R, B)],
    "7": [(L, T), (R, T), (C, 0.35), (L, 0.5), (R, 0.5), (L, B), (R, B)],
    "8": [(L, T), (R, T), (C, 0.35), (L, 0.5), (R, 0.5), (C, 0.65), (L, B), (R, B)],
    "9": [(L, T), (R, T), (L, 0.4), (R, 0.4), (C, 0.5), (L, 0.6), (R, 0.6), (L, B), (R, B)],
    "10": [(L, T), (R, T), (C, 0.3), (L, 0.4), (R, 0.4), (L, 0.6), (R, 0.6), (C, 0.7), (L, B),
           (R, B)],
}


def draw_pips(draw: ImageDraw.ImageDraw, rank: str, suit: str, color) -> None:
    # Big pips, a little smaller on the busiest cards so they don't touch.
    size = (0.115 if rank in ("9", "10") else 0.13) * H
    for x, y in PIPS[rank]:
        draw_suit(draw, suit, x * W, y * H, size, color, flip=y > 0.5)


def draw_ace(draw: ImageDraw.ImageDraw, suit: str, color) -> None:
    draw_suit(draw, suit, W / 2, H / 2, 0.5 * H, color)


def tint(color, amount: float, paper=PAPER):
    """Mixes `color` toward the `paper` color by `amount` (0 = color, 1 = paper)."""
    return tuple(int(c + (p - c) * amount) for c, p in zip(color, paper[:3])) + (255,)


def crown(draw: ImageDraw.ImageDraw, cx: float, base: float, width: float, points: int, color,
          gold) -> None:
    """A crown with `points` peaks, its band's bottom at `base`."""
    height = width * 0.55
    band = height * 0.28
    outline = [(cx - width / 2, base)]
    for i in range(points * 2 + 1):
        x = cx - width / 2 + width * i / (points * 2)
        y = base - height if i % 2 == 0 else base - height * 0.45
        outline.append((x, y))
    outline.append((cx + width / 2, base))
    draw.polygon(outline, fill=gold, outline=color, width=SCALE * 2)
    draw.rectangle((cx - width / 2, base - band, cx + width / 2, base), fill=color)
    for i in range(0, points * 2 + 1, 2):
        x = cx - width / 2 + width * i / (points * 2)
        r = width * 0.045
        draw.ellipse((x - r, base - height - r, x + r, base - height + r), fill=color)


def cap(draw: ImageDraw.ImageDraw, cx: float, base: float, width: float, color, gold) -> None:
    """A rounded cap with a brim and a pom-pom, for the Jack."""
    height = width * 0.5
    # The crown of the cap: a half ellipse sitting on the brim.
    draw.pieslice((cx - width * 0.36, base - height * 1.5, cx + width * 0.36, base), 180, 360,
                  fill=gold, outline=color, width=SCALE * 2)
    # The brim.
    draw.rounded_rectangle((cx - width / 2, base - height * 0.22, cx + width / 2, base),
                           3 * SCALE, fill=color)
    # The pom-pom.
    r = width * 0.09
    top = base - height * 0.75
    draw.ellipse((cx - r, top - r, cx + r, top + r), fill=color)


def draw_court(image: Image.Image, rank: str, suit: str, color, paper) -> None:
    """A two-ended court card: a crown (or cap), the rank's letter and a tinted suit emblem."""
    draw = ImageDraw.Draw(image)
    # The frame.
    left, top, right, bottom = 0.25 * W, 0.08 * H, 0.75 * W, 0.92 * H
    draw.rounded_rectangle((left, top, right, bottom), 14 * SCALE,
                           fill=tint(color, 0.88, paper), outline=color, width=3 * SCALE)
    draw.line((left + 10 * SCALE, H / 2, right - 10 * SCALE, H / 2),
              fill=tint(color, 0.5, paper), width=2 * SCALE)
    gold = (232, 183, 58, 255)
    # Draw the top half, then copy it upside down onto the bottom half.
    half = Image.new("RGBA", (W, H // 2), (0, 0, 0, 0))
    half_draw = ImageDraw.Draw(half)
    middle_x = W / 2
    draw_suit(half_draw, suit, middle_x, 0.365 * H, 0.24 * H, tint(color, 0.6, paper))
    letter_font = font(0.18 * H)
    half_draw.text((middle_x, 0.36 * H), rank, font=letter_font, fill=color, anchor="mm")
    crown_width = 0.28 * W
    crown_base = 0.25 * H
    if rank == "K":
        crown(half_draw, middle_x, crown_base, crown_width, 3, color, gold)
    elif rank == "Q":
        crown(half_draw, middle_x, crown_base, crown_width * 0.8, 2, color, gold)
    else:
        cap(half_draw, middle_x, crown_base, crown_width * 0.85, color, gold)
    image.alpha_composite(half, (0, 0))
    image.alpha_composite(half.rotate(180), (0, H - H // 2))


def draw_large_print_center(draw: ImageDraw.ImageDraw, rank: str, suit: str, color,
                            paper) -> None:
    """Large-print cards show one big suit (and, for courts, the letter) below the index."""
    if rank in ("J", "Q", "K"):
        draw.text((W * 0.5, H * 0.62), rank, font=font(0.36 * H), fill=tint(color, 0.35, paper),
                  anchor="mm")
    else:
        draw_suit(draw, suit, W * 0.5, H * 0.63, 0.38 * H, color)


def make_face(deck: Deck, rank: str, suit: str) -> Image.Image:
    image, draw = blank_card(deck.paper, deck.edge)
    color = deck.color(suit) + (255,)
    if deck.large_print:
        draw_large_print_center(draw, rank, suit, color, deck.paper)
        # Only the top-left index is big: it's the part that shows in a fanned pile.
        draw_index(image, rank, suit, color, large=True, mirrored=False)
        draw_small_bottom_index(image, rank, suit, color)
    else:
        if rank == "A":
            draw_ace(draw, suit, color)
        elif rank in ("J", "Q", "K"):
            draw_court(image, rank, suit, color, deck.paper)
        else:
            draw_pips(draw, rank, suit, color)
        draw_index(image, rank, suit, color, large=False)
    return image.resize((WIDTH, HEIGHT), Image.LANCZOS)


# --- Card backs ----------------------------------------------------------------------------------


@dataclass
class Back:
    name: str
    base: tuple[int, int, int]
    accent: tuple[int, int, int]
    pattern: str
    # Dark backs get a dark border, to match the dark decks.
    dark: bool = False


BACKS = [
    Back("teal-lattice", (14, 98, 104), (120, 200, 196), "lattice"),
    Back("crimson-weave", (128, 20, 36), (232, 120, 120), "weave"),
    Back("midnight-stars", (18, 24, 58), (236, 214, 140), "stars"),
    Back("graphite", (40, 42, 48), (98, 104, 116), "lattice", dark=True),
]


def make_back(back: Back) -> Image.Image:
    image, draw = blank_card(DARK_PAPER, DARK_EDGE) if back.dark else blank_card()
    margin = 0.055 * W
    inner = (margin, margin, W - margin, H - margin)
    panel = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    panel_draw = ImageDraw.Draw(panel)
    panel_draw.rectangle((0, 0, W, H), fill=back.base + (255,))
    accent = back.accent + (255,)
    if back.pattern == "lattice":
        step = 0.075 * W
        for i in range(-30, 30):
            x = i * step
            panel_draw.line((x, 0, x + H, H), fill=accent, width=2 * SCALE)
            panel_draw.line((x + H, 0, x, H), fill=accent, width=2 * SCALE)
    elif back.pattern == "weave":
        step = 0.06 * W
        for row, y in enumerate(range(0, H, int(step))):
            for col, x in enumerate(range(0, W, int(step))):
                if (row + col) % 2:
                    panel_draw.rounded_rectangle((x + 3, y + step * 0.3, x + step - 3,
                                                  y + step * 0.7), 6, fill=accent)
                else:
                    panel_draw.rounded_rectangle((x + step * 0.3, y + 3, x + step * 0.7,
                                                  y + step - 3), 6, fill=accent)
    else:  # stars
        rng = random.Random(7)
        for _ in range(260):
            x, y = rng.uniform(0, W), rng.uniform(0, H)
            r = rng.choice([1.5, 2, 2, 3, 4]) * SCALE
            panel_draw.ellipse((x - r, y - r, x + r, y + r), fill=accent)
    # A medallion in the middle.
    cx, cy, r = W / 2, H / 2, 0.2 * W
    panel_draw.ellipse((cx - r, cy - r, cx + r, cy + r), fill=back.base + (255,), outline=accent,
                       width=4 * SCALE)
    for i, suit in enumerate("shdc"):
        angle = math.pi / 2 * i - math.pi / 2
        draw_suit(panel_draw, suit, cx + math.cos(angle) * r * 0.5, cy + math.sin(angle) * r * 0.5,
                  r * 0.6, accent)
    # Keep the pattern inside a rounded inner border.
    mask = Image.new("L", (W, H), 0)
    ImageDraw.Draw(mask).rounded_rectangle(inner, RADIUS * 0.6, fill=255)
    image.paste(panel, (0, 0), mask)
    draw.rounded_rectangle(inner, RADIUS * 0.6, outline=back.base + (255,), width=3 * SCALE)
    return image.resize((WIDTH, HEIGHT), Image.LANCZOS)


def main() -> None:
    for deck in DECKS:
        out = ROOT / "assets" / "decks" / deck.name
        out.mkdir(parents=True, exist_ok=True)
        for number, rank in enumerate(RANKS, start=1):
            for suit in SUITS:
                make_face(deck, rank, suit).save(out / f"{number}{suit}.png", optimize=True)
        print(f"wrote {out.relative_to(ROOT)}")
    out = ROOT / "assets" / "backs"
    out.mkdir(parents=True, exist_ok=True)
    for back in BACKS:
        make_back(back).save(out / f"{back.name}.png", optimize=True)
        print(f"wrote {(out / back.name).relative_to(ROOT)}.png")


if __name__ == "__main__":
    main()
