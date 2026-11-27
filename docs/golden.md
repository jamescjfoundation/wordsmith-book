# Golden files

Golden files hold what an output looked like when a person last approved it. A test compares
each new build with its golden file and fails on any difference. When a change is meant to move
the output, a person looks at the new output, then the golden files are written again with
`UPDATE_GOLDEN=1` and committed with the change. It is the switch Wordsmith Script and Wordsmith
Manuscript use too, and here it also writes the guide's page of keys again (`docs/guide/keys.md`,
made from the commands). Until 2026-11-27 the switch was `BOOK_BLESS=1`; no golden file changed
with the name.

## Print PDFs (M5.4)

`tests/golden/print/<book>-<design>-<profile>.txt`, checked by
`crates/book-check/tests/print_pdf.rs`. Each file is a summary of one print PDF, read back from
the PDF itself:

- the page count, the page size in points, and the gutter used;
- for every page: its number, its folio, the section it belongs to, and either `blank` or its
  line count with the first and last four words it prints (running heads and folios included,
  contents leaders shown as `…`).

So the summaries catch changes to the text, to every page break, and to the page boxes. The same
test checks PRT-6 on every PDF: one page box at trim size (no crop marks or slug), crop box equal
to the media box, every font embedded, no annotations, links or bookmarks (read with the core's
`jcj_output::read_facts`, core v0.1.10).

| Book | Profile | Trim | Designs |
| --- | --- | --- | --- |
| Gulliver's Travels (the Manuscript fixture) | `pod-paperback` | 5.5 × 8.5 in | all thirteen |
| Waverley (imported from its DOCX) | `pod-paperback` | 6 × 9 in | all thirteen |
| Gulliver's Travels | `generic-pdf` | 5 × 8 in | all thirteen |

**State:** written 2026-11-24 from the builds of M5.4 (the seven designs of R3.1 added
2026-11-25); **awaiting a person's look** at the PDFs
they summarise (build one with `cargo test -p wordsmith-book-check --test pages_for_review --
--ignored`, or from the app once export lands in M8).

## EPUB structure (M6.3)

`tests/golden/epub/<book>-<design>.txt`, checked by `crates/book-check/tests/epub_check.rs`.
Each file is the structure of one EPUB, read back from it: the package metadata (the
identifier's kind only), the manifest, the reading order, every navigation entry and landmark,
the whole stylesheet, and for each document its kind, heading, paragraph count and word count.
Gulliver's Travels (the Manuscript fixture) and Waverley (imported from its DOCX) in all thirteen
designs (the seven added in R3.1 written 2026-11-25).

The same test file checks every one of those EPUBs with Book's own checks (`book-check::epub`),
reads it back through the core's reader word for word against Book's reader, and checks that
the author's words in it are exactly the source's. When `BOOK_EPUB_VALIDATOR` names the standard EPUB
validator's jar (and `BOOK_JAVA` a Java runtime), the validator runs on all twelve and any
error or warning fails the test.
When `BOOK_ACCESS_CHECKER` names an installed EPUB accessibility checker,
`crates/book-check/tests/access_check.rs` runs it on a book's ebook and anything above a note fails
the test. Rewritten in R2.14: the page list's source is now `dc:source` refined as the source of
the pagination (the earlier `a11y:pageBreakSource` was not in the validator's vocabulary).

**State:** written 2026-11-24 from the builds of M6.3; **awaiting a person's look** at the EPUBs
in a reading app (`cargo test -p wordsmith-book-check --test epub -- --ignored` writes them to
`target/review/`).

## The test of v0.1 (M9.4)

`tests/golden/v0.1/<source>-<design>.txt`, checked by `crates/book-check/tests/v0_1.rs`: the
summaries (as for the print PDFs above) of the 6 × 9 in paperback built from Waverley's DOCX,
Gulliver's Travels' DOCX and the Manuscript fixture, in each of the six designs. The same test
checks every word of each PDF and EPUB against the source, preflight for `pod-paperback`, the
EPUBs' structure (and the standard EPUB validator where set up) and that every file builds the
same twice. It takes about two minutes.

**State:** written 2026-11-25 from the builds of M9.4; **awaiting a person's look**.

**R2.4:** every golden file above was written again when printed notes became footnotes and the
ebook's notes pop-up footnotes (the editions' default); they await a person's look again.

## The files a project writes

`tests/golden/project-files.txt`, checked by `crates/book-format/tests/on_disk.rs`
(`the_files_on_disk_are_the_reference_bytes`; the steps are in `tests/on_disk/scenario.rs`):
the SHA-256 of every file each way of writing a project puts on disk. A new project folder on an
owned source (a design, pictures stored as they are, a font compressed, one in a folder, a build
record, and a file of the user's own), the folder after a save, after a save of `book.toml`
edited by hand, the folder packed and the project packed from memory, the recovery copy, the
folder after a save with recovery, two backups in the same second at a fixed time, a packed
project unpacked, a book on a linked source packed and unpacked elsewhere, and a schema 0
project saved.

| Date | What changed | Reviewed |
| --- | --- | --- |
| 2026-11-27 | First version, written by Wordsmith Book 0.9.0's own folder code, before the folder layer became the core's (core issue #45): the move must leave every byte the same | Checked by the author of the change; awaits a maintainer's review |

## The engine and the core's releases (R4.6)

Print is on the core's layout (decision 0003), at the newest core release: v0.1.23 (2026-11-27).
Every golden file was compared at each core move, by the print, EPUB and v0.1 tests in the full
run of that step; `crates/book-check/tests/golden_complete.rs` checks that the folders hold
exactly the files those tests compare (none missing, none stale).

| Core move | Step | Golden files |
| --- | --- | --- |
| v0.1.10 | M5.4 | written then |
| v0.1.11 | R2.3 | unchanged |
| v0.1.12 | R2.5 | unchanged |
| v0.1.13 | R2.16 | unchanged |
| v0.1.14 | R2.17 | unchanged |
| v0.1.15 | R3.5 | unchanged |
| v0.1.16 | core/v0.1.16, with Manuscript's `book-view-1.1` | unchanged |
| v0.1.17 | core/v0.1.17, with Manuscript's `book-view-1.2` | unchanged |
| v0.1.18 | core/v0.1.18, with Manuscript's `book-view-1.3` | unchanged |
| v0.1.20 | core/v0.1.20, with Manuscript's `book-view-1.4` | unchanged |
| v0.1.21 | core/v0.1.21, with Manuscript's `book-view-1.6` | unchanged |
| v0.1.22 | core/v0.1.22 and 0.9.0, with Manuscript's `book-view-1.7` | unchanged |
| v0.1.23 | core/v0.1.23, with Manuscript's `book-view-1.8` | unchanged |

The changes since they were written are Book's own, each reviewed in its step and recorded
there: footnotes (R2.4), asides (R2.6), the page list (R2.10), the page list's source (R2.14),
and the seven designs added in R3.1 (new files only). All still await a person's look (see the
sections above).

