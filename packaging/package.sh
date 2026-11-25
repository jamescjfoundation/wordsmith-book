#!/usr/bin/env bash
# Put a release build into its release file: package.sh <linux|macos|windows> <arch> <version>.
# Run from the repository root after `cargo build --release --locked -p wordsmith-book -p wordsmith-book-cli`.
set -euo pipefail
os=$1 arch=$2 version=$3
stem="wordsmith-book-${version}-${os}-${arch}"
out=dist
work="$out/$stem"
rm -rf "$work" && mkdir -p "$work"
icon=assets/brand/icon-256.png
mac_icon=assets/brand/icon-macos-1024.png
case "$os" in
  linux)
    cp target/release/wordsmith-book target/release/book-cli "$work/"
    cp packaging/linux/org.jcjfoundation.book.desktop packaging/linux/org.jcjfoundation.book.xml packaging/linux/install.sh "$work/"
    cp "$icon" "$work/org.jcjfoundation.book.png"
    cp LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md "$work/"
    tar -C "$out" -czf "$out/$stem.tar.gz" "$stem"
    echo "$out/$stem.tar.gz"
    ;;
  macos)
    app="$work/Wordsmith Book.app/Contents"
    mkdir -p "$app/MacOS" "$app/Resources"
    cp target/release/wordsmith-book target/release/book-cli "$app/MacOS/"
    sed "s/@VERSION@/$version/g" packaging/macos/Info.plist > "$app/Info.plist"
    iconset="$work/book.iconset" && mkdir -p "$iconset"
    for s in 16 32 128 256 512; do
      sips -z $s $s "$mac_icon" --out "$iconset/icon_${s}x${s}.png" >/dev/null
      sips -z $((s * 2)) $((s * 2)) "$mac_icon" --out "$iconset/icon_${s}x${s}@2x.png" >/dev/null
    done
    iconutil -c icns "$iconset" -o "$app/Resources/book.icns" && rm -rf "$iconset"
    cp LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md "$work/"
    hdiutil create -volname "Wordsmith Book" -srcfolder "$work" -ov -format UDZO "$out/$stem.dmg" >/dev/null
    echo "$out/$stem.dmg"
    ;;
  windows)
    cp target/release/wordsmith-book.exe target/release/book-cli.exe "$work/"
    cp packaging/windows/register-file-types.cmd packaging/windows/unregister-file-types.cmd "$work/"
    cp LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md "$work/"
    (cd "$out" && 7z a -tzip "$stem-portable.zip" "$stem" >/dev/null)
    echo "$out/$stem-portable.zip"
    ;;
  *) echo "unknown platform $os" >&2; exit 2 ;;
esac
