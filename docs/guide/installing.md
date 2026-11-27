# Installing and updating

Wordsmith Book is free and open source. Every release is published on the app's
[releases page](https://github.com/jamescjfoundation/wordsmith-book/releases), and nowhere else.

## The download for your computer

A release has one file for each system, and a list of their checksums. `X.Y.Z` stands for the
version.

| Your computer | File |
| --- | --- |
| Linux, 64-bit Intel or AMD | `wordsmith-book-X.Y.Z-linux-x86_64.tar.gz` |
| Mac with Apple silicon | `wordsmith-book-X.Y.Z-macos-aarch64.dmg` |
| Windows, 64-bit | `wordsmith-book-X.Y.Z-windows-x64-portable.zip` |
| Any | `SHA256SUMS.txt`, the checksums of the three |

Each holds the app (`wordsmith-book`), the command line (`wordsmith-book-cli`, see
[Command line and agents](command-line-and-agents.md)) and the license texts of everything they
are built from, the fonts of the designs included.

The builds are not signed with a publisher's certificate yet. macOS and Windows therefore ask
before they open the app the first time; the steps below say what to choose. To be sure a file
is the one this project published, check it first ([Checking a download](#checking-a-download)).

### Linux

The package needs the C library of Ubuntu 24.04 (glibc 2.39) or a newer one.

1. Unpack the file: `tar -xzf wordsmith-book-X.Y.Z-linux-x86_64.tar.gz`.
2. In the folder it makes, run `./install.sh`. It copies the programs into `~/.local/bin` and the
   menu entry, the icons and the project file types into `~/.local/share`. Nothing needs
   administrator rights. `PREFIX=<folder> ./install.sh` installs under another folder.
3. Start Wordsmith Book from your menu, or run `wordsmith-book`.

`./uninstall.sh` in the same folder removes those files again. Your books and preferences are
left alone.

### macOS

For a Mac with Apple silicon, on macOS 11 or later.

1. Open the `.dmg` file and drag Wordsmith Book onto the Applications folder beside it.
2. Open the app. The first time, macOS says it cannot check the app and does not open it.
3. Open System Settings › Privacy & Security, find the line about Wordsmith Book and choose
   Open Anyway. On macOS 14 and earlier you can instead hold Control, click the app, choose Open,
   then Open again.

After that it opens like any other app. The command line is inside the app:
`/Applications/Wordsmith Book.app/Contents/MacOS/wordsmith-book-cli`.

### Windows

1. Unpack the `.zip` file anywhere you like: a folder in your Documents, a USB stick. Nothing is
   installed system-wide.
2. Open `wordsmith-book.exe` in the folder. The first time, Windows shows "Windows protected
   your PC": choose More info, then Run anyway.
3. To open books from File Explorer, run `register-file-types.cmd` in the same folder. It
   registers `.wsbk` and `.wsbkz` for your account, with the program where it now is; run it
   again if you move the folder. `unregister-file-types.cmd` takes the registration away.

## Checking a download

`SHA256SUMS.txt` lists the SHA-256 checksum of every file of the release. With the file and the
list in one folder:

```sh
sha256sum --check --ignore-missing SHA256SUMS.txt        # Linux
shasum -a 256 --check --ignore-missing SHA256SUMS.txt    # macOS
```

On Windows, `Get-FileHash <file>` in PowerShell prints the checksum to compare with the list.

Every file also has a build provenance attestation: a signed statement, kept by GitHub, that the
file was built from this repository by the release workflow. With the
[GitHub CLI](https://cli.github.com/):

```sh
gh attestation verify wordsmith-book-X.Y.Z-linux-x86_64.tar.gz \
  --repo jamescjfoundation/wordsmith-book \
  --signer-repo jamescjfoundation/open-source-core
```

`--signer-repo` names open-source-core because the jobs that build the files are that
repository's shared release workflow.

## Updates

When the app starts it looks for a newer release, and tells you when there is one. File › About
shows the version you have, with Check for Updates to look at once.

| System | What an update does |
| --- | --- |
| Linux | A copy installed under your home folder downloads the new package, checks it against the release's checksums, replaces itself and starts the new version when you choose Install and Restart. A copy somewhere a package manager or an administrator put it is shown the release page |
| macOS | The app shows the release page; download the new `.dmg` and replace the app |
| Windows | The app shows the release page; download the new `.zip` and unpack it over the old folder, or beside it. Installing in place needs a signed build, which there is not yet |

Save your book before you choose Install and Restart, or let Book save it on its own as set in
Preferences: the app closes to start the new version.

## What the app sends

- **Nothing about you or your books.** The app has no telemetry: it does not report usage,
  crashes or anything else, and your text, designs, house file and file names never leave your
  computer.
- **The update check is the only request the app makes.** When it starts, and when you choose
  Check for Updates, it asks GitHub (`api.github.com`) for this app's list of releases. GitHub
  sees your computer's network address and a line that names the app and its version, and
  nothing else. An update is downloaded only when you choose to install it, from the release on
  `github.com`.
- **To turn the check off**, clear "Check for updates when the app starts" in File › Preferences.
  Nothing is then sent unless you choose Check for Updates yourself.
- **The command line never checks.** `wordsmith-book-cli` makes no network request at all.
- **Checking programs run on your own computer.** The EPUB validator, the accessibility checker
  and the PDF validator ([Previews and preflight](preview-and-preflight.md)) are programs you
  installed and named in Preferences; Book starts them on the files it built and reads what
  they report. *Open in a Reading App* hands the ebook to the reading app on your computer.
  Book sends nothing to any of them over a network; what such a program does itself is its own.

## What the app keeps on your computer

Beside your books, the app keeps its preferences in its settings folder:

| System | Settings folder |
| --- | --- |
| Linux | `~/.config/wordsmith-book` |
| macOS | `~/Library/Application Support/Wordsmith Book` |
| Windows | `%APPDATA%\Wordsmith Book` |

Deleting the folder puts the app back as it was when first started. Your books are not in it.
