# Requests to Wordsmith Manuscript

What Book needs from Wordsmith Manuscript. This file is Book's index: the record of each open
request is an issue in the
[Manuscript repository](https://github.com/jamescjfoundation/wordsmith-manuscript/issues), where
the people and agents working on Manuscript will see it (core decision 0015; Route B in the
[project map](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-b)).
A Book change never edits the Manuscript repository. Each row says what, why, which Book step
needed it, and what Book does meanwhile; the **Issue** column links the request.

**The contract's shape** (decided 2026-11-26). The contract is a named list of what Book
really uses, written in Manuscript's
[`docs/book-handoff.md`](https://github.com/jamescjfoundation/wordsmith-manuscript/blob/main/docs/book-handoff.md),
with a test and a CI job on Manuscript's side that builds Book against Manuscript's branch.
Book uses nothing the list lacks, moves to a new `book-view` tag only when Manuscript has cut
it, and keeps its `main` buildable against the newest tag (`AGENTS.md`, "Book and Wordsmith
Manuscript"). Requests 2, 3 and 7 are about the contract's own text; requests 4, 5, 6 and 8 ask
for something new in the book view.

| # | What | Why | Needed by | State | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Move Manuscript to the newest core release | Book is on a newer core than Manuscript's `v0.1.4`, so Cargo builds two copies of the core (slower builds, more disk). Every core release since is additive | – (cost only) | done 2026-11-26: Manuscript moved to the newest core and cuts a `book-view-1.N` tag on each core release it moves to; Book pins that tag | – |
| 2 | Treat the public parts Book uses beyond the `book` and `offers` modules as part of the hand-off contract: `folder::Folder::open` and `outside_changes`, `pack::read_packed`, `pack::unpack`, `pack::pack_folder`, the folder writer (`Folder::create`, `Folder::save`) for owned sources, and `words::words`. 2026-11-26: the issue's list was short of what the code names; a comment on it gives every item (the model crate's types, `pack::folder_files`, `pack::MAX_PACKED`, the reader's `Error` with its `Newer` case, `SCHEMA`), and says the contract's `book::open_folder` and `book::open_packed` are not called | Book reads linked projects, notices outside changes, reads packed pictures, packs a linked source into a bundle and writes owned sources through them (`docs/handoff-review.md`) | M1.2 | open | [#7](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/7) |
| 3 | Strip comment anchors (`Marks::comment`) from the book view, or say in the contract that they stay | The contract says comments are removed; the view keeps their anchors | M1.2 | open | [#8](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/8) |
| 4 | A cover role on a media item (or a cover field in the book's metadata) | Book cannot tell the cover from other pictures (EBK-5, PRT-10). Built around: an edition names its cover file in Book's project (`cover`, `docs/format.md`) | M6.2 | open | [#9](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/9) |
| 5 | Editions with formats in the book's metadata (or a fixed set of edition names for ISBNs) | ISBNs are keyed by free names; Book keeps its own editions (META-2) | M3.3 | open | [#10](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/10) |
| 6 | Say which target formats' raw blocks Book should honour, if any | Book drops `Block::Raw` content and reports it | M5.1 | open | [#11](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/11) |
| 7 | A release tag with book view version 1 | Book pins a non-release tag, `book-view-1.N` (decision 0001; `Cargo.toml` names the current one) | – | open | [#12](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/12) |
| 8 | A decorative mark on pictures (`Block::Image`) | Pictures with no alternative text block ebook export unless decorative (SPEC 6.6). Built around: Book keeps its own list of decorative pictures (`decorative` in `book.toml`) | M6.2 | open | [#13](https://github.com/jamescjfoundation/wordsmith-manuscript/issues/13) |
