# Performance

Timings Book's tests record, so a change that slows Book down shows. Measured on the development
machine with the test profile (optimised, with debug information), so release builds are as fast
or faster.

## Previews after a design change (M7.3)

`crates/book-ui/tests/live.rs`, on the 150,000-word test book: Waverley, imported from its DOCX
(about 210,000 words), paperback 6 × 9 in.

| What | Time (2026-11-24) | Rule |
| --- | --- | --- |
| First print preview page after opening the preview | 64 ms | – |
| The whole book laid out (540 pages, four passes at most) | 2.5 s | in the background; frames stay under 16 ms (M7.1) |
| First print preview page after a design change (three changes) | 81–91 ms | under 1 s (plan M7.3) |

How: a build lays out the opening first (the front matter and the first chapter) and shows it,
then lays out the whole book and replaces it (`book-check::build::print_opening`). Page images
are drawn on their own thread, only for the spreads in view. The ebook preview lays out only the
first chapter, so it follows a change at the same pace.
