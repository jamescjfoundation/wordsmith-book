# Changelog

What changed in each release of Wordsmith Book, newest first, for people who make books with
it. `## Unreleased` holds what the next release will bring; when that release is cut, the
section takes its version and date (`## 0.9.0 (2026-12-01)`), and the next `## Unreleased` opens
above it.

The first public release is 0.9.0, a preview; 1.0.0 follows it (decision 0005). Nothing was
published before it: the versions under "Before the first release", at the end, were the stages
of the build plan, each prepared as a release and none released.

## 0.9.0 (2026-11-27)

The first public release of Wordsmith Book, 0.9.0: a preview of 1.0.0, for trying the app, its
downloads and its update check. Everything the app does is in it. The user guide is at
<https://wordsmithand.ink/wiki/book/>, and in `docs/guide/` of the repository.

### Getting it

- One download for each system, and a list of their checksums:
  `wordsmith-book-0.9.0-linux-x86_64.tar.gz` (Linux, 64-bit Intel or AMD; needs the C library of
  Ubuntu 24.04 or newer), `wordsmith-book-0.9.0-macos-aarch64.dmg` (a Mac with Apple silicon,
  macOS 11 or later), `wordsmith-book-0.9.0-windows-x64-portable.zip` (Windows, 64-bit; nothing
  is installed system-wide) and `SHA256SUMS.txt`.
- **The builds are not signed with a publisher's certificate yet.** macOS and Windows ask before
  they open the app the first time. On macOS: System Settings › Privacy & Security › Open
  Anyway. On Windows: More info, then Run anyway. The guide's "Installing and updating" page has
  the steps for each system, and how to check a download against its checksum and its build
  attestation.
- Each download holds the app (`wordsmith-book`), the command line (`wordsmith-book-cli`) and
  the license texts of everything they are built from, the fonts of the designs included. The
  Linux package has an install script that puts the app in your menu and registers the project
  file types for your account; the Windows package has a script that registers them.
- The app looks for a newer release when it starts, and File › About has Check for Updates. On
  Linux a copy installed under your home folder replaces itself and starts the new version; on
  macOS and Windows the app shows the release page. The check is the only request the app
  makes, and File › Preferences turns it off.

### What the app does

- **Sources.** Link a Wordsmith Manuscript project, which Book reads and never changes (a
  correction goes back to the writer as an offered fix), or import DOCX, ODT, RTF, Markdown,
  text or EPUB into a source of the book's own, with a review before the import and a cleanup
  of quotes, dashes and spaces after it. A new version of the file is compared word by word
  before it replaces the old text. Every import is checked word for word.
- **Structure.** Parts and chapters numbered the way you choose; a half title, title page,
  copyright page and contents made from the book's details; a dedication, a foreword, an
  epilogue, a glossary and the other kinds of front and back matter, each placed by convention; parts that appear only in print or only in the ebook; box
  sets of several books in one.
- **Designs.** Thirteen designs to filter by genre and mood, each previewed with your own first
  chapter; parts mixed between them; your own designs saved and shared as a file; a large-print
  variant of any of them.
- **Print.** Eighteen trim sizes, paperback and hardcover profiles, mirrored margins with a
  gutter by page count, bleed and full-page pictures, footnotes on the page that calls them,
  asides, and hints for words that must not break. A full-wrap cover template from the page
  count and paper, and a check of the cover you bring. PDF/A-2b and PDF/UA-1, and proofs with a
  watermark and numbered lines.
- **Ebook.** EPUB 3.3 with navigation, accessibility metadata, required alternative text,
  embedded fonts, pop-up notes and the print edition's page numbers. Store editions, each with
  only its own links.
- **Checking.** Print spreads and an ebook preview at phone, e-ink reader and tablet sizes.
  Preflight on the built files, each finding with a button to its page or setting; typographic
  warnings by page; the standard EPUB, accessibility and PDF validators when you installed
  them. Export writes nothing when a check fails, and the same book gives the same bytes every
  time.
- **A publishing house.** One house file a press shares: imprints, series and ISBNs. House
  designs that stay put, title and copyright pages from the imprint, a series line and "Also
  by" from the series, each ISBN used once, and an ONIX for Books 3.0 record for each edition.
- **Files.** A book is a folder of plain text files (`.wsbk`), or one packed file (`.wsbkz`).
  Saving writes only what changed, each file safely; unsaved work is kept for recovery; Back Up
  Now keeps the newest ten backups beside the book.
- **The interface** in English and Spanish, light and dark, named for screen readers
  throughout, with every command in the command palette and every key yours to change.

### The command line and agents

