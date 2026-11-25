# 0003: Print typesetting on the core's layout

Date: 2026-11-24. Status: accepted (the plan's fallback, taken because the spike failed).

## Context

A book interior needs paragraph breaking, hyphenation, widow and orphan control, mirrored pages,
running heads that follow chapters, drop caps, real small caps, notes and precise font handling
(plan section 5.6). The plan proposed embedding an open-source typesetting engine, pinned at
0.15.1 (its library, PDF writer and renderer), in `book-typeset`, and set a spike (step M0.7) to
prove it first: it must compile with the core's patch block, its PDFs and page images must be
right with the patched PDF libraries, its licenses must pass, and one chapter must typeset.

## What the spike found (2026-11-24)

- **Licenses** of the engine's dependency tree passed both checks (`cargo xtask licenses`,
  `cargo deny check licenses sources`).
- **It does not compile with the core's patch block.** The core replaces the PDF writing and
  reading libraries (`krilla`, `hayro-*`) with craft-libs' copies (core `docs/UPSTREAM.md`).
  Those copies have moved past the published versions the engine was written against:
  - the engine's PDF writer (0.15.1) fails against the patched `krilla` 0.8.2 (four errors: new
    settings fields, new error and validation variants);
  - the published `hayro` 0.7.1 page renderer, which the engine's library uses for PDF pictures
    and its renderer and SVG writer use for drawing, fails against the patched `hayro-interpret`
    (a new glyph drawing mode).
  So not even the engine's library builds in an app on the core. Making it build needs the
  core's patch block or craft-libs' copies to change, or a fork of the engine: a pin move or fork
  change only a person may make (recorded under "Waiting on a person"). The brief for this run
  says to take the plan's fallback when the spike fails.

## Decision

Print is built on the core, as the plan's fallback says:

- **The print document.** `book-typeset` turns the book (sections, blocks, marks) and the
  design's values for an edition into a print document in the core's model (`jcj-doc`): a
  section per chapter opening, mirrored margins with the gutter inside, running heads on facing
  pages (author left, title right, as the design says) and none on openings and blank pages,
  folios in the footer, real small caps, drop caps, hyphenation by the book's language with at
  most two hyphens in a row, widow and orphan control, verse, block quotes and scene breaks.
  Book's text is only ever the text of runs: it is never read as instructions.
- **Pages, PDF and page images** come from the core: `jcj-layout` lays the document out,
  `jcj-output` writes the PDF (every font embedded, tagged, no links, the project's date, so the
  same book gives the same bytes) and draws page images for the preview.
- **Right-hand starts.** The model's "odd page" section start counts page numbers, not leaves,
  so Book adds blank pages itself: each pass predicts where each chapter lands and puts a blank
  page (no head, no folio) before any that would fall on a left-hand page; the layout runs again
  until nothing moves (two passes for a book, at most four).
- **Fonts** are Book's bundled fonts, loaded into the core's layout by role; system fonts are
  never used for print.
- **Line breaking.** The core's layout breaks lines. An optimal-fit pass in `book-typeset` (the
  plan's fallback) is built in step M5.2 on the core's measuring functions where it improves the
  result; until then the core's breaking with hyphenation is used.
- **Notes.** The core's layout places footnotes at the foot of their reference's page (core
  v0.1.7 made note kinds reachable and tests it), so print notes are footnotes from R2.4, as the
  plan wanted; until then (M5) they print as endnotes per chapter.
- **The page map** comes from the layout (which page holds each section's first block).
- `book-typeset` stays the only crate that knows how a book becomes print, and stays a
  core-candidate (C5, open-source-core#19): a book layer over the core's layout.

## The spike's results

- One public-domain chapter (Waverley's first) typesets in a designed layout: a sunk chapter
  opening with number, title and fleuron, a small-caps lead-in, justified text, running heads and
  folios. Its PDF reads back letter for letter through `jcj_output::read`; two builds give the
  same bytes; page images have the trim's pixel size (6 × 9 in at 72 pixels an inch: 432 × 648);
  the page map puts the chapter on page 1 (`crates/book-typeset/tests/spike.rs`).
- **Build time and size.** The fallback adds no dependency: everything it uses is already in the
  core. Rebuilding `book-typeset` after a change takes under a second; the workspace's `target/`
  was 4.8 GB after the spike (most of it the core and its upstream libraries, built for every
  app). The chapter typesets, writes its PDF and draws a page in a few hundredths of a second.
- **Licenses**: unchanged; `cargo xtask licenses` and `cargo deny check licenses sources` pass.

## Consequences

- The plan changes in the same commit: sections 5.1 to 5.6, 7 (PRT-12, PRT-14), 10 (M0.7, M5,
  R2.4, R3.5, R4.6), 12, 13 and 14 describe print on the core.
- What the core's layout cannot do is a core change (additive, as core v0.1.7 was) or recorded
  as blocked: PDF/A and PDF/X output (PRT-12, PRT-14) depend on the core's PDF writer.
- Book's print quality follows the core's layout; golden files (M5.4) catch any change when the
  core moves.

## Revisit when

A person moves the core's patch block or craft-libs' copies so that a published release of the
engine builds against them (or decides to carry a fork); or the core gains a typesetting layer of
its own (`jcj-typeset`, core decision 0002).

## Addendum (R4.6, 2026-11-25)

Print stays on the core's layout and moved with each core release up to v0.1.15, the newest;
no core move changed a golden file (`docs/golden.md`). Moving to the embedded engine still needs
a person (`docs/progress.md`, "Waiting on a person").

