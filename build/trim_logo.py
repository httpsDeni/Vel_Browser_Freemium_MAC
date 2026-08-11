#!/usr/bin/env python3
"""Turns a logo render into an icon-shaped PNG.

    ./build/trim_logo.py "Vel Browser.jpg" build/logo.png

Artwork usually arrives as a picture *of* an icon: the mark centred on a
backdrop, with a drop shadow. macOS wants the opposite — the mark alone, its
corners transparent, sitting on a canvas with the margin Apple's grid expects.
This script does that conversion:

  1. find the mark by walking in from the edges until the backdrop stops,
  2. mask it to a rounded square, which removes the shadow along with the
     corners and is more predictable than trying to key the shadow out,
  3. centre it on a transparent canvas at Apple's proportions.

Stock macOS Python only: PNG is decoded and encoded here with `zlib` and
`struct`, and JPEG input is converted by `sips` first, so the repo needs no
imaging library.
"""

import os
import struct
import subprocess
import sys
import tempfile
import zlib

# Apple's macOS icon grid: the rounded square covers about 80% of the canvas,
# and its corner radius is a bit over a fifth of its own side.
CONTENT_FRACTION = 0.80
CORNER_FRACTION = 0.225
# Supersampling for the corner mask. 4x4 is plenty for a curve this gentle.
SAMPLES = 4
# How far a pixel may drift from the corner colour and still count as backdrop.
# Generous because the source is usually a JPEG, and JPEG ringing around a
# hard edge is exactly where a tight threshold goes wrong.
TOLERANCE = 30


def decode_png(path):
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        sys.exit(f"trim_logo: {path} is not a PNG")

    pos, idat = 8, bytearray()
    width = height = colortype = None
    while pos < len(data):
        (length,) = struct.unpack(">I", data[pos : pos + 4])
        kind = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + length]
        if kind == b"IHDR":
            width, height, depth, colortype, _, _, interlace = struct.unpack(
                ">IIBBBBB", body
            )
            if depth != 8 or interlace:
                sys.exit("trim_logo: need an 8-bit, non-interlaced PNG")
        elif kind == b"IDAT":
            idat += body
        elif kind == b"IEND":
            break
        pos += 12 + length

    channels = {0: 1, 2: 3, 4: 2, 6: 4}.get(colortype)
    if channels is None:
        sys.exit(f"trim_logo: unsupported colour type {colortype}")

    raw = zlib.decompress(bytes(idat))
    stride = width * channels
    out = bytearray(width * height * 4)
    previous = bytearray(stride)
    pos = 0

    for y in range(height):
        filt = raw[pos]
        pos += 1
        line = bytearray(raw[pos : pos + stride])
        pos += stride

        # Undo the per-scanline filter (PNG spec, section 9).
        for i in range(stride):
            a = line[i - channels] if i >= channels else 0
            b = previous[i]
            c = previous[i - channels] if i >= channels else 0
            x = line[i]
            if filt == 1:
                x += a
            elif filt == 2:
                x += b
            elif filt == 3:
                x += (a + b) >> 1
            elif filt == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                x += a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
            line[i] = x & 0xFF
        previous = line

        for x in range(width):
            px = line[x * channels : (x + 1) * channels]
            if channels == 1:
                r = g = b = px[0]
                alpha = 255
            elif channels == 2:
                r = g = b = px[0]
                alpha = px[1]
            elif channels == 3:
                r, g, b = px
                alpha = 255
            else:
                r, g, b, alpha = px
            o = (y * width + x) * 4
            out[o : o + 4] = bytes((r, g, b, alpha))

    return width, height, out


def encode_png(path, width, height, rgba):
    raw = b"".join(
        b"\x00" + bytes(rgba[y * width * 4 : (y + 1) * width * 4]) for y in range(height)
    )

    def chunk(kind, body):
        return (
            struct.pack(">I", len(body))
            + kind
            + body
            + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
        )

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(raw, 9)))
        f.write(chunk(b"IEND", b""))


