# Packaging

`package.sh <os> <arch> <version>` puts a release build into the file the release workflow
publishes and the app's update check reads (`wordsmith-book-<version>-<os>-<arch>.<ext>`):

| Platform | File | Project types |
| --- | --- | --- |
| Linux | `…-linux-x86_64.tar.gz`: the app, `book-cli`, a desktop entry, MIME types, the icon and `install.sh` | `install.sh` registers `.wsbkz` files and Book project folders (`.wsbk`) with the desktop |
| macOS | `…-macos-arm64.dmg`: `Wordsmith Book.app` (with `book-cli` inside) | `Info.plist` declares `.wsbk` (a folder opened as one document) and `.wsbkz` |
| Windows | `…-windows-x64-portable.zip`: the app, `book-cli` and `register-file-types.cmd` | the script registers `.wsbk` and `.wsbkz` for the current user; `unregister-file-types.cmd` removes them |

Builds are not signed yet: signing needs the project's own certificates, set up by a person
(see `docs/progress.md`, Waiting on a person).
