#!/usr/bin/env python3
"""Builds the Lemon Squeezy product images for Vel.

Ten 1600x1200 (4:3) PNGs — the size Lemon Squeezy's uploader asks for: four are real screenshots of the app, six explain
something the screenshots cannot. Every number that appears is one that was
actually measured — see `05_cpu`, which carries its own conditions in small
print because a benchmark without them is decoration.
"""

import os
from PIL import Image, ImageDraw, ImageFilter, ImageFont

W, H = 1600, 1200
MARGIN = 116

INK = (245, 245, 247)
MUTED = (154, 154, 162)
DIM = (108, 108, 116)
ACCENT = (224, 122, 46)          # the glowing ring in the icon
BG_TOP = (26, 26, 30)
BG_BOTTOM = (10, 10, 12)
PANEL = (32, 32, 37)

ROOT = "/Users/devdeni/Documents/Browser_Open_source"
SHOTS = os.path.join(ROOT, "store/screenshots")
OUT = os.path.join(ROOT, "store")
SF = "/System/Library/Fonts/SFCompact.ttf"


def font(size, weight="Regular"):
    f = ImageFont.truetype(SF, size)
    try:
        f.set_variation_by_name(weight)
    except Exception:
        pass
    return f


def canvas(glow=True):
    """Dark backdrop with a soft accent bloom in the upper left."""
    img = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(img)
    for y in range(H):
        t = y / H
        d.line(
            [(0, y), (W, y)],
            fill=tuple(int(BG_TOP[i] + (BG_BOTTOM[i] - BG_TOP[i]) * t) for i in range(3)),
        )
    if glow:
        bloom = Image.new("RGB", (W, H), (0, 0, 0))
        ImageDraw.Draw(bloom).ellipse(
            [-360, -460, 900, 560], fill=(int(ACCENT[0] * 0.30), int(ACCENT[1] * 0.20), int(ACCENT[2] * 0.10))
        )
        img = Image.blend(img, Image.blend(img, bloom, 0.55).filter(ImageFilter.GaussianBlur(180)), 0.75)
    return img


def text(d, xy, body, size, weight="Regular", fill=INK, spacing=1.35, anchor=None):
    f = font(size, weight)
    x, y = xy
    for line in body.split("\n"):
        d.text((x, y), line, font=f, fill=fill, anchor=anchor)
        y += int(size * spacing)
    return y


def rounded(img, radius):
    mask = Image.new("L", img.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, img.size[0] - 1, img.size[1] - 1], radius, fill=255)
    out = img.convert("RGBA")
    out.putalpha(mask)
    return out


