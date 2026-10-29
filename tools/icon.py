"""Draws icon.png, the registry icon: two hands holding up a wooden chest, in a vanilla tooltip's frame (the frame
LookAt's icon uses, so the mods look like one family). 32 x 32 pixel art scaled 8 times to 256 x 256. Needs Pillow.

    python tools/icon.py [icon.png]
"""
import math
import sys

from PIL import Image

SIZE, SCALE = 32, 8
BACKGROUND = (0x10, 0x00, 0x10, 0xF0)
FRAME_TOP = (0x50, 0x00, 0xFF, 0xFF)
FRAME_BOTTOM = (0x28, 0x00, 0x7F, 0xFF)


def lerp(a, b, t):
    return tuple(round(ca + (cb - ca) * t) for ca, cb in zip(a, b))


class Canvas:
    """A 32 x 32 tooltip; drawing stays inside its frame."""

    def __init__(self):
        self.img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
        self.px = self.img.load()
        for y in range(SIZE):
            for x in range(SIZE):
                if x in (0, SIZE - 1) and y in (0, SIZE - 1):
                    continue
                edge = x in (0, SIZE - 1) or y in (0, SIZE - 1)
                frame = x in (1, SIZE - 2) or y in (1, SIZE - 2)
                self.px[x, y] = lerp(FRAME_TOP, FRAME_BOTTOM, (y - 1) / (SIZE - 3)) if frame and not edge else BACKGROUND

    def set(self, x, y, c):
        if not (2 <= x < SIZE - 2 and 2 <= y < SIZE - 2) or c is None:
            return
        if len(c) == 4 and c[3] < 255:
            under, a = self.px[x, y], c[3] / 255
            c = tuple(round(u + (v - u) * a) for u, v in zip(under[:3], c[:3])) + (under[3],)
        self.px[x, y] = c if len(c) == 4 else c + (255,)

    def sprite(self, x0, y0, art, palette):
        for dy, row in enumerate(art):
            for dx, ch in enumerate(row):
                if ch not in ". ":
                    self.set(x0 + dx, y0 + dy, palette[ch])

    def save(self, path):
        self.img.resize((SIZE * SCALE, SIZE * SCALE), Image.NEAREST).save(path, optimize=True)


PALETTE = {
    # the chest: its outline and corner bands, planks, their light top, the lines between them; the latch
    "K": (0x4A, 0x2C, 0x12), "W": (0xA8, 0x70, 0x38), "w": (0xC8, 0x90, 0x50), "d": (0x7A, 0x4E, 0x24),
    "G": (0xC8, 0xC8, 0xD0), "g": (0x70, 0x70, 0x7C),
    # the hands and arms: skin, its shadow, the sleeves and their shadow
    "H": (0xE8, 0xB4, 0x8C), "h": (0xBC, 0x84, 0x60), "S": (0x4C, 0x6C, 0xB0), "s": (0x34, 0x4C, 0x80),
}

CHEST = [
    ".KKKKKKKKKKKKKKKK.",
    "KKwwwwwwwwwwwwwwKK",
    "KWWWWWWWWWWWWWWWWK",
    "KWWWWWWWWWWWWWWWWK",
    "KdddddddddddddddWK",
    "KKKKKKKKgGGgKKKKKK",
    "KWwwwwwwgGGgwwwwWK",
    "KWWWWWWWWggWWWWWWK",
    "KWWWWWWWWWWWWWWWWK",
    "KddddddddddddddddK",
    "KWWWWWWWWWWWWWWWWK",
    "KWWWWWWWWWWWWWWWWK",
    "KWWWWWWWWWWWWWWWWK",
    "KddddddddddddddddK",
    ".KKKKKKKKKKKKKKKK.",
]

def arm(c, mirrored):
    """An arm reaching up from the bottom corner, its hand over the chest's lower corner; the right arm mirrors the
    left one."""
    for y in range(14, 29):
        centre = 9.0 - (y - 14) * 0.38
        for i in range(5):
            x = math.floor(centre - 2.5) + i
            hand = y <= 19
            shade = i == 4 or (hand and y == 19)
            colour = ("h" if shade else "H") if hand else ("s" if shade else "S")
            if hand and y == 14 and i in (0, 4):
                continue
            c.set(SIZE - 1 - x if mirrored else x, y, PALETTE[colour])


def draw():
    c = Canvas()
    c.sprite(7, 5, CHEST, PALETTE)
    arm(c, False)
    arm(c, True)
    return c


if __name__ == "__main__":
    draw().save(sys.argv[1] if len(sys.argv) > 1 else "icon.png")