def content_bounds(width, height, rgba):
    """Bounding box of everything that is not the backdrop."""
    backdrop = rgba[0:3]

    def is_backdrop(x, y):
        o = (y * width + x) * 4
        if rgba[o + 3] < 8:
            return True
        return all(abs(rgba[o + i] - backdrop[i]) <= TOLERANCE for i in range(3))

    left, right, top, bottom = width, -1, height, -1
    for y in range(height):
        for x in range(width):
            if not is_backdrop(x, y):
                left, right = min(left, x), max(right, x)
                top, bottom = min(top, y), max(bottom, y)
    if right < left or bottom < top:
        sys.exit("trim_logo: the whole image looks like backdrop")
    return left, top, right, bottom


def rounded_alpha(size, radius):
    """Coverage mask for a rounded square, supersampled at the corners."""
    mask = bytearray(size * size)
    step = 1.0 / SAMPLES
    for y in range(size):
        for x in range(size):
            # Only the corner boxes need sampling; everything else is solid.
            near_x = x < radius or x >= size - radius
            near_y = y < radius or y >= size - radius
            if not (near_x and near_y):
                mask[y * size + x] = 255
                continue
            cx = radius if x < radius else size - radius - 1
            cy = radius if y < radius else size - radius - 1
            hits = 0
            for sy in range(SAMPLES):
                for sx in range(SAMPLES):
                    px = x + (sx + 0.5) * step
                    py = y + (sy + 0.5) * step
                    dx = max(0.0, (cx - px) if x < radius else (px - cx))
                    dy = max(0.0, (cy - py) if y < radius else (py - cy))
                    if dx * dx + dy * dy <= radius * radius:
                        hits += 1
            mask[y * size + x] = (hits * 255) // (SAMPLES * SAMPLES)
    return mask


def main():
    if len(sys.argv) < 2:
        sys.exit(f"usage: {sys.argv[0]} <source image> [output.png]")
    source = sys.argv[1]
    dest = sys.argv[2] if len(sys.argv) > 2 else "build/logo.png"

    with tempfile.TemporaryDirectory() as work:
        png = os.path.join(work, "source.png")
        subprocess.run(
            ["sips", "-s", "format", "png", source, "--out", png],
            check=True,
            stdout=subprocess.DEVNULL,
        )
        width, height, rgba = decode_png(png)

    left, top, right, bottom = content_bounds(width, height, rgba)
    print(f"==> Mark found at {left},{top} to {right},{bottom}")

    # Square it up around the mark's centre, then pull in a couple of pixels:
    # the outermost ring is drop shadow and JPEG ringing, not artwork.
    side = max(right - left, bottom - top) + 1
    cx, cy = (left + right) // 2, (top + bottom) // 2
    inset = max(1, side // 100)
    side -= inset * 2
    x0, y0 = cx - side // 2, cy - side // 2

    mask = rounded_alpha(side, int(side * CORNER_FRACTION))

    canvas = int(round(side / CONTENT_FRACTION))
    offset = (canvas - side) // 2
    out = bytearray(canvas * canvas * 4)

    for y in range(side):
        sy = y0 + y
        if not 0 <= sy < height:
            continue
        for x in range(side):
            sx = x0 + x
            if not 0 <= sx < width:
                continue
            coverage = mask[y * side + x]
            if coverage == 0:
                continue
            si = (sy * width + sx) * 4
            di = ((y + offset) * canvas + (x + offset)) * 4
            out[di : di + 3] = rgba[si : si + 3]
            out[di + 3] = coverage * rgba[si + 3] // 255

    os.makedirs(os.path.dirname(dest) or ".", exist_ok=True)
    encode_png(dest, canvas, canvas, out)
    print(f"==> Wrote {dest} ({canvas}x{canvas}, mark {side}px)")
    print("    Next: ./build/make_icon.sh && ./build/bundle.sh")


if __name__ == "__main__":
    main()
