#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Draw Wordsmith Book's mark and app icons (standard library only, deterministic).

The mark is an open book seen from above: two pages curving from a spine, with a ribbon marker
falling from the top of the spine. The app icon puts the mark on a rounded square in the brand's
binding color. Writes, into assets/brand/book/:

  mark.svg, app-icon.svg            vector sources
  icon-32.png ... icon-1024.png     the app icon, rasterised here

Run from the repository root:  python3 tools/brand-icons.py
The shapes are also drawn by the app (crates/book-ui/src/brand.rs); keep the two in step.
"""

import math
import os
import struct
import zlib

OUT = os.path.join("assets", "brand", "book")

BINDING = (122, 46, 58)  # the brand's binding color (#7A2E3A)
PAPER = (250, 247, 240)  # paper (#FAF7F0)
GILT = (201, 162, 74)  # the ribbon (#C9A24A)
INK = (30, 38, 48)  # ink (#1E2630)

# The mark in a 100 x 100 box: (fill, polygon).
def page(side):
    """A page as a polygon: a curved top edge from the spine outward, straight outer edge and a
    curved bottom edge back to the spine. side is -1 (left) or +1 (right)."""
    pts = []
    steps = 24
    for i in range(steps + 1):  # top edge, spine to outer corner
        t = i / steps
        x = 50 + side * 40 * t
        y = 30 - 8 * math.sin(math.pi * t) + 2 * t
        pts.append((x, y))
    for i in range(steps + 1):  # bottom edge, outer corner to spine
        t = 1 - i / steps
        x = 50 + side * 40 * t
        y = 76 - 6 * math.sin(math.pi * t) + 2 * t
        pts.append((x, y))
    return pts


RIBBON = [(53, 30), (59, 30), (59, 86), (56, 82), (53, 86)]
SPINE = [(49.2, 28), (50.8, 28), (50.8, 78), (49.2, 78)]


def lines(side):
    """Three text lines on a page, as thin polygons."""
    out = []
    for k in range(3):
        y0 = 40 + k * 10
        pts = []
        steps = 16
        for i in range(steps + 1):
            t = 0.15 + 0.7 * i / steps
            pts.append((50 + side * 40 * t, y0 - 6 * math.sin(math.pi * t) + 2 * t))
        for i in range(steps + 1):
            t = 0.85 - 0.7 * i / steps
            pts.append((50 + side * 40 * t, y0 + 1.6 - 6 * math.sin(math.pi * t) + 2 * t))
        out.append(pts)
    return out


def mark_shapes(ink):
    shapes = [(PAPER, page(-1)), (PAPER, page(1))]
    for side in (-1, 1):
        shapes += [(ink, pts) for pts in lines(side)]
    shapes += [(ink, SPINE), (GILT, RIBBON)]
    return shapes


def inside(pts, x, y):
    c = False
    n = len(pts)
    j = n - 1
    for i in range(n):
        xi, yi = pts[i]
        xj, yj = pts[j]
        if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
            c = not c
        j = i
    return c


def rounded(x, y, r=22):
    """Inside the icon's rounded square (0..100)."""
    cx = min(max(x, r), 100 - r)
    cy = min(max(y, r), 100 - r)
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r and 0 <= x <= 100 and 0 <= y <= 100


def bbox(pts):
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    return min(xs), min(ys), max(xs), max(ys)


def render(size, ss):
    shapes = [(c, p, bbox(p)) for c, p in mark_shapes(BINDING)]
    # The mark sits in the middle 80% of the icon.
    rows = []
    for py in range(size):
        row = bytearray()
        for px in range(size):
            acc = [0, 0, 0, 0]
            for sy in range(ss):
                for sx in range(ss):
                    x = (px + (sx + 0.5) / ss) * 100 / size
                    y = (py + (sy + 0.5) / ss) * 100 / size
                    if not rounded(x, y):
                        continue
                    color = BINDING
                    mx = 50 + (x - 50) / 0.8
                    my = 50 + (y - 50) / 0.8 + 3
                    for c, p, (x0, y0, x1, y1) in shapes:
                        if x0 <= mx <= x1 and y0 <= my <= y1 and inside(p, mx, my):
                            color = c
                    acc[0] += color[0]
                    acc[1] += color[1]
                    acc[2] += color[2]
                    acc[3] += 255
            n = ss * ss
            a = acc[3] // n
            if a == 0:
                row += bytes((0, 0, 0, 0))
            else:
                k = acc[3] / 255
                row += bytes((round(acc[0] / k), round(acc[1] / k), round(acc[2] / k), a))
        rows.append(bytes(row))
    return rows


def png(rows, size):
    raw = b"".join(b"\x00" + r for r in rows)

    def chunk(kind, data):
        c = kind + data
        return struct.pack(">I", len(data)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def hexc(c):
    return "#%02X%02X%02X" % c


def svg_path(pts):
    return "M" + " L".join("%.2f %.2f" % p for p in pts) + " Z"


def svg(icon):
    body = []
    if icon:
        body.append('<rect x="0" y="0" width="100" height="100" rx="22" fill="%s"/>' % hexc(BINDING))
        body.append('<g transform="translate(10 7.6) scale(0.8)">')
    else:
        body.append("<g>")
    for c, p in mark_shapes(BINDING if icon else INK):
        if not icon and c == PAPER:
            body.append('<path d="%s" fill="none" stroke="%s" stroke-width="3"/>' % (svg_path(p), hexc(INK)))
        else:
            body.append('<path d="%s" fill="%s"/>' % (svg_path(p), hexc(c)))
    body.append("</g>")
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">\n' + "\n".join(body) + "\n</svg>\n"


def main():
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, "mark.svg"), "w") as f:
        f.write(svg(False))
    with open(os.path.join(OUT, "app-icon.svg"), "w") as f:
        f.write(svg(True))
    for size, ss in ((32, 4), (256, 3), (512, 2), (1024, 1)):
        with open(os.path.join(OUT, "icon-%d.png" % size), "wb") as f:
            f.write(png(render(size, ss), size))
        print("wrote icon-%d.png" % size)


if __name__ == "__main__":
    main()
