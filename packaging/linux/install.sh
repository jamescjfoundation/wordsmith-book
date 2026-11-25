#!/bin/sh
# Install Wordsmith Book for the current user and register its project types.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
prefix=${PREFIX:-$HOME/.local}
mkdir -p "$prefix/bin" "$prefix/share/applications" "$prefix/share/mime/packages" "$prefix/share/icons/hicolor/256x256/apps"
cp "$here/wordsmith-book" "$here/book-cli" "$prefix/bin/"
cp "$here/org.jcjfoundation.book.desktop" "$prefix/share/applications/"
cp "$here/org.jcjfoundation.book.xml" "$prefix/share/mime/packages/"
cp "$here/org.jcjfoundation.book.png" "$prefix/share/icons/hicolor/256x256/apps/"
command -v update-mime-database >/dev/null && update-mime-database "$prefix/share/mime" || true
command -v update-desktop-database >/dev/null && update-desktop-database "$prefix/share/applications" || true
echo "Wordsmith Book is installed in $prefix/bin."
