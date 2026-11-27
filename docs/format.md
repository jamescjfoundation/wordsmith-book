# The Book project format

Version 1 (`schema = 1`). Version 1 was extended in place, with no migrations, until Book's
first release, 0.9.0 (decision 0005). From that release on, every change raises the version and
adds a migration and a fixture of the old version (plan section 6.1, decision 0001).

A Book project is a folder ending in `.wsbk`; packed into one file it ends in `.wsbkz`
(decision 0002). Every file is plain text except pictures and fonts, so a project can be kept in
version control. The same project always gives the same bytes.

```text
My Book.wsbk/
  book.toml               Book's own layer: the source, editions, the design, elements,
                          settings per section, metadata additions, cleanup, fixes
  designs/<id>.toml       the user's own designs (docs/design-format.md)
  source.wsm/             an owned source: a Wordsmith Manuscript project folder
  assets/                 cover images, pictures added in Book, fonts the user added
  builds/<edition>.json   the last build's record
  .recovery/              unsaved work while the app runs (never packed)
```

Reading never trusts the files: `book.toml` is at most 4 MB and nests at most 32 tables or arrays
deep; a project holds at most 100 editions, 1,000 elements, 100,000 section settings and 100,000
fixes; design files are at most 1 MB (1,000 of them), assets 256 MB each (10,000, 2 GB in all),
build records 16 MB (100). File names inside the project have no folders above them, no hidden
names and no control characters. A project from a newer version is refused with a message
naming both versions. Folders under `designs/`, `assets/` and `builds/` are read 16 levels deep;
a symbolic link is never followed, `book.toml` included, and any other file in the folder is
the user's: never read, packed, changed or removed.

The folder is a project folder as open-source-core specifies it (its `crates/jcj-project/FOLDER.md`,
core decision 0018), and Book reads and writes it through the core's `jcj_project::folder`:
`book.toml` is the manifest, written last alone; a path belongs to the project when no part of it
is hidden and it holds no control character; Book keeps no conflict copies. What each file holds,
and the limits above, are Book's.

