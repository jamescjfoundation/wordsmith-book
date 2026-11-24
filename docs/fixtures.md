# Fixtures

Every file under `tests/fixtures/` is public-domain or written for the test, made by a script in
`tools/` (Python standard library only, deterministic) or by `cargo xtask fixtures`, and listed
here with where it came from. Private manuscripts are never fixtures: run them locally from
`.private/` (plan section 4, rule 1). A test (`crates/book-format/tests/fixtures_listed.rs`) fails
when a file listed here is missing or a file under `tests/fixtures/` is not listed.

## Public-domain novels (`tests/fixtures/novels/`)

Plain UTF-8 text with LF line ends, each the body of a public-domain e-text from a public-domain
e-text library (its addresses are kept in `.private/sources.md` for a person to recheck): the
lines between the e-text's "START" and "END" markers, with the distributor's header and licence
footer removed as its licence allows for public-domain texts; nothing else was changed.
Paragraphs are separated by blank lines; italic is shown as `_underscores_`. Made by
`python3 tools/make-fixtures.py texts <download folder>` (fetched once, 2026-11-24).

| File | Work | Edition | Words | Used for |
| --- | --- | --- | --- | --- |
| `tests/fixtures/novels/waverley.txt` | *Waverley; or, 'Tis Sixty Years Since*, Walter Scott | 1893 edition with an introductory essay and notes, e-text 5998 | about 210,000 | The v0.1 test novel with parts (volumes), notes and verse |
| `tests/fixtures/novels/gullivers-travels.txt` | *Gulliver's Travels into Several Remote Nations of the World*, Jonathan Swift | e-text 829 | about 105,000 | The second v0.1 test novel: parts, front matter, notes |
| `tests/fixtures/novels/middlemarch.txt` | *Middlemarch*, George Eliot | e-text 145 | about 316,000 | Scale (over 300,000 words); parts (books) and verse epigraphs |

## Documents written from the novels (`tests/fixtures/novels/`)

Written for the test by `python3 tools/make-fixtures.py documents`: each novel's text from its
first part to its last chapter (Waverley: from "Volume I." to the start of the notes; Gulliver's
Travels: from the publisher's note to the start of the notes; Middlemarch: from the Prelude to
the Finale), parts as Heading 1, chapters and front and back matter as Heading 2, the
`_underscored_` words in italic, the texts' numbered notes as real notes at their references,
verse (paragraphs of short, indented lines) as one paragraph per stanza with line breaks, and
`[Illustration]` markers left out.

| File | Made from | What it is |
| --- | --- | --- |
| `tests/fixtures/novels/waverley.docx` | `waverley.txt` | DOCX with footnotes |
| `tests/fixtures/novels/waverley.odt` | `waverley.txt` | ODT with notes |
| `tests/fixtures/novels/waverley.rtf` | `waverley.txt` | RTF with footnotes |
| `tests/fixtures/novels/waverley.md` | `waverley.txt` | Markdown with `[^n]` notes |
| `tests/fixtures/novels/gullivers-travels.docx` | `gullivers-travels.txt` | DOCX with footnotes |
| `tests/fixtures/novels/gullivers-travels.odt` | `gullivers-travels.txt` | ODT with notes |
| `tests/fixtures/novels/gullivers-travels.rtf` | `gullivers-travels.txt` | RTF with footnotes |
| `tests/fixtures/novels/gullivers-travels.md` | `gullivers-travels.txt` | Markdown with `[^n]` notes |
| `tests/fixtures/novels/gullivers-travels-online.docx` | `gullivers-travels.txt` | Shaped like a downloaded DOCX from an online word processor: a font and size on every run, no heading styles, part titles and "Chapter N" lines in bold after page breaks, an empty paragraph after each, uneven scene-break marks (`* * *`, `***`, `#`, `*  *  *`, `~`) every twelfth paragraph |
| `tests/fixtures/novels/middlemarch.docx` | `middlemarch.txt` | DOCX, for scale |

**To add by a person:** DOCX, ODT and RTF files saved by real word processors (at least two
different programs), and a DOCX downloaded from an online word processor. No word processor is
installed on the build machine. Open `gullivers-travels.txt` in each program, apply Heading 1 to
the part titles and Heading 2 to the chapter titles, save, and add the files here named by kind
("an open-source word processor, version N"), never by product, with the program's name removed
from the files' metadata.

## Book projects (`tests/fixtures/projects/`)

Written for the test by hand.

| Folder | What it is |
| --- | --- |
| `tests/fixtures/projects/version-0.wsbk/` | A Book project in the never-released draft schema 0 (`docs/format.md`), for the migration path |

## The Manuscript project fixture (`tests/fixtures/novels/`)

Written through Wordsmith Manuscript's public folder writer by `cargo xtask fixtures` (which runs
`crates/book-format/examples/manuscript_fixture`; a Rust generator because the format's writer
is Rust). Deterministic: ids come from a counter, so the same text gives the same files.

| Folder | Made from | What it is |
| --- | --- | --- |
| `tests/fixtures/novels/gullivers-travels.wsm/` | `novels/gullivers-travels.txt` | A Manuscript project: book metadata; a title page, the publisher's note and the letter as front matter; four parts with their 39 chapters, italic and the text's notes as notes at their references; an "About the Author" page as back matter; and, to test what the book view leaves out, a chapter excluded from export, a research note and a comment anchored on the first words |