def drop(base, art, xy, blur=42, spread=26, opacity=170):
    """Paste `art` with a soft shadow beneath it."""
    shadow = Image.new("RGBA", (art.width + spread * 4, art.height + spread * 4), (0, 0, 0, 0))
    shadow.paste((0, 0, 0, opacity), (spread * 2, spread * 2 + spread // 2), art.split()[3])
    shadow = shadow.filter(ImageFilter.GaussianBlur(blur))
    base.paste(shadow, (xy[0] - spread * 2, xy[1] - spread * 2), shadow)
    base.paste(art, xy, art)


def screenshot(name, width):
    im = Image.open(os.path.join(SHOTS, name)).convert("RGB")
    h = int(im.height * width / im.width)
    return rounded(im.resize((width, h), Image.LANCZOS), 18)


def logo(size):
    return Image.open(os.path.join(ROOT, "build/logo.png")).convert("RGBA").resize(
        (size, size), Image.LANCZOS
    )


def caption(d, title, sub, y=MARGIN):
    """Standard headline block: one claim, one sentence under it."""
    end = text(d, (MARGIN, y), title, 68, "Bold")
    if sub:
        text(d, (MARGIN, end + 16), sub, 33, "Regular", MUTED, spacing=1.42)


def footnote(d, body):
    text(d, (MARGIN, H - MARGIN - 34), body, 22, "Regular", DIM, spacing=1.4)


# ----------------------------------------------------------------- the images


def hero():
    img = canvas()
    d = ImageDraw.Draw(img)
    mark = logo(430)
    drop(img, mark, ((W - 430) // 2, 210), blur=60, opacity=190)
    text(d, (W // 2, 700), "Vel", 168, "Bold", INK, anchor="ma")
    text(d, (W // 2, 900), "A fast, quiet browser for macOS", 44, "Medium", INK, anchor="ma")
    text(
        d,
        (W // 2, 968),
        "Native WebKit. Hardware video. 1.6 MB.",
        34,
        "Regular",
        MUTED,
        anchor="ma",
    )
    d.rounded_rectangle([W // 2 - 60, 1052, W // 2 + 60, 1058], 3, fill=ACCENT)
    return img


def shot_page(headline, sub, shot, note=None):
    img = canvas()
    d = ImageDraw.Draw(img)
    caption(d, headline, sub)
    # Sized to land inside the frame: a screenshot clipped by the canvas edge
    # reads as a mistake rather than as a crop.
    art = screenshot(shot, 1200)
    drop(img, art, ((W - 1200) // 2, 366))
    if note:
        footnote(d, note)
    return img


def cpu():
    img = canvas()
    d = ImageDraw.Draw(img)
    caption(
        d,
        "Roughly a quarter of the CPU",
        "Same 1080p live stream, playing in both browsers at once.",
    )

    rows = [("Safari", 45.1, (118, 118, 126)), ("Vel", 11.9, ACCENT)]
    top, bar_h, gap = 430, 124, 56
    x0, full = MARGIN + 210, W - MARGIN - 210 - MARGIN
    for i, (name, value, colour) in enumerate(rows):
        y = top + i * (bar_h + gap)
        text(d, (MARGIN, y + bar_h // 2 - 26), name, 44, "Medium", INK)
        d.rounded_rectangle([x0, y, x0 + full, y + bar_h], 14, fill=(38, 38, 44))
        width = int(full * value / 50.0)
        d.rounded_rectangle([x0, y, x0 + width, y + bar_h], 14, fill=colour)
        text(d, (x0 + width + 28, y + bar_h // 2 - 28), f"{value:.1f}%", 46, "Bold", INK)

    text(d, (MARGIN, 800), "Total CPU across every process, at steady state.", 30, "Regular", MUTED)
    footnote(
        d,
        "Measured on one Apple silicon Mac: mean of three 5-second samples, both windows visible.\n"
        "Memory was a tie — WebKit's decode buffers dominate a single video tab in either browser.",
    )
    return img


def bullets(headline, sub, items, note=None):
    img = canvas()
    d = ImageDraw.Draw(img)
    caption(d, headline, sub)
    y = 428
    for title, body in items:
        d.rounded_rectangle([MARGIN, y, W - MARGIN, y + 152], 18, fill=PANEL)
        d.rounded_rectangle([MARGIN, y + 34, MARGIN + 6, y + 118], 3, fill=ACCENT)
        text(d, (MARGIN + 48, y + 34), title, 40, "Semibold")
        text(d, (MARGIN + 48, y + 90), body, 29, "Regular", MUTED)
        y += 182
    if note:
        footnote(d, note)
    return img


def shortcuts():
    img = canvas()
    d = ImageDraw.Draw(img)
    caption(d, "Built for the keyboard", "Every macOS shortcut where you expect it.")
    keys = [
        ("⌘T  ⌘W", "new / close tab"),
        ("⌘L", "address bar"),
        ("⌘1 – ⌘9", "jump to a tab"),
        ("⌘[  ⌘]", "back / forward"),
        ("⌘R  ⇧⌘R", "reload / ignore cache"),
        ("⇧⌘P", "picture in picture"),
    ]
    cw, ch = (W - MARGIN * 2 - 40) // 2, 146
    for i, (combo, what) in enumerate(keys):
        x = MARGIN + (i % 2) * (cw + 40)
        y = 440 + (i // 2) * (ch + 38)
        d.rounded_rectangle([x, y, x + cw, y + ch], 18, fill=PANEL)
        text(d, (x + 40, y + 28), combo, 46, "Bold", ACCENT)
        text(d, (x + 40, y + 84), what, 27, "Regular", MUTED)
    return img


def plans():
    img = canvas()
    d = ImageDraw.Draw(img)
    caption(d, "Free browser. Supporter extras.", "Donations fund it. Nothing is nagged or time-limited.")
    cols = [
        ("Free", ["Full browsing and tabs", "Hardware-decoded video", "All keyboard shortcuts", "Chromeless interface"], PANEL, MUTED),
        ("Supporter", ["Ad & tracker blocking", "Custom filter lists", "Memory saver", "Picture in Picture"], (44, 33, 24), INK),
    ]
    cw = (W - MARGIN * 2 - 48) // 2
    for i, (name, items, fill, body_ink) in enumerate(cols):
        x = MARGIN + i * (cw + 48)
        d.rounded_rectangle([x, 440, x + cw, 1010], 22, fill=fill)
        if i == 1:
            d.rounded_rectangle([x, 440, x + cw, 448], 4, fill=ACCENT)
        text(d, (x + 46, 492), name, 50, "Bold", ACCENT if i else INK)
        y = 596
        for item in items:
            d.ellipse([x + 50, y + 12, x + 64, y + 26], fill=ACCENT if i else DIM)
            text(d, (x + 88, y), item, 30, "Regular", body_ink)
            y += 88
    return img


def main():
    os.makedirs(OUT, exist_ok=True)
    images = {
        "01_hero": hero(),
        "02_chromeless": shot_page(
            "The window is the page",
            "One translucent bar. No toolbars, no sidebars, nothing you did not ask for.",
            "a_single.png",
        ),
        "03_tabs": shot_page(
            "Tabs that stay out of the way",
            "The strip appears with your second tab and vanishes with it.",
            "b_tabs.png",
        ),
        "04_omnibox": shot_page(
            "One field, centred",
            "URLs and searches in the same place. Pasted script URLs are searched, never run.",
            "c_omnibox.png",
        ),
        "05_cpu": cpu(),
        "06_blocking": bullets(
            "Blocking that runs inside WebKit",
            "Filters are compiled to native rules, not enforced by an extension.",
            [
                ("Blocked before the socket opens", "Rules are evaluated in WebKit's network process, ahead of the request."),
                ("No per-request cost in the app", "Nothing crosses back into the browser to decide. There is no callback to be slow."),
                ("Your own lists, layered on", "Adblock Plus syntax. Point it at any list you already trust."),
            ],
            "Twitch stitches video ads into the stream itself; no request-level blocker can remove those.",
        ),
        "07_memory": bullets(
            "Tabs that give memory back",
            "Three stages, so switching stays instant and idle tabs stop costing anything.",
            [
                ("Hidden, instantly", "Leave a tab and it stops animating, but stays warm. Switching back is immediate."),
                ("Parked after 45 seconds", "Detached, so WebKit suspends its JavaScript and layout entirely."),
                ("Discarded after five minutes", "The web process is torn down and the memory returns. Audible tabs are never touched."),
            ],
        ),
        "08_shortcuts": shortcuts(),
        "09_stack": bullets(
            "It uses the Mac you already bought",
            "No bundled engine. macOS ships a very good one, already wired to\nhardware that Vel could not reach on its own.",
            [
                ("VideoToolbox", "AV1, VP9, HEVC and H.264 decode on the media engine, not the CPU."),
                ("Metal, through Core Animation", "Frames reach the screen as IOSurfaces, with no copy in between."),
                ("Written in Rust", "The whole app is 1.6 MB. No render loop, no background thread, no telemetry."),
            ],
        ),
        "10_plans": plans(),
    }
    for name, img in images.items():
        path = os.path.join(OUT, f"{name}.png")
        img.save(path, "PNG", optimize=True)
        print(f"  {name}.png  {img.size[0]}x{img.size[1]}")


if __name__ == "__main__":
    main()
