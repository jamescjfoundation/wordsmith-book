# The hand-off from Wordsmith Manuscript: Book's review

Status: **awaiting a person's sign-off** ([issue 12](https://github.com/jamescjfoundation/wordsmith-book/issues/12)). Written in step M1.2 (2026-11-24) against
Manuscript's `docs/book-handoff.md` (book view version 1) at the tag `book-view-1` (decision
0001). Manuscript's `docs/SPEC.md` is reviewed on Manuscript's side, not here.

## What Book accepts

- **Book opens the project directly** through `wordsmith-manuscript-format`'s read-only `book`
  module: `book::open_folder`, `book::open_packed`, `book::from_project`. Book reads book view
  version 1 only and refuses any other with a message naming both versions
  (`book-format::source::check_version`, tested).
- **What the view gives**: metadata (`BookInfo`, the byline), front matter, body and back
  matter as `Section`s with stable ids, roles, kinds, titles, numbers, semantic content, notes
  as notes, children; and assets by path. Book keys its own layer (`book.toml`) by the section
  ids. Tested on the Manuscript project fixture: every word, role and id
  (`crates/book-format/tests/sources.rs`).
- **The draft chosen for export** is what the view gives; Book never asks for another draft.
- **Left out**: items excluded from export, screenplays, bible entries and media are not
  sections; a research note is not either (tested); a book in a series opens without its series
  bible.
- **Revision pass marks** are not in the view (Manuscript clears them).
- **Book never writes the project.** Every Book test that uses a linked source checks that the
  project folder's bytes are unchanged.
- **Fixes go back as offers** in the file beside the project (`offers::offers_path`,
  `OffersFile` version 1, appended, never rewritten), and Book reads the writer's decisions with
  `offers::decisions(&project)` (step M3.4).
- **No chosen fonts yet**: Book uses its own designs' fonts.

## What Book relies on beyond the contract

The contract names the `book` and `offers` modules. Book also uses these public parts of
`wordsmith-manuscript-format`, read-only unless said:

- `folder::Folder::open` (read-only) to get the `Project` behind the view, which
  `offers::decisions` needs, and `Folder::outside_changes` to notice that a linked folder
  changed on disk (FIL-2). Opening never writes.
- `pack::read_packed` for the same with a packed project, and `pack::unpack` to read a packed
  project's pictures (the view gives their paths only).
- **The public folder and pack writers for owned sources**: a book imported from DOCX, ODT, RTF,
  Markdown, text or EPUB becomes a Manuscript-format folder inside the Book project
  (`source.wsm/`), written with `folder::Folder::create` and `Folder::save` and read back
  through the same book view (plan section 6.3). That folder is Book's own, never a writer's
  project; it also opens in Manuscript, so a writer can carry on there. `pack::pack_folder`
  packs a copy of a linked source into Book's project bundle (EXP-1).
- `wordsmith-manuscript-model`'s public types (`Block`, `Inline`, `Marks`, `Note`, `Role`,
  `ItemId`, `Project`, `Binder`, `Item`) and `words::words`, the same word rule Manuscript's own
  checks use, so Book's word-for-word checks agree with Manuscript's.

Any of these changing is a change Book must follow; Book asks that they be treated as part of the
contract (request 2).

## Gaps

Each is also a request in `docs/requests-to-manuscript.md`.

1. **Comment anchors stay on the text.** The contract says comments are removed; the view keeps
   each span's `Marks::comment` id (the comment itself is not in the view). Book ignores the
   anchors, so nothing shows, but the contract and the view disagree. (Request 3.)
2. **No cover role on an asset.** The view lists pictures by path and title; Book cannot tell the
   cover from other pictures. Book asks the user for the cover. (Request 4.)
3. **ISBNs by edition name only.** `BookInfo::isbn` maps free names (`print`, `ebook`) to ISBNs;
   there is no list of editions with formats. Book keeps its own editions and offers the source's
   ISBNs where the names match. (Request 5.)
4. **Raw blocks.** `Block::Raw` keeps content for one target format; the contract does not say
   which formats Book should honour. Book drops raw blocks and lists each in the build report.
   (Request 6.)
5. **The tag is not a release.** `book-view-1.N` (first `book-view-1`) is a tag on an unreleased commit. Book moves to a
   release tag when one exists with book view version 1. (Request 7.)
6. **Two copies of the core.** Manuscript at `book-view-1` depends on core v0.1.4; Book is on a
   newer core, so builds carry both. (Request 1; settled 2026-11-26, when Manuscript moved to
   core v0.1.16 and Book to the tag `book-view-1.1`.)
7. **Notes are per section with the writer's labels.** Book numbers notes itself (per chapter or
   for the whole book, as the design says); the writer's labels are not shown. Nothing needed;
   recorded so the contract can say so.

## Since this review (2026-11-26)

Dated notes; the review above is as it was written.

- **The contract's shape is decided**: a named list of what Book really uses, in Manuscript's
  `docs/book-handoff.md`, held by a test and by a CI job on Manuscript's side that builds Book
  (decision 0001, amended). "What Book relies on beyond the contract" above is in that list
  now, with the rest of what Book's code names (the contract's "What Book may use"; request 2,
  Manuscript issue #7, closed).
- **A correction to "What Book accepts".** Book does not call `book::open_folder` or
  `book::open_packed`. It opens the project with `folder::Folder::open` or `pack::read_packed`,
  which it needs for the project behind the view, and takes the view with `book::from_project`.
- **The version check now covers the version that changes.** The book view's version is
  compiled into Book with the view, so that check alone could not fail. Book now also states
  the Manuscript project formats it reads (1 to 3) and refuses a project outside them in its
  own words, naming both versions (`book-format`, `MANUSCRIPT_SCHEMAS`, `source::check_schema`;
  tested with a project inside the range and one outside, as a folder, packed and as an owned
  source).
- Gap 1 is decided: the anchors stay and the contract says so (request 3). Gap 5 is
  answered: the contract's "Tags" says how tags are named and cut, and Manuscript's first
  release tag, `v0.9.0`, is in its table (request 7). Gap 6 is settled, as it says. Gaps 2, 3
  and 4 are open requests in Manuscript's repository (`docs/requests-to-manuscript.md`).

## Sign-off

- [ ] A person from the Book side has read this review and the contract and agrees, or has
      noted what must change.