The source is a Wordsmith Manuscript project, and Book states which it reads in one place
(`MANUSCRIPT_SCHEMAS` and `BOOK_VIEWS` in `crates/book-format/src/lib.rs`): project formats 1 to
3 (the `schema` of the project's `project.toml`) and book view version 1. A linked project, a
packed one or an owned source outside that range is refused when it is opened, in Book's own
words, naming the version found and the versions Book reads.

Saving writes only the files whose bytes changed, each atomically (written beside, then moved
into place), `book.toml` last, so an interrupted save leaves the previous `book.toml` pointing
at whole files.

## `book.toml`

Every key but `schema` and `source` may be left out; defaults are shown in comments. Keys are
written in this order; empty values are left out.

<!-- example: book.toml -->
```toml
# A Wordsmith Book project (docs/format.md).
schema = 1
date = "2026-11-24"   # the date builds carry, so a build repeats exactly
decorative = ["media/rule.png"]   # pictures that need no alternative text (SPEC 6.6)

[source]
manuscript = "../The Lighthouse.wsm"   # or: owned = "source.wsm"

[design]
id = "classic"          # a built-in design or designs/<id>.toml

[design.mix]            # parts taken from other designs (DSN-3)
chapter-opening = "fantasy"
scene-breaks = "literary"

[numbering]
chapters = "label-words"   # label-words, label-figures, words, figures, roman, none
parts = "words"
chapter_label = "Chapter"
part_label = "Part"

[metadata]
subject_codes = ["FIC019000"]
copyright_holder = "The Estate of the Author"
about_author = "The author lives by the sea."

[[metadata.contributors]]
name = "A. Translator"
role = "translator"   # author, editor, translator, illustrator, foreword, introduction, cover-designer, narrator

[metadata.imprint]
name = "Harbour Light Books"
address = ["1 Quay Street", "Port Town"]
website = "https://example.org"

[metadata.overrides]
subtitle = "A Novel"

[[metadata.also_by]]
title = "The Second Tide"
series = "The Lighthouse Books"

[[editions]]
id = "paperback"
name = "Paperback"
format = "paperback"     # paperback, hardcover, ebook, large-print
isbn = "978-0-306-40615-7"
trim = "6x9"
paper = "cream"          # white, cream, color
profile = "pod-paperback"
bleed = true             # pages bleed (default false): full-page pictures run to the edge
price_notes = "List at the usual trade price."

[editions.overrides]
"body.size" = "11.5"

[[editions]]
id = "ebook"
name = "Ebook"
format = "ebook"
notes = "chapter-end"    # popup (default: footnotes in print, pop-up notes in ebooks), chapter-end, book-end
embed_fonts = false      # default true
page_list = true         # the print edition's page numbers in the ebook (EBK-6)
cover = "assets/cover.jpg"   # the ebook cover, a PNG or JPEG in the project (EBK-5)

[[elements]]
id = "dedication"
kind = "dedication"
placement = "front"      # front (default) or back
text = "For the keepers of lights."

[[elements]]
id = "copyright"
kind = "copyright"
text = "Copyright © {year} {holder}. All rights reserved.\nISBN {isbn}"

[[elements]]
id = "newsletter"
kind = "newsletter"
placement = "back"
only = "ebook"           # both (default), print, ebook
stores = ["store-one"]

[sections."01965f2a-7c00-7000-8000-000000000001"]
subtitle = "In Which the Light Goes Out"
epigraph = "The sea remembers."
epigraph_source = "A saying of the coast"

[sections."01965f2a-7c00-7000-8000-000000000002"]
only = "print"
unnumbered = true

[[sections."01965f2a-7c00-7000-8000-000000000002".blocks_only]]
starts = "A note for readers of this edition"   # the paragraph's first words, unique in the section
only = "ebook"                                 # print or ebook (STR-9)

[cleanup]
keep_exactly = false
off = ["dashes"]

[cleanup.rules]
off = ["empty-paragraph-breaks"]
chapter_words = ["Book"]
break_marks = ["~"]

[[fixes]]
id = "fix-1"
section = "01965f2a-7c00-7000-8000-000000000001"
find = "teh harbour"
replace = "the harbour"
note = "typo"
made = "2026-11-24"
offered = true
```

- `source`: exactly one of `manuscript` (a path from the Book project's folder to a Manuscript
  project folder or `.wsmz`; Book never writes it) or `owned` (a `.wsm` folder inside the
  project that Book owns).
- Section ids are the source's stable ids (UUIDs). Settings for a section the source no longer
  has are kept and ignored.
- `fixes`: for a linked source, `offered` is true once the offer is in the offers file beside the
  Manuscript project (plan section 6.4); the fix's id is the offer's id.

### Version 0 (a draft, never released)

Version 0 exists only to build and test the migration path (plan M1.1): the source was a table
with `kind` and `path`, and the design a plain string. Book reads it and saves it as version 1.

<!-- example: book.toml version 0 -->
```toml
schema = 0
design = "classic"

[source]
kind = "owned"
path = "source.wsm"
```

The fixture `tests/fixtures/projects/version-0.wsbk/` holds this file; a test opens it.

## `designs/<id>.toml`

A design file, as `docs/design-format.md` describes (step M4.1). The id is the file name: letters,
digits, `-` and `_`.

## `builds/<edition>.json`

The last build of an edition: what it was built from and where each section landed, so Book can
say what changed since the last export.

<!-- example: builds/paperback.json -->
```json
{
  "edition": "paperback",
  "date": "2026-11-24",
  "app": "0.1.0",
  "core": "v0.1.7",
  "design": "classic",
  "pages": 312,
  "files": { "pdf": "6f1c…" },
  "sections": [
    { "id": "01965f2a-7c00-7000-8000-000000000001", "page": 1, "words": 3120, "checksum": "9b2e…" }
  ]
}
```

## `assets/`

Pictures (PNG, JPEG) and fonts the user added (TTF, OTF), each font with its license file beside
it (`assets/fonts/<name>.ttf` and `assets/fonts/<name>.txt`). Names follow the rules above.

## `house.toml` (a publishing house's file, not part of a project)

Version 1. Shared by the books of a house (DSN-7, META-3 to META-5); a book records what it took
from it under `[house]` in its own `book.toml` (`imprint`, `series`, `series_number`, and the
design `lock`: who locks it and the design), so it builds the same without the house file.

```toml
schema = 1
name = "Lantern House"

[[imprints]]
id = "lantern"                 # lowercase letters, digits, hyphens
name = "Lantern Press"
address = ["1 Harbour Street", "Port Town"]
website = "lantern.example"
logo = "logos/lantern.png"     # beside the house file
copyright = "Copyright © {year} {holder}\nPublished by {imprint}"  # empty: Book's wording
design = "classic"             # a built-in design, or designs/<id>.toml beside the house file
lock_design = true             # its books keep that design

[[series]]
id = "voyages"
name = "Voyages"
imprint = "lantern"
design = ""                    # overrides the imprint's when set
lock_design = false
books = [{ number = 1, title = "The First Voyage", project = "books/first.wsbk" }]

[[isbns]]
isbn = "978-0-306-40615-7"
book = "The First Voyage"      # empty: free
edition = "Paperback"
```

At most 10,000 imprints, series and ISBNs each; the file at most 4 MB, each file beside it at
most 16 MB, by paths that stay beside it.

## Packed projects (`.wsbkz`)

One file holding a whole project, for sending, backups and recovery (core decision 0009): a zip
whose first entry is `mimetype` (stored, `application/x-wordsmith-book-packed`), then every file
in name order, with fixed times, so the same project packs to the same bytes.

```text
mimetype
book.toml
designs/<id>.toml
assets/<path>
builds/<edition>.json
source.wsm/<path>        the owned source's files, when the book has one
linked.wsmz              a packed copy of a linked Manuscript project (read-only), so the
                         project travels whole; read when the linked path is not found
```

Reading never trusts the file: at most 2 GB packed, 50,000 entries, 512 MB an entry and 4 GB
unpacked; entries are read one by one into memory (never extracted); names are checked (no
folders above, no absolute or hidden names, at most 512 bytes); `mimetype` must come first and
match. `.wsmz`, `.png` and `.jpg` entries are stored as they are, every other is compressed. A
pack opens as a new project folder, never over a folder that holds something, and only once the
whole file has been read and Book has read the project from it; its files are written as the
pack holds them.

**Recovery.** While there are unsaved changes, the project's own files are packed to
`.recovery/unsaved.wsbkz` inside the project folder before each save; a save that finishes
removes it, and the `.recovery` folder when it is then empty. After a crash the project opens
whole (each file is written atomically) and the recovered work is offered.

**Backups** are packed projects named `<name> <YYYY-MM-DD HH.MM.SS> UTC.wsbkz` (a space and a
number from 2 added for a second backup in the same second; a file is never replaced), in a
folder the app chooses, the oldest removed beyond the number kept, in the order they were made.
Only a file with exactly such a name is one of the project's backups.