- `wordsmith-book-cli` does the app's work without its window: `build`, `check`, `verify`,
  `convert`, `report`, `run` and `commands`. It makes no network request.
- One convention with the other Wordsmith apps: a result on standard output; a reason on
  standard error; exit code 0 when it did what was asked, 1 when it could not, 2 when the
  command line was wrong and nothing was done, and 3 when a check of the content did not pass
  (`verify` found the words differ, or preflight blocks export).
- `wordsmith-book-cli mcp [<file>]` serves every Book command to an agent as a tool named
  `book_` and the command's id. `--read-root <folder>` and `--write-root <folder>` keep the
  files an agent's commands name to folders you choose.
- `wordsmith-book --version` and `wordsmith-book-cli --version` print the program's name and
  version.

### With Wordsmith Manuscript

- This release reads Manuscript project formats 1 to 3 and book view version 1: every project
  Wordsmith Manuscript 0.9.0 saves. A project saved in a newer format is refused in Book's own
  words, naming both versions, until Book is updated.

### Not yet

- Signed builds, and with them an update installed in place on Windows.
- Words hyphenated by Book itself: in print a word breaks only where the manuscript or your
  line-break hints allow it (the guide's Print page, "Line breaks and hyphens").
- Command replies (what agents read), preflight findings and build messages are in English
  whatever the interface language.

### If you built the app from source before this release

What changed since the last stage of the build plan was prepared (1.0.0, below):

- **The command-line program is `wordsmith-book-cli`** (it was `book-cli`), in its file name,
  its usage, its version line and the start of every message. Scripts and agent configurations
  that name the old program must be changed. Every program of a release is named after the
  app, so two Wordsmith apps installed in one folder share no name.
- **Release files have the names above, with checksums and license texts.** The earlier, unused
  release workflow named the macOS file `…-macos-arm64.dmg`, which the update check does not
  find, published no checksum file and put no third-party license texts in a package. Releases
  are made by the shared workflow of open-source-core now
  ([core decision 0019](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0019-app-releases.md)),
  started by hand on `main`; a pushed tag starts nothing. `packaging/` is gone: `cargo xtask
  package` writes the menu entry, the file types and the scripts from data.
- **On Windows an update is finished by the program itself.** The updater ends its work through
  a copy of the app started with its own arguments; the app now hands them over before it reads
  its command line. A project whose name is not Unicode opens (the program stopped on one).
- On Linux a `.wsbkz` file is told by its name, no longer by its first bytes. The macOS app's
  file types are named `org.jcjfoundation.book.wsbk` and `.wsbkz`.
- **`UPDATE_GOLDEN=1` rewrites the golden files and the guide's page of keys** (it was
  `BOOK_BLESS=1`), the switch Wordsmith Script and Wordsmith Manuscript use.
- The brand files in `assets/brand/` are held to the brand kit's by `cargo xtask brand`, in
  the gate; the files themselves did not change.
- The guide has pages for installing and updating and for files, shows its pictures, and names
  features without their ids, which are in one index page.

## Before the first release

The four sections below are the stages of the build plan (`docs/PLAN.md`), written as each was
finished, between 2026-11-24 and 2026-11-26. None was published: no release and no tag carries
any of these numbers. They are kept as the record of what each stage built. Where they say
`book-cli`, the program is `wordsmith-book-cli` since 0.9.0; "the first release" in the oldest
one meant the stage, not a publication.

### 1.0.0 (prepared, never released)

A publishing house: shared imprints, series and ISBNs, house designs that stay put, and the
record distributors read.

#### Added

- **An agent can be kept to folders.** `book-cli mcp [<file>] [--read-root <folder>]
  [--write-root <folder>]`: the two folders are the only ones the agent's commands may read
  and write files in. The project named on the command line is yours and is saved where it
  lives; a project the agent opens or makes is read only inside the read folder and changed
  only inside the write folder. With neither flag nothing is confined, as before, and
  `book-cli` now says so when it starts.
- **The house file.** A press keeps its imprints, series and the ISBNs it owns in one
  `house.toml` (its path in Preferences). A book joins an imprint, a series, or both, and keeps a
  copy of what it takes, so it builds the same anywhere.
- **House styles.** An imprint or series can lock its house design: its books keep it until they
  leave, whatever command or agent asks.
- **Imprint profiles.** The imprint's name, address, website, logo and copyright wording on the
  title and copyright pages, in print and in the ebook.
- **Series.** The copyright page's series line, "Also by" built from the series, and a series
  view that names any book off the series' design.
- **ISBN list.** Each of the house's ISBNs checked and used once; an edition takes the next free
  one and the house file records who has it.
