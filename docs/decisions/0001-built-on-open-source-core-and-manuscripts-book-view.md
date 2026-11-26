# 0001: Built on open-source-core and Manuscript's book view

Date: 2026-11-24. Status: accepted.

## Context

Wordsmith Book is the foundation's third desktop app. open-source-core already holds the window,
ribbon, panels, File page, preferences, saving, backups, recovery, spelling, undo, format import,
PDF reading, theming and agent control. Wordsmith Manuscript, where books are written, publishes a
read-only view of a finished project for Book (`docs/book-handoff.md` in that repository, book
view version 1). Book needs both and must not duplicate either.

## Decision

- **The core gives the general parts.** The repository starts from the core's `template/` and
  uses only its `jcj-` crates, all pinned at one release tag. It started at `v0.1.6`: `v0.1.5` added Manuscript's repository to the core's allowed git
  sources (plan change C10, core issue #15) and `v0.1.6` made `cargo xtask sync` find the app's
  own core when Manuscript's older core is in the same graph (core issue #16). Later pins are recorded in `docs/progress.md` and the core's `CHANGELOG.md`.
- **Manuscript gives the book.** `wordsmith-manuscript-format` and `wordsmith-manuscript-model`
  are git dependencies of `https://github.com/jamescjfoundation/wordsmith-manuscript` at the tag
  `book-view-1`: commit `952d2267653d07b970864bca892ff3aa673ec545`, book view version 1. That
  tag is not a release tag; Manuscript's release tags await a person. Book moves to a release tag
  only when a step needs something a later commit has. Book reads version 1 of the view only and
  refuses others with a clear message. Book never writes a Manuscript project; fixes go back as
  offers in the file beside the project folder (`offers` module, version 1). Since 2026-11-26
  the tag is `book-view-1.1`: the same book view version 1, cut after Manuscript moved to core
  `v0.1.16`.
- **Two copies of the core.** Manuscript at `book-view-1` depends on core `v0.1.4`; Book is
  ahead. Cargo builds both. No `jcj-` type crosses from Manuscript's crates into Book's (the book
  view gives Manuscript's own types), so this builds; it costs compile time and disk. The request
  to move Manuscript is in `docs/requests-to-manuscript.md`. This ended on 2026-11-26: Book and
  Manuscript's `book-view-1.1` both pin core `v0.1.16`, and `cargo xtask core-pin` (in the gate)
  fails if the lock file holds two cores again. The two move to a new core together, Manuscript
  first (core decision 0015).
- **Eight crates and two apps**, in layers (plan section 5.2, AGENTS.md):
  `book-model`, `book-format`, `book-design`, `book-import`, `book-typeset`, `book-epub`,
  `book-check`, `book-ui`, and `apps/book` (the desktop app) and `apps/book-cli` (`book-cli`).
  `cargo xtask app-layers` checks the table. Package names are `wordsmith-book-*`; the app id is
  `org.jcjfoundation.book`; extension keys use the `book.` namespace (core decision 0006).
- **Owned sources are Manuscript folders.** A book imported from a word-processor file becomes a
  Manuscript-format folder inside the Book project, written with `manuscript-format`'s public
  writer and read back through the book view, so Book has one input model (plan section 6.3).

## Consequences

- Core fixes reach Book by moving the pin; `cargo xtask sync --check` keeps the shared settings
  identical to the pinned release.
- Book cannot change a core interface; a missing one is a core change (plan section 5.5) released
  as an additive patch, or a `core-candidate` built here first.
- Changes to the book view reach Book only when Book moves its Manuscript reference.

## Revisit when

Manuscript publishes a release tag with a newer book view, or moves to a newer core release.
