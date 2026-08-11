#!/usr/bin/env bash
#
# Builds the application bundle.
#
# The bundle is not cosmetic. A bare binary run from a terminal gets no bundle
# identity, and WebKit uses that identity for the on-disk website data store —
# so cookies, logins and the compiled blocklist cache would be filed under the
# terminal and shared with every other unbundled binary. The menu bar and the
# `NSApplicationActivationPolicyRegular` activation also depend on it.
#
#   ./build/bundle.sh            release build into dist/
#   ./build/bundle.sh --debug    faster build, same bundle layout
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE="release"
CARGO_FLAGS=(--release)
if [[ "${1:-}" == "--debug" ]]; then
  PROFILE="debug"
  CARGO_FLAGS=()
fi

APP_NAME="${APP_NAME:-Vel}"
BUNDLE_ID="app.vel.browser"

APP="$ROOT/dist/$APP_NAME.app"
CONTENTS="$APP/Contents"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"

echo "==> Building ($PROFILE)"
cargo build "${CARGO_FLAGS[@]}" --manifest-path "$ROOT/Cargo.toml" -p vel

echo "==> Assembling $APP"
rm -rf "$APP"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"
cp "$ROOT/target/$PROFILE/vel" "$CONTENTS/MacOS/vel"

# The icon is optional so a fresh clone builds without artwork; regenerate it
# with ./build/make_icon.sh.
ICON_KEY=""
if [[ -f "$ROOT/build/AppIcon.icns" ]]; then
  cp "$ROOT/build/AppIcon.icns" "$CONTENTS/Resources/AppIcon.icns"
  ICON_KEY=$'\t<key>CFBundleIconFile</key>\n\t<string>AppIcon</string>'
else
  echo "    (no build/AppIcon.icns — building without an icon)"
fi

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>${APP_NAME}</string>
	<key>CFBundleIdentifier</key>
	<string>${BUNDLE_ID}</string>
	<key>CFBundleExecutable</key>
	<string>vel</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
${ICON_KEY}
	<key>CFBundleVersion</key>
	<string>${VERSION}</string>
	<key>CFBundleShortVersionString</key>
	<string>${VERSION}</string>
	<key>LSMinimumSystemVersion</key>
	<string>13.0</string>
	<key>LSApplicationCategoryType</key>
	<string>public.app-category.productivity</string>
	<key>NSHighResolutionCapable</key>
	<true/>

	<!-- Lets macOS keep the discrete GPU asleep. On a video page the decode
	     runs on the media engine and compositing is trivial, so waking a
	     second GPU would cost battery and buy nothing. -->
	<key>NSSupportsAutomaticGraphicsSwitching</key>
	<true/>

	<!-- A browser has to be able to load plaintext http:// pages. This key
	     relaxes App Transport Security for *web content only*: requests the
	     app itself makes are still held to ATS, and HSTS still applies, so
	     this does not downgrade any site that asked not to be downgraded. -->
	<key>NSAppTransportSecurity</key>
	<dict>
		<key>NSAllowsArbitraryLoadsInWebContent</key>
		<true/>
	</dict>

	<key>CFBundleDocumentTypes</key>
	<array>
		<dict>
			<key>CFBundleTypeName</key>
			<string>Web Page</string>
			<key>CFBundleTypeRole</key>
			<string>Viewer</string>
			<key>LSItemContentTypes</key>
			<array>
				<string>public.html</string>
			</array>
		</dict>
	</array>
</dict>
</plist>
PLIST

# Ad-hoc signature. Unsigned bundles are refused outright on Apple silicon;
# this is enough to launch locally. Shipping to anyone else needs a Developer
# ID signature and notarisation instead.
echo "==> Signing (ad-hoc)"
codesign --force --deep --sign - "$APP"

echo "==> Done: $APP"
du -sh "$APP" | sed 's/^/    /'