- **ONIX for Books 3.0.** One record for each edition, validated against the published schema
  and its code lists.

#### Changed after the stage was prepared (2026-11-26)

- **The command line follows one convention**, the same as every app built on the same base
  (core decision 0017), so a script that has learned one app's command line knows the next.
  What a script or a person will notice:
  - **Exit code 3** is new: the command ran, and a check of the content did not pass. `verify`
    ends with 3 when the words differ (it was 1), `check` with 3 when something blocks export
    (it was 1), and `build` with 3 when a check stops it (it was 1). What the check found is
    now on standard output, as a result; it was on standard error.
  - **Exit code 1** is "it could not be done": a project or a file that cannot be opened or
    read, a project that is there already, a failed import, a file of a kind Book cannot check.
    These were 2. **Exit code 2** is now only a wrong command line.
  - Run with **no argument**, `book-cli` prints the usage on standard error (it was standard
    output); the code is still 2.
  - **`--version`** and **`-V`** print `book-cli` and its version.
  - What went wrong is written as **`book-cli: <reason>`**.
  - `commands` refuses an argument it does not take.
- **Agents see only Book's own tools.** `book-cli mcp` serves the `book_…` tools, under the
  same names as before. The general document tools it also listed (`execute`, `list_commands`,
  typing, inspecting, rendering, opening and saving a document) are gone: they worked on an
  empty document Book never had. `book_isbn_list` and `book_cover_check` now say they only
  read.
- **The window's *Open…* is the system's Open dialog**, in place of a box to type a path into.
  Choose a project's `book.toml`, or a packed project (`.wsbkz`); where the system shows a
  `.wsbk` folder as one document, choose that. A path can still be given when starting the app.
- **A Manuscript project Book cannot read is refused in Book's own words**, naming both
  versions: the project's format, and the formats this version of Book reads, with what to do
  ("Update Wordsmith Book to open it."). Before, a project saved by a newer Manuscript was
  refused with Manuscript's message, which said to update "the app". Book reads Manuscript
  project formats 1 to 3 and book view version 1.
- **The Wordsmith Book brand.** The app icon, the mark (the letter B and a ribbon cursor) and
  the window's colors come from the Wordsmith brand kit: Iron Gall Ink on Page White, Margin Blue
  for links and focus, Ribbon Green as the one accent, and Night for the dark themes. The Windows
  program carries the icon, so its file types show it.
- Print follows the newest core release's layout (core 0.1.15); no core move changed the
  golden files, and a test keeps the golden folders exactly what the tests compare.
- Book's own pages (the title page) can carry a picture: the imprint's logo.
- In Details, each edition's other settings (*Take an ISBN from the House*, *PDF/A and PDF/UA*)
  are on a line of their own.

### 0.3.0 (prepared, never released)

Designs, series and standards: thirteen designs, large print from any of them, box sets, EPUB
import, PDF/A and PDF/UA, and proofs.

#### Added

- **Designs.** Seven more: Memoir, Historical, Young Adult, Horror, Cozy, Poetry and Large Print
  (thirteen in all), each with its genres and moods for the chooser.
- **Large print.** A large-print edition sets the book in the large-print variant of whatever
  design it has: all text 16 pt and up, ragged right, no hyphenation, headings grown to match;
  preflight holds it to that.
- **Box sets.** Several books in one, each a part with its own front and back matter, under one
  main contents; every word of every book checked in every edition.
- **EPUB import.** An EPUB imports with its parts, chapters, front and back matter and notes
  taken from its own structure; the ebook's own contents page is left out (Book makes the
  contents again). An EPUB Book exported imports back word for word.
- **PDF/A-2b and PDF/UA-1.** A print edition can declare both on its PDF (archival and
  accessible); preflight checks the declarations and runs a PDF validator you installed. The
  test book's PDF passes the standard validator for both (core 0.1.15).
- **Proofs.** A print edition's proof: PROOF across every page and every line numbered, for
  reading and marking; written only when asked for, never as part of an export.

#### Changed

