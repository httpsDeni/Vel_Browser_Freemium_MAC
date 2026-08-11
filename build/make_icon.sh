#!/usr/bin/env bash
#
# Builds build/AppIcon.icns from a source image.
#
#   ./build/make_icon.sh build/logo.png
#   ./build/make_icon.sh                  # defaults to build/logo.png
#
# Uses only `sips` and `iconutil`, both of which ship with the Xcode Command
# Line Tools, so this adds no dependency to the project.
#
# The source should ideally be a square PNG, 1024x1024 or larger, containing
# the finished artwork — macOS does not round the corners for you, so whatever
# rounding and padding the icon needs must already be in the file. A
# non-square source is centre-cropped to its shorter side, which is right for
# artwork centred on a plain background and wrong for anything else; crop it
# yourself if the result looks off.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${1:-$ROOT/build/logo.png}"
ICONSET="$ROOT/build/AppIcon.iconset"
ICNS="$ROOT/build/AppIcon.icns"

if [[ ! -f "$SRC" ]]; then
  echo "make_icon: no source image at $SRC" >&2
  echo "Save the logo there (square PNG, 1024x1024 or larger) and run again." >&2
  exit 1
fi

W=$(sips -g pixelWidth  "$SRC" | awk '/pixelWidth/{print $2}')
H=$(sips -g pixelHeight "$SRC" | awk '/pixelHeight/{print $2}')
echo "==> Source ${W}x${H}: $SRC"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
SQUARE="$WORK/square.png"

if [[ "$W" != "$H" ]]; then
  SIDE=$(( W < H ? W : H ))
  echo "==> Centre-cropping to ${SIDE}x${SIDE}"
  sips -c "$SIDE" "$SIDE" "$SRC" --out "$SQUARE" >/dev/null
else
  cp "$SRC" "$SQUARE"
fi

# Master at 1024 so every size below divides down from one resample rather
# than compounding rounding across a chain of them.
sips -z 1024 1024 "$SQUARE" --out "$WORK/master.png" >/dev/null

echo "==> Rendering iconset"
rm -rf "$ICONSET"
mkdir -p "$ICONSET"
for entry in "16:icon_16x16" "32:icon_16x16@2x" "32:icon_32x32" "64:icon_32x32@2x" \
             "128:icon_128x128" "256:icon_128x128@2x" "256:icon_256x256" \
             "512:icon_256x256@2x" "512:icon_512x512" "1024:icon_512x512@2x"; do
  size="${entry%%:*}"
  name="${entry##*:}"
  sips -z "$size" "$size" "$WORK/master.png" --out "$ICONSET/$name.png" >/dev/null
done

echo "==> Packing $ICNS"
iconutil -c icns "$ICONSET" -o "$ICNS"

echo "==> Done"
ls -lh "$ICNS" | awk '{print "    " $9 " (" $5 ")"}'
echo "    Run ./build/bundle.sh to fold it into the app."
