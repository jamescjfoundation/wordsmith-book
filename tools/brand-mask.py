#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Draw the mark's letter as a white mask for the app (standard library only, deterministic).

The app paints the Wordsmith Book mark itself: the letter B from this mask, tinted with the
theme's ink, and the ribbon cursor as plain shapes in Ribbon Green (crates/book-ui/src/brand.rs).
This reads the B (the first path) from assets/brand/wordsmith-book-mark-ink.svg, fills it with
the even-odd rule at 4 x 4 samples a pixel, and writes assets/brand/mark-mask.png: white, with
the letter's coverage as alpha, covering the SVG's whole view box at SCALE pixels a unit.

Run from the repository root:  python3 tools/brand-mask.py
"""

import os
import re
import struct
import zlib

SRC = os.path.join("assets", "brand", "wordsmith-book-mark-ink.svg")
OUT = os.path.join("assets", "brand", "mark-mask.png")
SCALE = 2.0  # pixels per view-box unit
SAMPLES = 4  # samples per pixel along each axis


def tokens(d):
    return re.findall(r"[MLHVQZmlhvqz]|-?\d*\.?\d+(?:e-?\d+)?", d)


def outlines(d):
    """The path's closed outlines as point lists, quadratic curves flattened."""
    out, cur, pos, start = [], [], (0.0, 0.0), (0.0, 0.0)
    t = tokens(d)
    i, cmd = 0, None
    num = lambda: float(t[i])
    while i < len(t):
        if re.match(r"[A-Za-z]", t[i]):
            cmd = t[i]
            i += 1
            if cmd in "Zz":
                if cur:
                    out.append(cur)
                cur, pos = [], start
                continue
        if cmd == "M":
            if cur:
                out.append(cur)
            pos = (float(t[i]), float(t[i + 1]))
            start, cur = pos, [pos]
            i += 2
            cmd = "L"
        elif cmd == "L":
            pos = (float(t[i]), float(t[i + 1]))
            cur.append(pos)
            i += 2
        elif cmd == "H":
            pos = (float(t[i]), pos[1])
            cur.append(pos)
            i += 1
        elif cmd == "V":
            pos = (pos[0], float(t[i]))
            cur.append(pos)
            i += 1
        elif cmd == "Q":
            c = (float(t[i]), float(t[i + 1]))
            e = (float(t[i + 2]), float(t[i + 3]))
            for k in range(1, 9):
                u = k / 8
                x = (1 - u) ** 2 * pos[0] + 2 * (1 - u) * u * c[0] + u * u * e[0]
                y = (1 - u) ** 2 * pos[1] + 2 * (1 - u) * u * c[1] + u * u * e[1]
                cur.append((x, y))
            pos = e
            i += 4
        else:
            raise SystemExit(f"unsupported path command {cmd!r}")
    if cur:
        out.append(cur)
    return out


def png(width, height, rgba):
    raw = b"".join(b"\x00" + rgba[y * width * 4:(y + 1) * width * 4] for y in range(height))
    chunk = lambda kind, data: struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def main():
    svg = open(SRC, encoding="utf-8").read()
    box = [float(v) for v in re.search(r'viewBox="([^"]+)"', svg).group(1).split()]
    d = re.search(r'<path[^>]*\bd="([^"]+)"', svg).group(1)
    edges = []
    for poly in outlines(d):
        for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
            if y0 != y1:
                edges.append((x0, y0, x1, y1))
    width, height = round(box[2] * SCALE), round(box[3] * SCALE)
    rgba = bytearray(width * height * 4)
    n = SAMPLES
    for py in range(height):
        cover = [0] * width
        for sy in range(n):
            y = box[1] + (py + (sy + 0.5) / n) / SCALE
            xs = sorted(x0 + (y - y0) * (x1 - x0) / (y1 - y0) for x0, y0, x1, y1 in edges if (y0 <= y < y1) or (y1 <= y < y0))
            for a, b in zip(xs[0::2], xs[1::2]):
                for px in range(max(0, int((a - box[0]) * SCALE)), min(width, int((b - box[0]) * SCALE) + 1)):
                    for sx in range(n):
                        x = box[0] + (px + (sx + 0.5) / n) / SCALE
                        if a <= x < b:
                            cover[px] += 1
        for px, c in enumerate(cover):
            if c:
                o = (py * width + px) * 4
                rgba[o:o + 4] = bytes((255, 255, 255, round(255 * c / (n * n))))
    with open(OUT, "wb") as f:
        f.write(png(width, height, bytes(rgba)))
    print(f"{OUT}: {width} x {height}")


if __name__ == "__main__":
    main()