- Front and back matter inside the body (a box set's book keeps its own) is set as matter, as
  at a book's front or back.

### 0.2.0 (prepared, never released)

Parity for most novels: every common trim, hardcovers, bleed, notes, asides, more front and back
matter, store editions, covers, re-import, typographic checks, an accessibility checker,
line-break hints, a Spanish interface and screen-reader names throughout.

#### Added

- **Trims.** Eighteen sizes in the print-on-demand and generic profiles, metric ones included,
  each with its gutter table and page limits.
- **Hardcover.** A hardcover edition on its own profile (`pod-hardcover`): case-laminate sizes,
  wider margins for the hinge, its page limits.
- **Bleed and full-page pictures.** An edition set to bleed gets pages larger than the trim, trim
  and bleed boxes in the PDF (core 0.1.11), and pictures that fill their page to the edge.
- **Notes.** Footnotes at the foot of the page that calls them, continued on the next page when
  long; or notes at each chapter's end or the book's end. In the ebook, pop-up footnotes or
  endnotes per edition, linked both ways. The core's EPUB writer keeps notes too (core 0.1.12).
- **Asides.** Letters, notes, signs and messages styled by the design (inset, italic, framed or
  centred), in print and in the ebook.
- **More front and back matter.** Foreword, preface, introduction, prologue, epilogue,
  afterword, a newsletter or website page, an excerpt from the next book, a glossary, endnotes and
  discussion questions, each placed where convention puts it.
- **Print-only and ebook-only.** Per chapter, per element and per paragraph.
- **Store editions.** Ebook editions named for a store, so links and pages meant for one store
  show only in its edition.
- **Page list.** An ebook edition carries the print edition's page numbers, so readers and
  teachers can cite the same page.
- **Covers.** The full-wrap cover template for a paperback or hardcover (PDF and PNG: trim,
  spine, bleed, safe areas, the barcode area), and a supplied cover checked against it.
- **Re-import.** A new version of the file a book was imported from, compared section by section
  and word by word before it replaces the old text; Book's settings are kept.
- **Typographic warnings.** Widows, orphans, stacked hyphens, loose lines and rivers, by page, in
  preflight.
- **Accessibility checker.** An EPUB accessibility checker you installed runs in preflight beside
  the standard EPUB validator; its findings link to their files.
- **Line-break hints.** Words and phrases never split across lines, and words hyphenated only
  where you say, applied in print and kept in the ebook without changing your manuscript.
- **Interface languages.** English and Spanish, through the core's translation catalogs (core
  0.1.13); the language follows your computer unless you choose one in Preferences.
- **Screen readers.** Every button, field, link, tab and page preview has a name a screen reader
  reads (the core named its last two unnamed fields in 0.1.14).

#### Changed

- The ebook's page list names its source with `dc:source`, refined as the source of the
  pagination, which both the standard EPUB validator and accessibility checkers read.
- Book's word checks leave soft hyphens out of words, so hinted words still match the source.

#### Known limits

- Command replies (what agents read), preflight findings and build messages are in English.
- The core's layout does not hyphenate on its own; words break only at soft hyphens (from the
  manuscript or from Book's hints).

### 0.1.0 (prepared, never released)

The first release: a finished manuscript to a print-ready paperback PDF and a valid, accessible
EPUB, every word checked against the source.

#### Added

- **Sources.** Link a Wordsmith Manuscript project (read, never written, followed when it
  changes), or import a DOCX, ODT, RTF, Markdown or text file into a source of the book's own,
  with a review of the structure found, the formatting kept and dropped, a cleanup of quotes,
  dashes, ellipses and spaces, and a word-for-word check of every import.
- **Structure.** The outline; chapter and part numbering; a half title, title page, copyright
  page and contents made from the book's details; twenty kinds of front and back matter; Book's
  own settings per chapter (subtitle, epigraph, unnumbered, print or ebook only); light fixes
  offered back to the writer.
- **Designs.** Six designs (Classic, Literary, Romance, Thriller, Fantasy, Non-fiction), parts
  mixed between them, your own designs saved, exported and imported, and a chooser that previews
  your own first chapter.
- **Print.** Four trims, mirrored margins with a gutter by page count, right-hand chapter
  openings, drop caps and small-capital lead-ins, running heads and folios, notes at chapter
  ends, pictures; a print-on-demand paperback profile and a generic one; PDFs with every font
  embedded and no annotations or bookmarks.
- **Ebook.** EPUB 3.3 with navigation, landmarks, an NCX, accessibility metadata, required
  alternative text, embedded fonts per edition and a cover.
- **Previews.** Print spreads with guides, drawn in the background; an ebook preview at phone,
  e-ink reader and tablet sizes with text sizes and a dark page.
- **Preflight and export.** Every rule checked on the built files, each finding linked to its
  page or section; the standard EPUB validator run when installed; export that writes nothing if
  a check fails; a project bundle; the same files from the same project every time.
- **Automation.** `book-cli` (build, check, verify, convert, report, run, commands, mcp) and
  every command as an agent tool.
- The user guide (`docs/guide/`), packages for three desktop platforms with Book's project
  types registered, and update checks from this repository's releases.
