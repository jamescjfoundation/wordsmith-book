# Requests to Wordsmith Manuscript

What Book needs from Manuscript, for a person to carry over. Book never edits the Manuscript
repository. Each request says what, why, and which Book step needs it.

| # | What | Why | Needed by | State |
| --- | --- | --- | --- | --- |
| 1 | Move Manuscript to the newest core release | Book is on a newer core than Manuscript's `v0.1.4`, so Cargo builds two copies of the core (slower builds, more disk). Every core release since is additive | – (cost only) | open |
| 2 | Treat the public parts Book uses beyond the `book` and `offers` modules as part of the hand-off contract: `folder::Folder::open` and `outside_changes`, `pack::read_packed`, `pack::unpack`, `pack::pack_folder`, the folder writer (`Folder::create`, `Folder::save`) for owned sources, and `words::words` | Book reads linked projects, notices outside changes, reads packed pictures, packs a linked source into a bundle and writes owned sources through them (`docs/handoff-review.md`) | M1.2 | open |
| 3 | Strip comment anchors (`Marks::comment`) from the book view, or say in the contract that they stay | The contract says comments are removed; the view keeps their anchors | M1.2 | open |
| 4 | A cover role on a media item (or a cover field in the book's metadata) | Book cannot tell the cover from other pictures (EBK-5, PRT-10). Built around: an edition names its cover file in Book's project (`cover`, `docs/format.md`) | M6.2 | open |
| 5 | Editions with formats in the book's metadata (or a fixed set of edition names for ISBNs) | ISBNs are keyed by free names; Book keeps its own editions (META-2) | M3.3 | open |
| 6 | Say which target formats' raw blocks Book should honour, if any | Book drops `Block::Raw` content and reports it | M5.1 | open |
| 7 | A release tag with book view version 1 | Book pins the non-release tag `book-view-1` (decision 0001) | – | open |
| 8 | A decorative mark on pictures (`Block::Image`) | Pictures with no alternative text block ebook export unless decorative (SPEC 6.6). Built around: Book keeps its own list of decorative pictures (`decorative` in `book.toml`) | M6.2 | open |
