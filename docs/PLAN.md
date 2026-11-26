# Wordsmith Book: build plan

As of 2026-11-24. Status: accepted as the build plan; changed only by a commit that says why.

Wordsmith Book is the J. C. J. Foundation's open-source app for designing and typesetting books. A
writer or a small press brings a finished manuscript, picks a book design, sets the trim size,
fonts, chapter openings and front and back matter, and exports a print-ready PDF for paperback and
hardcover and a valid, accessible EPUB for ebook stores, every word checked against the source.

It is the third app on [open-source-core](https://github.com/jamescjfoundation/open-source-core),
after [Wordsmith Script](https://github.com/jamescjfoundation/wordsmith-script) and
[Wordsmith Manuscript](https://github.com/jamescjfoundation/wordsmith-manuscript). Manuscript is
where a book is written; Book is where it becomes a product. The core already holds the window,
ribbon, panels, File page, preferences, saving, backups, recovery, spelling, undo, format import,
PDF reading, theming and agent control. Manuscript gives Book a read-only view of a finished
project. This repository holds only what is about making a book.

## Contents

1. How to follow this plan
2. Goals and non-goals
3. Who it is for, and the production loop
4. Ground rules
5. Architecture
6. Data format
7. Features by release
8. Import
9. Print, ebook and preflight
10. Milestones and steps
11. Testing and quality gates
12. Risks
13. Open questions
14. Third-party building blocks

## 1. How to follow this plan

This plan is meant to be carried out step by step, by people and coding agents, until v1.0.

- **Order.** Milestones run in the order of section 10. Steps inside a milestone run in order
  unless they say they can run in parallel.
- **Step names.** Each step has an id and a short name (`M1.1 m1/format`). Commits for a step use
  both in the subject (`M1.1 m1/format: the book project folder`). Whether a step is one branch
  and one pull request, or commits straight to `main`, is set by whoever runs the work; the
  step's "Done when" is the same either way.
- **Each step's "Done when"** is its acceptance test. A step is finished when every item there is
  true, its tests pass, `cargo clippy --workspace --all-targets`, `cargo xtask app-layers` and
  `cargo xtask privacy` are clean, and the step's tests are committed with it. While working, run `cargo test` for the affected crates; at the end of every step run
  `cargo test --workspace` once (it gives the count below). The full `cargo xtask ci` runs
  before a release.
- **Progress** is recorded in `docs/progress.md`: one line per finished step (date, step, what
  changed, tests). **Tests** is the sum of the "passed" counts of every `test result:` line printed
  by `cargo test --workspace`. At the end of a milestone, mark its row in the table at the top of
  section 10.
- **Decisions** about this app's behavior go in `docs/decisions/NNNN-title.md` (context, decision,
  consequences, revisit when). Where this plan says "decision", write one in the step that makes
  it. Decisions about shared interfaces go in the core's `docs/decisions/`, never here.
- **Core changes** follow section 5.5 and the core's AGENTS.md. Each has a core issue; app-first
  work is marked `// core-candidate: open-source-core#N` in code and listed in
  `docs/core-candidates.md`.
- **Changing the plan.** If a step turns out wrong or impossible as written (an interface the core
  does not have, a tool not installed, an upstream gap), change this plan in a commit that says
  why, then carry on. Never work around the plan silently, and never remove its rules.
- **Feature ids** (`IMP-1`, `DSN-3`) in section 7 are used in commit subjects, tests and
  `docs/progress.md`, so each feature can be traced from plan to code.

## 2. Goals and non-goals

### Goals

1. **Beautiful by default.** Every built-in design gives a book a reader would take for a
   professionally made one: justified text with good line breaking and hyphenation, no widows or
   orphans at page ends, correct running heads and folios, real small caps, typographic quotes
   and dashes, chapter openings that look designed.
2. **The usual production workflow** writers already expect from book formatting tools: import,
   structure, choose a design, set print and ebook options, preview, check, export. Concepts are
   general; every name, design and text in the app is our own (section 4).
3. **Store-ready output, checked before upload.** Print PDFs meet the common rules of print-on-
   demand services; EPUBs pass the standard EPUB validator with no errors; both are checked by
   Book's own preflight before export.
4. **The author's words are never changed** without the author's say. Typographic cleanup is
   listed, reviewable and reversible; every import and export proves the words with a
   word-for-word check.
5. **Open and portable.** Free, on the three desktop platforms the core supports; plain-text
   project files and design files that can be shared and kept in version control.
6. **Serve a small press.** House designs, imprint details, series consistency, editions with their
   own ISBNs, and scripted, reproducible builds from the command line.
7. **Accessible output.** EPUB accessibility metadata, required alt text, a navigation document
   with landmarks, and tagged PDF, because accessible ebooks are a legal requirement in some
   markets.

### Non-goals for v1.0

- Writing and heavy editing: Manuscript's job. Book allows light fixes only, offered back to the
  writer (section 6.4).
- Picture books, comics, cookbooks and other fixed-layout books.
- A cover designer. Book makes the exact cover template (size, spine, bleed, safe areas, barcode
  area) and checks a supplied cover; the art is made elsewhere.
- Uploading to stores or print services. Book makes files and a checklist.
- Academic publishing (citations, indexes, cross-references).
- Proprietary file formats: an import needs a legitimately published description of the format
  first (as in Script and Manuscript).
- A web build (the core has none).

## 3. Who it is for, and the production loop

Primary users, in order:

1. Self-publishing authors who have a finished manuscript in a word-processor file or a
   Manuscript project, and want a print book and an ebook without learning typesetting.
2. The foundation itself, producing books for its own imprint: house designs, several editions per
   title, builds that can be repeated exactly.
3. Small presses and designers who make and share book designs.

The loop the app is designed around:

1. **New book** from a Manuscript project (linked, read-only) or from a DOCX, ODT, RTF, Markdown
   or text file (imported into a source the Book project owns).
2. **Import review**: detected parts, chapters, scene breaks and front matter beside the source,
   and the list of typographic cleanups proposed. The user accepts, changes or skips each.
3. **Structure**: the outline on the left, the text in the middle, settings on the right; title
   page, copyright, dedication, contents, about the author and "also by" from templates filled
   from the metadata.
4. **Design**: choose a design with a live preview of the book's own first chapter; adjust it; save
   it as the user's own.
5. **Editions**: paperback and ebook (more later), each with its ISBN, trim, paper and overrides.
6. **Preview**: print spreads with trim, margin and bleed guides; ebook pages at device sizes.
7. **Preflight and export**: fix what preflight finds, then export the print PDF, the EPUB and the
   cover template.

## 4. Ground rules

These hold for every step. `AGENTS.md` in this repository carries them for agents (step M0.1).

**Since 2026-11-26** three of them are replaced, and `AGENTS.md` has the current text. Rules 2
and 4, and "named only in" in rule 5, give way to
[project decision 0003](https://github.com/jamescjfoundation/wordsmith/blob/main/project/decisions/0003-naming-other-companies-programs-and-formats.md):
another company, program, service or file format may be named where that is a fact a reader
needs; interface names, designs and font role names stay our own; `cargo xtask names` and its
private list are removed. The route in rule 7 is now an issue in the Manuscript repository
(core decision 0015), indexed in `docs/requests-to-manuscript.md`.

1. **Public, forever.** This repository is public, history included. No one's name, email or
   account, no details of anyone's machine (no home-folder paths), no private writing (even as test
   data), no secrets, no conversation quotes. Private manuscripts may be used only in local runs
   whose inputs and outputs stay in `.private/` (ignored by Git).
2. **No other companies or commercial products, anywhere.** Code, docs, interface text, commit
   messages, fixtures, design names and test names never name another company, store, print
   service, distributor, ISBN agency, commercial font, or commercial book tool, and never say
   the app replaces, follows or is inspired by another product. Use generic words: "an online
   ebook store", "a print-on-demand service", "the national ISBN agency", "an e-ink reader". Print
   and ebook presets are named by trim size, binding and format ("6 × 9 in paperback, cream
   paper"), never by company. Open-source projects we build on are named only where we depend on
   them (`Cargo.toml`, section 14, `ATTRIBUTION.md`, `NOTICE`).
3. **Our own names and design.** Every interface name, menu label, help text, design and icon is
   our own. Book's names: **Designs**, **Chapter openings**, **Scene breaks**, **Editions**,
   **Store editions**, **Preflight**, **House styles**. Behavior comes from `docs/SPEC.md`
   (step M0.3), built from published standards, general typographic practice and our own design.
4. **Sources.** Published standards (W3C, ISO) are cited by URL. Rules taken from commercial
   services' public file guides are recorded generically ("a print-on-demand service's public file
   guide, checked 2026-11-25"); their addresses are kept only in `.private/sources.md` for a
   person to recheck, never committed.
5. **Fonts.** Bundled fonts have open licenses (OFL-1.1 or the core allowlist) and are named only
   in `ATTRIBUTION.md` and their own files under `assets/fonts/`. Everywhere else (designs, the
   interface, docs, commits) they go by our role names ("classic serif", "reading serif",
   "display", "ornaments"). Preflight refuses fonts whose embedding bits forbid embedding.
6. **The author's words are sacred.** Nothing the author wrote is dropped or rewritten by
   importing, building or exporting. Every importer and exporter proves it with a word-for-word
   test. Cleanup happens only as listed changes the user accepted. Light fixes to a linked
   Manuscript project are offers the writer decides on (section 6.4), never edits.
7. **Book never writes a Manuscript project** and never edits the Manuscript repository. What Book
   needs from Manuscript goes in `docs/requests-to-manuscript.md` for a person.
8. **App or core.** Before writing anything, ask the core's four questions (core AGENTS.md, step
   1); section 5.5 has the answers this plan already knows. Never name WordCraft or craft-libs
   types here, never copy code from another app repository, never change a core interface from
   this repository.
9. **House rules for code** (the core's): SPDX line, `#![forbid(unsafe_code)]` and the six
   panicking lints denied at every crate root; product code returns errors and never panics; input
   from files is bounded and never trusted; new dependencies pass the license allowlist (core
   decision 0001); public items documented in plain English.
10. **Licenses.** Only licenses on the core's allowlist (core decision 0001) may be linked;
    anything else needs a person to change the allowlist first. LGPL and GPL code is never copied or
    linked; a GPL program may only be run as a separate, optional program the user installed.
11. **Golden files are the judge.** Print PDFs (text, page breaks, page boxes) and EPUB structure
    are compared with reviewed reference files. A change that alters them is deliberate, reviewed
    and recorded in `docs/golden.md`.
12. **Fixtures** are public-domain texts or written for the test, made by scripts in `tools/` and
    recorded in `docs/fixtures.md`.

## 5. Architecture

### 5.1 Workspace

The repository starts from the core's `template/` at the newest core release (step M0.1):

```text
apps/
  book/                  desktop app (eframe), implements jcj_shell::App
  book-cli/              command line: build, check, verify, convert, report, mcp
crates/
  book-model/            the Book layer: editions, design choices, overrides and fixes keyed by
                         ids, metadata additions, cleanup decisions; no files, no interface
  book-format/           the Book project folder: reader, writer, limits, packing; the source
                         (linked Manuscript project or owned source) and the offers it writes
  book-design/           the design schema, built-in designs, mixing, design files, versions
  book-import/           DOCX, ODT, RTF, Markdown, text, EPUB into an owned source; structure
                         detection; the cleanup list
  book-typeset/          print: the book and a design to a print document laid out and written
                         by the core (section 5.6); page images for the preview
  book-epub/             ebook: XHTML, CSS, package, navigation, accessibility metadata
  book-check/            preflight: print and ebook checks, print profiles, external validators
  book-ui/               views: outline, text, design chooser, settings, previews, preflight,
                         export, ribbon tabs, panels, agent tools
assets/
  brand/                 the brand kit's logos and icons, tokens.json (the brand lives here, never in the core)
  fonts/                 bundled open-licensed fonts (named only in ATTRIBUTION.md)
  designs/               the built-in designs (TOML)
  print/                 print profiles: trims, margins, paper, limits (TOML data)
  i18n/                  interface catalogs (R2)
docs/                    PLAN.md, SPEC.md, format.md, design-format.md, decisions/, guide/,
                         progress.md, golden.md, fixtures.md, core-candidates.md,
                         requests-to-manuscript.md, handoff-review.md
tests/fixtures/          public-domain and written-for-test inputs, with provenance
tools/                   fixture and icon scripts (standard library only)
xtask/                   the core's shared tasks plus this app's (`app-layers`, `assets`)
```

Package names are `wordsmith-book-*` (folder names above are short). App id:
`org.jcjfoundation.book`; extension namespace: `book.` (core decision 0006).

### 5.2 Layers

Each layer calls only the layers under it; `cargo xtask app-layers` checks the table (step M0.1).

| Layer | Crate | Depends on |
| --- | --- | --- |
| 1 | `book-model` | `wordsmith-manuscript-model` (types only), `jcj-doc`, `serde` |
| 2 | `book-format`, `book-design` | model, `wordsmith-manuscript-format`, `jcj-project` |
| 3 | `book-import`, `book-typeset`, `book-epub` | layers 1–2, `jcj-interchange`, `jcj-layout`, `jcj-output`, `jcj-proof`, `jcj-features` |
| 4 | `book-check` | layers 1–3 |
| 5 | `book-ui` | everything below, `jcj-engine`, `jcj-shell`, `jcj-theme`, `jcj-agents` |
| 6 | `apps/*` | the crates above |

`book-typeset`, `book-epub` and `book-check` never depend on interface code, so the command line
builds books with no window.

### 5.3 What the core gives Book

| Need | Core crate | How Book uses it |
| --- | --- | --- |
| Rich text model, extension data | `jcj-doc` | Light editing of one section at a time; the ebook preview's documents |
| Editing, undo, commands, recovery | `jcj-engine` | One session for a section being fixed; Book's commands through `registry_with(commands::merged(..))` |
| Pages, measuring, fonts | `jcj-layout` | Print pagination (decision 0003); the ebook preview at device sizes (section 9.5) |
| PDF writing and reading, page images | `jcj-output` | The print PDF and the preview's page images (decision 0003); `read` to check exported PDFs letter by letter and their pages |
| DOCX, ODT, RTF, Markdown, text, EPUB reading | `jcj-interchange` | Every import; Book maps the result to its structure |
| Spelling, hyphenation | `jcj-proof` | Spelling while fixing text; hyphenation exceptions per language for print |
| Atomic saves, backups, recovery, auto-save timing | `jcj-project` | Book project files |
| Comparison | `jcj-features` | Re-import and offer differences (with the word-level diff of C2) |
| Window, ribbon, panels, File page, start window, preferences, keys, updates | `jcj-shell` | Book implements `App` and supplies tabs, panels, File page sections, templates and About |
| Theme | `jcj-theme` | `assets/brand/tokens.json` |
| Agent control | `jcj-agents` | Every Book command as a tool |

### 5.4 What Manuscript gives Book

Book reads Manuscript projects through `wordsmith-manuscript-format`'s read-only `book` module,
as `docs/book-handoff.md` in the Manuscript repository describes (book view version 1). The facts
Book relies on:

- **Version.** The book view is version 1 (`book::VERSION`). Book declares the versions it reads
  (`[1]`) and refuses others with a clear message.
- **What it reads.** `book::open_folder`, `book::open_packed`, `book::from_project` give a
  `book::Book`: metadata (`BookInfo` and the byline), front matter, body and back matter as
  `Section`s with stable ids, roles, kinds, titles, numbers, semantic content
  (`Vec<Block>`), notes as notes, children; and assets. Helpers: `outline`, `words`,
  `missing_metadata`, `counts`.
- **The draft chosen for export.** The view always gives the draft the writer chose to export.
- **Working marks are gone.** Revision pass marks and comments are not in the view.
- **Left out.** Screenplays, bible entries, items excluded from export and media items are not
  sections. A book in a series opens without its series bible (the view does not use it).
- **Fixes go back as offers.** Book writes offers to the file beside the project folder
  (`offers::offers_path`, `offers::OffersFile`, version 1, appended, never rewritten once
  written) and reads the writer's decisions with `offers::decisions(&project)`. An offer with no
  decision is waiting.
- **Fonts.** The view gives no chosen fonts yet; Book uses its own designs' fonts.
- **Book never writes the Manuscript project.**

**The dependency.** `wordsmith-manuscript-format` and `wordsmith-manuscript-model` are git
dependencies of `https://github.com/jamescjfoundation/wordsmith-manuscript` at the tag
`book-view-1` (commit `952d2267653d07b970864bca892ff3aa673ec545`, book view version 1); decision
0001 records it. The core's `deny.toml` accepts git sources only from its list;
Manuscript's address is added to that list by core change C10. Book also uses
`manuscript-format`'s public folder and pack writers for the **owned source** of an imported book
(section 6.3): that is a Book-owned folder, never a writer's project, and the hand-off review
(step M1.2) records it.

**Two copies of the core.** Manuscript at its pinned reference depends on core v0.1.4. When Book
moves to a newer core release, Cargo builds both core versions. No `jcj-` type crosses from
Manuscript's crates into Book's (the book view gives Manuscript's own types), so this builds;
it costs compile time and disk. Book records the request "move to core vX" in
`docs/requests-to-manuscript.md` and moves its Manuscript reference when a newer one exists.

**Requests and review.** `docs/requests-to-manuscript.md` lists what Book needs from Manuscript
(for example chosen fonts, a cover role on an asset, an edition list), each with why; a person
carries them over. `docs/handoff-review.md` is Book's side of the review of the hand-off contract
(step M1.2): what Book accepts, what it relies on beyond the contract, and the gaps.

### 5.5 Core changes Book needs

Each change has a core issue describing the general need. Core work follows the brief for the
run: additive changes with tests and a use in `core-testbed`, released as a patch (0.1.x),
never a minor release, never an upstream pin move. "Promote" means the general piece moves into
the core because Book is the second app that needs it; Manuscript's code may be the starting
point (app to core, as Manuscript's entries layer moved into the core as `jcj-entries` in core
0.1.4), and Manuscript's own copies stay until a person moves Manuscript over. Code is never
copied from Manuscript into Book. C10 changes only the core's settings, so its use in
`core-testbed` is the core's own `cargo deny check sources` passing.

The core's rule that apps stay at most one core release behind cannot hold during this build:
Book's promotions release several patches while Script and Manuscript stay where they are. Every
release here is additive ("What apps must do: nothing"), and moving the other apps is recorded for
a person in Book's `docs/progress.md`.

| Id | Change | Kind | Needed by | Core issue |
| --- | --- | --- | --- | --- |
| C1 | Word-for-word verification: the words of a source and a result in reading order, compared, first difference with context | Promote (Manuscript's `manuscript-io::verify`) | M2.1 | [#10](https://github.com/jamescjfoundation/open-source-core/issues/10) |
| C2 | Word-level comparison of two texts (paragraphs matched, then words), beside `jcj-features` compare | Promote (Manuscript's `manuscript-model::diff`) | M2.4 | [#17](https://github.com/jamescjfoundation/open-source-core/issues/17) |
| C3 | A tree widget with drag and drop | Promote (Manuscript's binder tree) | M1.4 | [#6](https://github.com/jamescjfoundation/open-source-core/issues/6) |
| C4 | An interface translation catalog, and the shell's own texts through it (English unchanged by default) | Promote (Manuscript's `manuscript-ui::i18n`) | R2 (ACC-3) | [#18](https://github.com/jamescjfoundation/open-source-core/issues/18) |
| C5 | Book typesetting (optimal-fit line breaking, right-hand starts, page templates, running heads, footnotes, tagged PDF) | Build in app first (`book-typeset`, on the core's layout since decision 0003), `core-candidate`; promoted as `jcj-typeset` when a second app uses it (section 5.6) | M0.7, M5 | [#19](https://github.com/jamescjfoundation/open-source-core/issues/19) |
| C6 | An EPUB 3 package layer (container, package document with accessibility metadata, navigation with landmarks and page list, NCX, deterministic zip) | Build in app first (`book-epub`), `core-candidate` | M6.1 | [#20](https://github.com/jamescjfoundation/open-source-core/issues/20) |
| C7 | The core's EPUB writer drops notes | Core fix, additive: notes written as numbered endnotes per chapter with links both ways, as Manuscript's reading copy does by hand | R2 | [#21](https://github.com/jamescjfoundation/open-source-core/issues/21) |
| C8 | Structure detection for imports: sections by heading level and title patterns ("Chapter 12", "Part Two"), scene-break paragraphs | Build in app first (`book-import`), `core-candidate`; Manuscript has its own | M2.2 | [#22](https://github.com/jamescjfoundation/open-source-core/issues/22) |
| C9 | PDF facts for preflight: page boxes (media, trim, bleed), fonts and whether embedded, annotations, images and their resolution | Core, additive to `jcj_output` (`read_facts`, core v0.1.10) | M5.4 (brought forward from M8.1: M5.4's tests need it) | [#23](https://github.com/jamescjfoundation/open-source-core/issues/23) |
| C10 | `deny.toml` allows `https://github.com/jamescjfoundation/wordsmith-manuscript` as a git source | Core, additive (synced settings) | M0.1 | [#15](https://github.com/jamescjfoundation/open-source-core/issues/15) |
| C11 | A sortable, editable table widget | Promote (Manuscript's outline table) if editions and the ISBN list need it | R4 | [#7](https://github.com/jamescjfoundation/open-source-core/issues/7) |
| C12 | The PDF names the app that made it | Book's PDFs come from `jcj-output` (decision 0003), which does not name the app yet; stays blocked upstream for `jcj-output`, not needed for any step | – | [#5](https://github.com/jamescjfoundation/open-source-core/issues/5) |

Not needed by Book: the stitched view (#9; Book fixes one section at a time), project search (#8),
story board lanes (#12).

`docs/core-candidates.md` lists every `// core-candidate:` in this repository with its issue and
state (app-first, promoted in core vX, waiting).

### 5.6 Print typesetting

Print typesetting is the largest technical choice. A book interior needs optimal-fit paragraph
breaking, hyphenation, widow and orphan control, mirrored pages, running heads that follow
chapters, drop caps, real small caps, footnotes at the page foot, and precise font handling.

The plan first proposed embedding an open-source typesetting engine (pinned at 0.15.1) in
`book-typeset`, with a spike in M0.7 to prove it. The spike failed: the engine does not compile
against the core's patched PDF libraries (decision 0003). Print is therefore built on the core,
as the plan's fallback said:

- **A print document.** `book-typeset` turns the book and the design's values for an edition into
  a document in the core's model (`jcj-doc`): a section per opening, mirrored margins, running
  heads on facing pages, folios, small caps, drop caps, hyphenation, widow and orphan control.
  The author's text is only ever the text of runs, never instructions.
- **The core lays it out and writes it.** `jcj-layout` paginates; `jcj-output` writes the PDF
  (fonts embedded, tagged, the project's date, so output is reproducible) and draws the page
  images for the preview; the page map (which section starts on which page) comes from the
  layout.
- **Book's own passes.** Right-hand chapter starts (blank pages added by leaf, not by page
  number), the gutter from the page count, and an optimal-fit line-breaking pass where the core's
  measuring allows it, each laying out again until stable (at most four passes).
- **Fonts** only from `assets/fonts/` and fonts the user added to the project; no system fonts.
- **Notes.** The core's layout places footnotes at the page foot; print notes are endnotes per
  chapter in 0.1 and footnotes from R2.4.
- **Core.** `book-typeset` is a `core-candidate` (C5): a book layer over the core's layout.
  Core decision 0002 expects a typesetting layer to come into the core as `jcj-typeset`; that
  happens when a second app wants it. What the core's layout or PDF writer lacks is an additive
  core change or recorded as blocked.
- **The engine, later.** If a person moves the core's patch block so a published engine release
  builds against it, decision 0003 says how to revisit; nothing in Book depends on that.

### 5.7 The ebook writer and notes

Book writes EPUB itself in `book-epub` from the book and the design, not through the core's
`jcj-interchange` EPUB writer: an ebook needs the design's CSS, semantic `epub:type` on every
front and back matter element, landmarks, a page list mapped to print pages, accessibility
metadata and pop-up notes, none of which a `jcj_doc::Document` carries. Notes are written as the
edition chooses: pop-up footnotes (`epub:type="noteref"` and `"footnote"` in `aside` elements) or
endnotes per chapter or at the book's end, each with links both ways. The general package layer
is a core-candidate (C6). The core writer's own gap (it drops notes, which Manuscript works around
in its reading copy) is fixed in the core as C7, additively, so other apps stop needing the
workaround.

### 5.8 The ebook preview

The core's shell is egui and has no web view. Book's ebook preview (EBK-8) is drawn by Book: the
chapter in the ebook mapping of the design (the same conservative styles the CSS carries), laid
out through `jcj-layout` at device sizes (phone, e-ink reader, tablet), with text size and
dark-page toggles. It is labelled a preview. "Open in a reading app" hands the exported EPUB to
the reading app the user has installed. An embedded web view is after 1.0, when the core has one.

## 6. Data format

Specified in `docs/format.md` (step M1.1); the design format in `docs/design-format.md` (M4.1).

### 6.1 Requirements

1. Plain text and diff-friendly, so a press can keep books in version control.
2. Book's own layer is separate from the words: designs, editions and overrides point at stable
   ids; the words live in the source.
3. Stable ids for every section (the source's ids).
4. A schema version; a newer one is refused clearly. **Until Book's first release tag is pushed,
   format version 1 is extended in place, with no migrations and fixtures regenerated by
   `tools/`**; the migration path itself is built and tested in M1.1 with a test-only "version
   0" fixture, so the first real change after a release has it ready. From the first tag on,
   every format change raises the version with a migration and a fixture of the old version.
5. Bounded, safe parsing: hostile or damaged files fail cleanly.
6. The same project gives the same bytes.

### 6.2 Layout

A Book project is a folder (extensions checked in M0.2, decision 0002: `.wsbk`, packed `.wsbkz`):

```text
My Book.wsbk/
  book.toml               schema, the source, metadata additions, editions, the design in use
                          and per-edition design overrides, print-only and ebook-only flags,
                          cleanup decisions, fixes and their offer ids
  designs/<id>.toml       the user's own designs (built-in ones ship in the app)
  source.wsm/             an owned source (section 6.3), when the book was imported
  assets/                 cover images, pictures added in Book, fonts the user added (each with
                          its license file)
  builds/<edition>.json   the last build's record: page map, word checksum, core and design
                          versions (for "changed since last export")
  .recovery/              unsaved work while the app runs
```

- `book.toml` names its source: `source = { manuscript = "../My Novel.wsm" }` (a path from the
  Book project's folder to a Manuscript folder or `.wsmz`), or `source = { owned = "source.wsm" }`.
- Saving writes changed files only, each atomically through `jcj-project`, `book.toml` last.
- Packing (`.wsbkz`), backups and recovery follow the core's container rules (core decision 0009):
  deterministic zip, `mimetype` first, sorted entries, fixed times, entries read one by one into
  memory with sizes and counts capped. A packed Book project with a linked source carries a
  packed copy of the source (read-only) so it travels whole.

### 6.3 Sources: linked and owned

- **Linked Manuscript project.** Book opens it through the book view on every open and when it
  changes on disk; Book never writes it. Each build records a checksum of the words per section,
  so Book shows what changed in the text since the last export, section by section, word by word
  (C2).
- **Owned source.** A book imported from DOCX, ODT, RTF, Markdown, text or EPUB is converted into
  a Manuscript-format project folder inside the Book project (`source.wsm/`), written with
  `manuscript-format`'s public writer and read back through the same book view, so Book has one
  input model whatever the source. Book owns this folder and may edit it; it also opens in
  Manuscript, so a writer can carry on there.

### 6.4 Fixes

A light fix (a typo, a missing comma) is a `find`/`replace` within one paragraph of a section,
kept in `book.toml` with a note:

- **Owned source**: applied to the source as an undoable edit, with the word-level difference
  recorded in the import report.
- **Linked source**: written as an offer to the offers file beside the Manuscript project
  (Manuscript's `offers` module, version 1, appended, never rewritten) and applied in Book's own
  builds while it waits. Book reads the writer's decisions with `offers::decisions`: accepted and
  now in the text → the fix retires; rejected → the fix is removed from builds and the user told;
  waiting → preflight lists it ("1 fix not yet accepted by the writer"); words no longer found →
  stale, shown with the new text. Never a silent overwrite.

### 6.5 Designs

A design is a small TOML file with a schema version (`docs/design-format.md`): body and heading
fonts by role, sizes, leading, indents, justification and hyphenation strictness, widow and
orphan rules; chapter opening (sink, number style and placement, title and subtitle style, rule or
ornament, image area, first-line treatment: drop cap, small-caps lead-in or none); scene-break
style (ornament glyph, image, or space with a small-caps restart); part page; running heads and
folios and the pages that suppress them; front and back matter styles; and the ebook mapping
(the same choices as conservative CSS). A design is data, never code: the print document and the
CSS writer read its values. Built-in designs are ours, each with a golden PDF and EPUB.

## 7. Features by release

Releases: **0.1** = first usable release (milestones M0–M9); **0.2** (R2) = parity for most
novels; **0.3** (R3) = designs, series and standards; **1.0** (R4) = publishing house.
"Core" marks features that need a core change from section 5.5.

### Import (section 8)

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| IMP-1 | Manuscript project | Link a Manuscript project (folder or `.wsmz`) through the book view; never written | 0.1 |
| IMP-2 | DOCX, ODT, RTF | Through `jcj-interchange` into an owned source, including DOCX downloaded from online word processors with their inline formatting | 0.1 |
| IMP-3 | Markdown and plain text | Markdown through the core; text split by chapter patterns | 0.1 |
| IMP-4 | Structure detection | Parts, chapters, scene breaks and front and back matter from heading styles, page breaks, "Chapter N" and numbered headings, symbol lines; confirmed in the import review (C8) | 0.1 |
| IMP-5 | Formatting kept and dropped | Italic, bold, small caps, superscript, links, notes, images, block quotes, lists and meaningful alignment kept (underline offered as italic); fonts, sizes, colors, manual spacing, tab indents, page breaks inside chapters and empty paragraphs dropped and listed | 0.1 |
| IMP-6 | Cleanup review | Straight to curly quotes, double hyphens to dashes, three periods to an ellipsis, double and trailing spaces, uneven scene-break marks: each listed, accepted or undone one by one or all at once, never touching letters; "Keep exactly as written" skips all of it | 0.1 |
| IMP-7 | Word-for-word check | Every import verified against its source (C1) | 0.1 |
| IMP-8 | Re-import | A revised file replaces an owned source's text, keeping design, metadata, editions and front and back matter, with a word-level difference (C2) | 0.2 |
| IMP-9 | EPUB | Re-design an existing ebook: chapters in spine order through the core's EPUB reader | 0.3 |

### Structure and text

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| STR-1 | Outline | Front matter, parts, chapters, back matter in a side panel; drag to reorder Book's own elements (C3); a linked source's order is the writer's | 0.1 |
| STR-2 | Chapters | Number on or off; number style ("Chapter One", "1", "I", "One"); title, subtitle; epigraph | 0.1 |
| STR-3 | Parts | Part pages with their own title | 0.1 |
| STR-4 | Front matter | Half title, title page, copyright page (from metadata, with imprint and ISBN), dedication, epigraph, contents | 0.1 |
| STR-5 | Back matter | About the author, also by, acknowledgements | 0.1 |
| STR-6 | Element templates | Copyright and "also by" filled from metadata and editable | 0.1 |
| STR-7 | Light fixes | Fix words in place; owned sources edited, linked sources offered back (section 6.4) | 0.1 |
| STR-8 | More front and back matter | Foreword, preface, introduction, prologue, epilogue, afterword, newsletter or website page, excerpt from the next book, glossary, endnotes, discussion questions | 0.2 |
| STR-9 | Print-only and ebook-only | Per block or element | 0.2 |
| STR-10 | Store editions | Ebook variants the user names (by store), so links and "buy" pages show only in the matching edition | 0.2 |
| STR-11 | Box sets | Several books in one, each keeping its front matter, one main contents | 0.3 |
| TXT-1 | Text elements | Body, first paragraph (no indent; drop cap or small-caps lead-in), block quote, verse with hanging indents, subheadings, lists, links (live in the ebook; shown or removed in print), inline and full-width images with caption and required alt text | 0.1 |
| TXT-2 | Asides | Letters, notes, signs, messages styled by the design | 0.2 |
| TXT-3 | Notes | Print footnotes at the page foot; ebook pop-up notes or endnotes per edition | 0.2 |
| TXT-4 | Full-page and bleed images | With the edition's bleed | 0.2 |
| TXT-5 | No-break text and hyphenation hints | Per word | 0.2 |

### Designs

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| DSN-1 | Design schema | Versioned TOML (section 6.5), print and ebook from one design | 0.1 |
| DSN-2 | Built-in designs | Six at 0.1 (literary, romance, thriller, fantasy, non-fiction, classic) | 0.1 |
| DSN-3 | Mix and match | Chapter opening from one design, body from another | 0.1 |
| DSN-4 | Your own designs | Save a changed design; export and import it as one file | 0.1 |
| DSN-5 | Design chooser | Genre and mood filters, live preview on the book's own first chapter | 0.1 |
| DSN-6 | More designs | Twelve to sixteen, including a large-print design | 0.3 |
| DSN-7 | House styles | Lock a design for an imprint or series | 1.0 |

### Print

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| PRT-1 | Trims | 5 × 8, 5.25 × 8, 5.5 × 8.5, 6 × 9 in at 0.1; the full common list and metric sizes at 0.2; as data in `assets/print/` | 0.1 / 0.2 |
| PRT-2 | Margins | Gutter from the page count by the common print-on-demand table, recomputed after every layout; outside margins with their minimums; mirrored; per-side overrides | 0.1 |
| PRT-3 | Page rules | Chapters on right-hand pages or any page; roman folios in front matter, arabic from the first chapter; blank-page runs within limits | 0.1 |
| PRT-4 | Type | Size and leading presets per trim, 7 pt minimum | 0.1 |
| PRT-5 | Paper | White, cream, color: spine width per page | 0.1 |
| PRT-6 | Print PDF | Single pages, trim size (or trim plus bleed), every font embedded, no crop marks, annotations, bookmarks or comments | 0.1 |
| PRT-7 | Bleed | 0.125 in outside, top and bottom, for full-bleed images | 0.2 |
| PRT-8 | Hardcover | Case-laminate sizes and wider margins | 0.2 |
| PRT-9 | Cover template | Full-wrap PDF and PNG guide: width (bleed + back + spine + front + bleed), height, bleed, safe areas, spine text only above the page count the profile sets, barcode area | 0.2 |
| PRT-10 | Cover check | A supplied cover PDF or image checked against the computed size and resolution | 0.2 |
| PRT-11 | Large-print variant | One click from any design (16 pt and up) | 0.3 |
| PRT-12 | Tagged and archival PDF | PDF/UA-1 tagging; PDF/A-2b | 0.3 |
| PRT-13 | Proof PDF | Watermark and line numbers for proofreading | 0.3 |
| PRT-14 | PDF/X | When the core's PDF writer supports it; blocked upstream until then (section 12) | after 1.0 |

### Ebook

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| EBK-1 | EPUB 3.3 | XHTML, CSS from the design, package, NCX for older readers | 0.1 |
| EBK-2 | Navigation | Navigation document, landmarks, `epub:type` on every front and back matter element | 0.1 |
| EBK-3 | Accessibility metadata | Access modes, features, summary, conformance to EPUB Accessibility 1.1 and WCAG 2 AA; alt text required | 0.1 |
| EBK-4 | Embedded fonts | On or off per edition | 0.1 |
| EBK-5 | Cover | Cover image with size checks per profile | 0.1 |
| EBK-6 | Page list | Ebook page markers mapped to print pages | 0.2 |
| EBK-7 | Pop-up notes | Where reading systems support them (TXT-3) | 0.2 |
| EBK-8 | Ebook preview | Section 5.8 | 0.1 |

### Preview, preflight, export

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| PRV-1 | Print preview | Spreads with trim, margin and bleed guides, updated as settings change; jump from a page to its section | 0.1 |
| PRE-1 | Preflight | Before export, each issue linked to its page or section: margins against page count, fonts embedded and licensed for embedding, minimum type size, image resolution (300 dpi), blank-page runs, page count within the profile, missing alt text, missing metadata, EPUB structure, fixes waiting on the writer | 0.1 |
| PRE-2 | External validators | The standard EPUB validator and an accessibility checker, run as separate programs the user installed, results in preflight | 0.1 (EPUB validator) / 0.2 (accessibility) |
| PRE-3 | Typographic warnings | Widows, orphans, stacked hyphens, loose lines, rivers flagged by page | 0.2 |
| PRE-4 | Print profiles | Print-on-demand paperback and a generic PDF at 0.1; hardcover and offset at 0.2; as data | 0.1 / 0.2 |
| EXP-1 | Export | Print PDF, EPUB, and a project bundle (book, designs, fonts, pictures) for hand-off and archive | 0.1 |
| EXP-2 | Reproducible builds | The same project and app version give byte-identical files | 0.1 |
| EXP-3 | DOCX | For an editor or a store that takes only word-processor files | after 1.0 |

### Metadata and publishing

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| META-1 | Book metadata | Title, subtitle, series and number, contributors with roles, language, description, keywords, subject codes, copyright year and holder (from the source, with Book's additions) | 0.1 |
| META-2 | Editions | Ebook and paperback at 0.1, each with ISBN, trim, design overrides and price notes; hardcover and large print at 0.2 | 0.1 / 0.2 |
| META-3 | Imprint profiles | Name, logo, address, copyright wording, default house design | 1.0 |
| META-4 | Series view | Books of a series with one design and "also by" built from the series | 1.0 |
| META-5 | ISBN list | ISBNs the imprint owns and which edition uses each (C11 if the table widget is needed) | 1.0 |
| META-6 | Distributor metadata | ONIX for Books 3.0 export | 1.0 |

### Access, automation, files

| Id | Feature | What it does | Release |
| --- | --- | --- | --- |
| ACC-1 | Themes and keys | Light and dark through the shell; every command by keyboard | 0.1 |
| ACC-2 | Screen readers | Views named through AccessKit | 0.2 |
| ACC-3 | Interface languages | Through the promoted catalog (C4) | 0.2 |
| AUT-1 | Command line | `build` (an edition or all), `check` (preflight), `verify`, `convert`, `report`, `run`, `mcp` | 0.1 |
| AUT-2 | Agent tools | Every Book command as an agent tool | 0.1 |
| FIL-1 | Auto-save, backups, recovery | Core timing, packed backups, recovery | 0.1 |
| FIL-2 | Linked source changes | A linked Manuscript project changed on disk is read again, with the changes shown | 0.1 |

After 1.0: fixed-layout EPUB; translated editions sharing a design; proofing comments on preview
pages; a shared design gallery with a review step; audiobook-ready chapter scripts; plays from
Wordsmith Script as books (needs Book's own reader of Script's published format); optional helpers
(alt text and description drafts) behind a setting that is off by default and never touching the
text; an embedded web view for the ebook preview.

## 8. Import

Every import produces a report: what was created, formatting kept and dropped, the cleanup list,
and the word-for-word check (C1). Imports go through `jcj-interchange` to a `jcj_doc::Document`;
`book-import` maps it to structure and writes the owned source. Messy sources (fake headings made
of bold text, manual page breaks, inline fonts instead of styles, uneven scene breaks) are handled
by the import review: detected structure beside the source, rules the user can change (kept in
`book.toml`), then import. An import whose words do not match is refused with the first difference
shown.

## 9. Print, ebook and preflight

### 9.1 Print profiles

`assets/print/<id>.toml`, one per profile, data only, updated without code: trims, bleed, the
gutter table by page count, outside margin minimums with and without bleed, paper thickness per
page by paper, page count limits per trim and paper, blank-page run limits, minimum type size,
spine-text threshold, cover barcode area. Each file records when it was last checked and against
what kind of source (rule 4 of section 4). Profiles at 0.1: `pod-paperback` and `generic-pdf`.

### 9.2 The print pipeline

Book → the assembled book, the design's values and the profile → a print document in the core's
model → the core's layout → pages → PDF and page images and the page map (decision 0003). The gutter depends on the page count, so a build lays out, reads the count, sets the
gutter and lays out again until the count is stable (at most four passes, tested; changed from three because right-hand starts, the gutter and the
contents' page numbers each settle in turn on the core's layout, decision 0003).

### 9.3 The ebook pipeline

Book → XHTML per section (semantic, `epub:type`), one CSS from the design's ebook mapping, the
package document with metadata and accessibility metadata, navigation (contents, landmarks, page
list), NCX, cover, fonts if embedded → a deterministic zip with `mimetype` first.

### 9.4 Preflight

`book-check` runs on the built files, not on intentions: it reads the PDF back (`jcj_output::read`
and C9) and the EPUB back, checks every rule of the profile, checks the words of both against the
source letter by letter (C1), and runs the external validators when the user has them. Each
finding has a severity (blocks export, warning, note), a message in plain words and a link to the
page or section.

### 9.5 Ebook preview

Section 5.8.

## 10. Milestones and steps

| Milestone | Release | Result | State |
| --- | --- | --- | --- |
| M0 Foundations | – | Repository, rules, spec, brand, fixtures, core issues, engine spike | done (the spike took the fallback, decision 0003) |
| M1 Project | 0.1 | Format, sources, saving, outline | done |
| M2 Import | 0.1 | Verifier, imports, structure detection, cleanup review | done |
| M3 Structure | 0.1 | Front and back matter, chapters and parts, metadata, editions, fixes | done |
| M4 Designs | 0.1 | Design format, six designs, mixing, own designs, chooser | done |
| M5 Print | 0.1 | Typesetting, trims and margins, chapter openings, print PDF | done |
| M6 Ebook | 0.1 | EPUB writer, navigation, accessibility, cover, fonts | done |
| M7 Preview | 0.1 | Print spreads, ebook preview, live updates | done |
| M8 Preflight and export | 0.1 | Checks, validators, export, bundle | done |
| M9 Release | 0.1 | CLI, agent tools, guide, packaging, the v0.1 test | done (the release itself waits for a person) |
| R2 | 0.2 | Parity for most novels | done (the release itself waits for a person) |
| R3 | 0.3 | Designs, series and standards | done (the release itself waits for a person) |
| R4 | 1.0 | Publishing house | done (the release itself waits for a person) |

**The test of v0.1**: two public-domain novels (one of at least 150,000 words, one with parts,
notes and verse) go from a DOCX file and from a Manuscript project fixture to a 6 × 9 in
paperback PDF and an EPUB in each of the six designs; every word of both outputs matches the
source; the PDFs pass Book's preflight for `pod-paperback` and match their golden files; the EPUBs
pass Book's structural checks (and the standard EPUB validator where installed); building twice
gives the same bytes. This runs as an end-to-end test (M9.4).

### M0 Foundations

**M0.1 `m0/template`**: start from the core's `template/` at the newest core release.
- Rename the packages and apps (section 5.1), set `APP`, `NAME`, `SHIPPED`, `PRODUCT`; run
  `cargo xtask sync`, `install-hooks`.
- LICENSE-MIT, LICENSE-APACHE, NOTICE (the core's required credit), SECURITY.md, CONTRIBUTING.md,
  CODE_OF_CONDUCT.md, ATTRIBUTION.md, `.gitignore` with `.private/`.
- AGENTS.md: the template's text plus "Wordsmith Book's own rules" (section 4, rules 2–7, 11, 12,
  and where each crate's code goes). CLAUDE.md: `@AGENTS.md`.
- Empty crates of section 5.1 with crate-root rules; the layers table, the `assets` check and the
  `names` check in `xtask`. `names` reads `.private/names.txt` when it exists (one name per line)
  and fails when a tracked file or a commit message on `main` holds one as a whole word, case
  ignored; `ATTRIBUTION.md`, `assets/fonts/` and the public-domain texts under
  `tests/fixtures/novels/` are exempt (section 12). (The `names` check was removed on 2026-11-26: see the
  note at the head of section 4.)
- `wordsmith-manuscript-format` and `-model` as dependencies at the reference of decision 0001.
- C10: its core issue opened, the change made in the core (Manuscript's address in `deny.toml`'s
  `allow-git`), released as a patch; Book pinned to it.
- `docs/progress.md`, `docs/decisions/README.md`, `docs/core-candidates.md`,
  `docs/requests-to-manuscript.md`; decision 0001 "Built on open-source-core and Manuscript's book
  view" (what comes from the core, what from Manuscript and at which reference, the crate layout).
- Done when: the workspace builds, `cargo xtask sync --check`, `app-layers`, `privacy` and
  `licenses` pass, and the empty app opens a window through `jcj-shell` with Book's name.

**M0.2 `m0/name`** (can run in parallel with M0.3–M0.5): check that `.wsb` and `.wsbz` are not
registered to a common file type on any of the three desktop platforms. Done when: decision 0002
records the name and extensions; if one changes, this plan is updated.

**M0.3 `m0/spec`**: `docs/SPEC.md`, the behavior spec, from published standards (EPUB 3.3, EPUB
Accessibility 1.1, PDF/UA, PDF/A), general typographic practice and public file guides recorded as
section 4 rule 4 says, with our own design where they are silent. Each statement marked Verified
(two or more public sources), Sourced (one) or Designed (ours). Sections: import and cleanup,
structure and front and back matter, text elements, designs and chapter openings, print rules and
profiles, ebook rules, preview, preflight, export, metadata and editions, linked sources and
offers. Done when: every 0.1 feature id has a spec section, no company or product is named, and
it is marked as awaiting a person's review.

**M0.4 `m0/brand`**: `assets/brand/tokens.json` (light, dark, extra-dark), the mark and app
icons (from the Wordsmith brand kit since decision 0004; a placeholder drawn by a script in `tools/` before); interface fonts from the core's bundled
OFL faces. Done when: the window uses the tokens and icons and `cargo xtask assets` passes.

**M0.5 `m0/fixtures`**: `tests/fixtures/` and `docs/fixtures.md`, made by `tools/make-fixtures.py`
(standard library only, deterministic): public-domain novels as text (fetched once; each recorded
by title, author, edition and translator, its source described as "a public-domain e-text
library" with the address kept in `.private/sources.md`; one over 300,000 words for scale; one
with parts, notes and verse); DOCX, ODT, RTF and Markdown written from them; a DOCX written to look
like an online word processor's download (inline fonts and sizes, no heading styles, bold "Chapter
N" lines, uneven scene breaks). The Manuscript project fixture comes in M1.2. No word processor is
installed, so files saved by real word processors are listed as "to add by a person".
- Tests: a test reads every fixture file listed in `docs/fixtures.md` and fails on one that is
  missing or unlisted.
- Done when: every fixture's provenance is recorded and `cargo xtask privacy` and `names` pass.

**M0.6 `m0/core-issues`** (can run in parallel): open a core issue for each "new" row of section
5.5 not opened yet (C10's is opened in M0.1), each describing the general need without Book's
features, labelled `core-candidate` where the work starts here; link each from section 5.5; start
`docs/core-candidates.md`.
- Done when: every row of 5.5 has an issue link.

**M0.7 `m0/engine`** (done; the spike failed and the fallback was taken, decision 0003): the
typesetting spike of section 5.6 in `book-typeset`: the engine at an exact
version, Book's closed `World`, the JSON-data template with one design's body and chapter opening,
one public-domain chapter to PDF and page images. Records build time, added target size and the
licenses of the new dependency tree. Decision 0003 (the engine, how it is embedded, the fallback).
- Tests: the chapter's PDF read back word for word through `jcj_output::read`; two builds compared
  byte for byte; a page image has the trim's pixel size.
- Done when: those tests pass; `cargo xtask licenses` and `cargo deny check licenses sources`
  pass; decision 0003 is written (or the fallback is chosen and this plan changed).

### M1 Project

**M1.1 `m1/format`**: `book-model` and `book-format`: the folder of section 6.2, `book.toml`
schema 1, limits, changed-files-only saving; the migration path tested with a test-only version 0
fixture; `docs/format.md` with an example of every file.
- Tests: round trips (model → files → model exact, unchanged files byte-identical); property tests
  of random projects; hostile input at every limit; every example in `docs/format.md` parsed.
- Done when: those pass, a project saved twice gives the same bytes, and a version-2 project is
  refused clearly.

**M1.2 `m1/sources`**: linking a Manuscript project (folder or `.wsmz`) through the book view;
refusing book view versions other than 1; the owned source written and read through
`manuscript-format`; the per-section word checksums; outside changes to a linked source detected
and read again (FIL-2). The Manuscript project fixture, written through `manuscript-format`'s public
API by `cargo xtask fixtures` (Book's own task; a Rust generator is needed because the format's
writer is Rust) and recorded in `docs/fixtures.md`. `docs/handoff-review.md`: Book's side of the
review of Manuscript's `docs/book-handoff.md` (what Book accepts, what it relies on beyond the
contract, such as the public folder and pack writers for owned sources, and the gaps, each gap also
in `docs/requests-to-manuscript.md`).
- Tests: the fixture through the book view (every word, role and id); an owned source written and
  read back identical; a linked source changed on disk; the Manuscript folder's bytes unchanged
  after every test.
- Done when: those pass and the review is written and marked as awaiting a person's sign-off.

**M1.3 `m1/save`**: atomic saves through `jcj-project`, auto-save on the core's timing driven by
`jcj-shell`, recovery, packed backups (`.wsbkz`) with a number kept, Save as one file, open
`.wsbkz`.
- Tests: crash injection (stop after each file write); packing twice; damaged and hostile `.wsbkz`.
- Done when: every crash reopens to the last good save or the recovered work, packing is
  deterministic, and a damaged `.wsbkz` is refused cleanly.

**M1.4 `m1/outline`**: the app opens and creates Book projects (start window and templates from
the core); the outline panel on the promoted tree widget (C3, released in the core first): front
matter, parts, chapters, back matter; Book's own elements reorder by drag, a linked source's order
shows read-only with "Change this in Manuscript"; every change a command (undoable,
agent-callable).
- Needs: C3 in a core release.
- Tests: outline operations at the model level and through the egui test harness; a 1,000-section
  outline drawn within 50 ms a frame.
- Done when: those pass and the app opens the fixture's outline.

**M1.5 `m1/document`**: Book's own conversion between the book view's semantic text (blocks,
marks, notes, images) and the core's `jcj_doc::Document` at Book's core version, with `book.`
extension keys. Manuscript has a conversion of its own, but it is built against Manuscript's core
version and is never copied here; Book needs its own for imports (M2.2), fixing text (M3.4) and the
ebook preview (M7.2).
- Tests: property tests that every block and mark goes text → document → text exactly; a document
  with formatting the book view cannot hold reports what it dropped.
- Done when: those pass on every fixture.

### M2 Import

**M2.1 `m2/verify`**: C1 promoted into the core (additive, tests, `core-testbed`, patch release),
Book pinned to it; Book's checks of sources, imports and outputs use it.
- Tests: in the core, the verifier's own tests; in Book, a check of every fixture against itself
  and against a copy with one word changed (found, with context).
- Done when: the core release exists, those pass, and `docs/core-candidates.md` records the
  promotion.

**M2.2 `m2/docx-odt-rtf`**: IMP-2, IMP-4, IMP-5: the import review (detected structure beside the
source; rules editable and kept); structure detection as a `core-candidate` (C8); formatting kept
and dropped as listed; the owned source written; the report with the word-for-word check.
- Tests: each fixture format imported and verified; the "online word processor" fixture's chapters
  and scene breaks found with default rules; a source whose words would change is refused; the
  review through the egui harness.
- Done when: those pass.

**M2.3 `m2/markdown-text`**: IMP-3: Markdown through the core, text by chapter patterns and
separator lines, front matter kept.
- Tests: both fixtures imported and verified; separator choices.
- Done when: those pass.

**M2.4 `m2/cleanup`**: IMP-6 on C2 (the word-level comparison promoted into the core first): each
proposed change listed with its place and shown word by word, accepted or undone one by one or all
at once, "Keep exactly as written".
- Needs: C2 in a core release.
- Tests: on every fixture, only quotes, dashes, ellipses and spaces change; every change undone
  singly gives the source back; "Keep exactly as written" changes nothing.
- Done when: those pass.

- Milestone done when: every fixture imports through the review with a passing check.

### M3 Structure

**M3.1 `m3/matter`**: STR-4, STR-5, STR-6: front and back matter elements, generated from metadata
where they can be (title page, copyright with imprint, ISBN and rights, contents, also by), each
editable; for a linked source, the source's own front and back matter used as they are, Book's
elements added beside them.
- Tests: each element generated from fixture metadata; edits round-trip through the format; a
  source's own copyright page is not duplicated.
- Done when: those pass.

**M3.2 `m3/chapters`**: STR-2, STR-3: chapter numbering on or off and its styles, titles,
subtitles, epigraphs, part pages.
- Tests: every number style for 1–120 (words, numerals, roman); parts numbered on their own.
- Done when: those pass.

**M3.3 `m3/metadata`**: META-1 and META-2 (ebook and paperback): the metadata panel showing the
source's metadata with Book's additions (contributors with roles, subject codes, imprint), editions
with ISBN (checksum validated), trim, paper and price notes.
- Tests: ISBN-10 and ISBN-13 checksums; Book's additions never written to a linked source;
  missing metadata listed from `book::missing_metadata` plus Book's own needs.
- Done when: those pass.

**M3.4 `m3/fixes`**: STR-7 and section 6.4: fix words in place in a section, one paragraph at a
time in a fix editor (changed from "one engine session": a light fix is words within one
paragraph, which a paragraph editor gives without Book carrying a full text editor; M1.5's
conversion stays for imports and the ebook preview); owned source edited; linked source offered
back through Manuscript's offers file, decisions read and shown.
- Tests: an offers file Book writes is accepted by Manuscript's `offers::read`; the Manuscript
  folder's bytes unchanged; each decision state (accepted, rejected, waiting, stale) shown as
  section 6.4 says; an existing offers file is appended to, never rewritten.
- Done when: those pass.

- Milestone done when: the v0.1 test books have complete front and back matter from metadata and
  every element round-trips through the format.

### M4 Designs

**M4.1 `m4/format`**: `book-design`: the design schema version 1, `docs/design-format.md`, reading
and writing, limits, validation with plain messages, the migration path (test-only version 0).
- Tests: round trips; every example in `docs/design-format.md`; hostile files; each validation
  message.
- Done when: those pass.

**M4.2 `m4/built-in`**: DSN-2: six designs (literary, romance, thriller, fantasy, non-fiction,
classic) written by us from typographic practice, with fonts by role from `assets/fonts/`
(open-licensed, listed in ATTRIBUTION.md).
- Tests: every design validates; every font it names is bundled and its license allows embedding;
  `cargo xtask assets` passes. Golden files follow in M5.4 and M6.3.
- Done when: those pass and SPEC lists the rules each design follows.

**M4.3 `m4/mixing`**: DSN-3, DSN-4: take a part of a design (chapter opening, body, scene breaks,
running heads) from another; save as the user's own; export and import one file.
- Tests: every pair of built-in designs mixes into a valid design; export → import is exact.
- Done when: those pass.

**M4.4 `m4/chooser`**: DSN-5: the design chooser with filters and a live preview of the book's
first chapter (a placeholder until M5.1 renders, then the real first page).
- Tests: filters; choosing a design is one undo step; the chooser through the egui harness.
- Done when: those pass.

### M5 Print

**M5.1 `m5/typeset`**: `book-typeset` from the spike to the full book: every block and mark of the
book view, notes printed as endnotes until R2's footnotes, images inline and full width with alt
text carried for tagging, the page map.
- Tests: every fixture typeset and its PDF checked letter by letter against the source; the
  text that looks like markup or field codes prints literally; the page map points every section at the
  page where it starts.
- Done when: those pass.

**M5.2 `m5/pages`**: PRT-1 (four trims), PRT-2 (gutter by page count, recomputed until stable),
PRT-3, PRT-4, PRT-5: mirrored margins, recto chapter starts, roman and arabic folios, running heads
(author and title, book and chapter, none) and the pages that suppress them, blank-page control.
- Tests: page sizes per trim; the gutter for page counts on each side of every threshold in the
  profile; recto starts; folio sequence; no running head on chapter openings or blank pages;
  blank-page runs within limits; the layout loop stops within four passes (section 9.2).
- Done when: those pass.

**M5.3 `m5/openings`**: chapter openings from the design (sink, number, title, ornament, image
area), drop caps, small-caps lead-ins, scene-break styles, part pages, widow and orphan control.
- Tests: each design's opening on the fixtures (position of the first line, drop cap lines);
  no page in the golden books ends with a single line of a paragraph or begins with one.
- Done when: those pass.

**M5.4 `m5/print-pdf`**: PRT-6 for `pod-paperback` and `generic-pdf`; golden PDFs for each design on
the v0.1 test books (text, page breaks, page boxes), recorded in `docs/golden.md`.
- Tests: the golden comparisons; two builds byte for byte; fonts embedded; no annotations,
  bookmarks or crop marks.
- Done when: those pass and the golden files are recorded (marked as awaiting a person's look).

### M6 Ebook

**M6.1 `m6/epub`**: `book-epub`: EBK-1, EBK-2: XHTML per section, CSS from the design's ebook
mapping, package document, navigation with landmarks, NCX; the package layer as a
`core-candidate` (C6); deterministic zip with `mimetype` first.
- Tests: the package's structure on every fixture; every navigation target exists; two builds byte
  for byte.
- Done when: those pass.

**M6.2 `m6/access`**: EBK-3, EBK-4, EBK-5: accessibility metadata, alt text required (export
blocked without it), fonts embedded or not, the cover.
- Tests: the metadata present and consistent with the content; an image without alt text blocks
  export; fonts present or absent per edition; the cover in the package and its metadata.
- Done when: those pass.

**M6.3 `m6/check`**: Book's own EPUB structure checks (container, package, manifest and spine,
navigation, ids, links, XHTML well-formed, CSS within the supported subset); the EPUB read back
through the core's reader and checked word for word; golden EPUB structure per design.
- Tests: each check against a broken EPUB written for it; every design's EPUB on the v0.1 test
  books passes and matches its golden structure.
- Done when: those pass; where the standard EPUB validator is installed it reports no errors
  (otherwise recorded as waiting on a person).

### M7 Preview

**M7.1 `m7/print-preview`**: PRV-1: spreads from the core's page images, trim, margin and bleed
guides, zoom, jump between a page and its section; background builds that never block the
interface; only pages in view drawn.
- Tests: through the egui harness, a page click opens its section and back; a frame stays under
  16 ms while a build runs.
- Done when: those pass.

**M7.2 `m7/ebook-preview`**: EBK-8 (section 5.8): device sizes, text size, dark page; "Open in a
reading app".
- Tests: each device size lays out the fixture's first chapter; text size and dark page change
  the layout and colors; the egui harness finds every control named.
- Done when: those pass.

**M7.3 `m7/live`**: changes to settings and designs update both previews; the time to the first
preview page after a design change recorded in `docs/performance.md`.
- Tests: a design change reaches both previews; the timing test on the 150,000-word book.
- Done when: the first page shows under 1 s after a design change.

### M8 Preflight and export

**M8.1 `m8/preflight`**: PRE-1 and PRE-4 in `book-check`, on C9 (PDF facts promoted into the core
first): every rule of section 9.4 with severity and links; the preflight panel.
- Needs: C9 in a core release (released in core v0.1.10 during M5.4).
- Tests: for each rule, a book written to break it and one that passes; each finding links to the
  right page or section.
- Done when: those pass and the v0.1 test books pass preflight for `pod-paperback`.

**M8.2 `m8/validators`**: PRE-2 (EPUB validator): a validator the user installed (its path in the
preferences) run as a separate program with a time limit, its report read into preflight; "not
installed" is a note, never a failure.
- Tests: report parsing from recorded sample output written for the test; a missing program; a
  program that runs past the time limit.
- Done when: those pass.

**M8.3 `m8/export`**: EXP-1, EXP-2: the export page (editions, formats, where), the project
bundle, the word check before any file is written, reproducible output.
- Tests: every export twice, same bytes; a failing word check writes nothing; the bundle reopens
  as the same project.
- Done when: those pass.

### M9 Release

**M9.1 `m9/agents`**: every Book command as an agent tool (AUT-2).
- Tests: the tool list equals the command list; a book imported, designed, built and checked
  through the tools only.
- Done when: those pass.

**M9.2 `m9/cli`**: `book-cli` (AUT-1): `build --edition <id>` (or all), `check`, `verify`,
`convert` (a file to a Book project with an owned source), `report`, `run`, `commands`, `mcp`;
exit codes for scripts.
- Tests: each command on the fixtures, including exit codes for a failing check.
- Done when: those pass.

**M9.3 `m9/guide`**: the user guide in `docs/guide/` (getting started, import, structure, designs,
print, ebook, preview and preflight, export, editions and metadata, working with Manuscript,
command line and agents, keys).
- Tests: the keys page made from the commands by a test that also refuses two commands sharing a
  key.
- Done when: every 0.1 feature id appears in the guide.

**M9.4 `m9/release`**: the test of v0.1 as an end-to-end test; packaging for the three desktop
platforms (with the project types registered); the release workflow (gate and v0.1 test on three
platforms, packages, attestation, release from the changelog) run when a maintainer pushes the
tag; update checks from this repository's releases; CHANGELOG.md for 0.1.0.
- Tests: the test of v0.1.
- Done when: it passes locally and the full `cargo xtask ci` passes; the release itself waits for
  a person's approval.

### R2 (v0.2): parity for most novels

Each step is named `r2/<name>`, runs in this order, and is done when its tests pass and the
v0.1 test still passes.

| Step | Builds | Tests |
| --- | --- | --- |
| R2.1 `r2/trims` | All common trims and metric sizes (PRT-1) | Page sizes and gutters for each trim |
| R2.2 `r2/hardcover` | Hardcover and its profile (PRT-8, PRE-4) | Margins and limits per profile |
| R2.3 `r2/bleed` | Bleed and full-page images (PRT-7, TXT-4) | Page boxes with bleed; image placement |
| R2.4 `r2/notes` | Footnotes in print, pop-up notes or endnotes in the ebook (TXT-3, EBK-7) | Notes on the page they are called from, split across pages; ebook links both ways; word checks with notes |
| R2.5 `r2/core-epub-notes` | C7 in the core (additive patch release) | The core writer's notes, in the core |
| R2.6 `r2/asides` | Asides (TXT-2) | Each aside kind in print and ebook |
| R2.7 `r2/matter` | More front and back matter (STR-8) | Each element generated and placed |
| R2.8 `r2/only` | Print-only and ebook-only (STR-9) | Content present in one output only |
| R2.9 `r2/store-editions` | Store editions (STR-10) | Each edition carries only its own links |
| R2.10 `r2/page-list` | Page list (EBK-6) | Every print page has its marker in reading order |
| R2.11 `r2/cover` | Cover template and cover check (PRT-9, PRT-10) | Template sizes for page counts and papers; good and bad covers |
| R2.12 `r2/reimport` | Re-import with differences (IMP-8) | Design, metadata and matter kept; differences listed word by word |
| R2.13 `r2/typography` | Typographic warnings (PRE-3) | Each warning on a page written to show it |
| R2.14 `r2/access-check` | The accessibility checker (PRE-2) | Report parsing; missing program |
| R2.15 `r2/hints` | No-break text and hyphenation hints (TXT-5) | Hints honoured in print and kept in the ebook |
| R2.16 `r2/languages` | Interface languages on the promoted catalog (ACC-3, C4 in the core first) | Every interface text in every catalog; no unused texts |
| R2.17 `r2/access` | Screen readers (ACC-2) | The egui harness finds every widget named |
| R2.18 `r2/close` | CHANGELOG for 0.2.0, guide updated | The v0.1 test and the full `cargo xtask ci` |

### R3 (v0.3): designs, series and standards

| Step | Builds | Tests |
| --- | --- | --- |
| R3.1 `r3/designs` | Six to ten more designs, including large print (DSN-6) | Each validates and has golden files |
| R3.2 `r3/large-print` | The large-print variant of any design (PRT-11) | 16 pt and up on every design; preflight passes |
| R3.3 `r3/box-sets` | Box sets (STR-11) | Each book keeps its matter; one main contents; word checks per book |
| R3.4 `r3/epub-import` | EPUB import (IMP-9) | An exported EPUB imports back word for word |
| R3.5 `r3/standards` | Tagged PDF/UA-1 and PDF/A-2b (PRT-12), through the core's PDF writer (tagged already; PDF/A as a core change, or recorded as blocked) | Tags and metadata present; the standards' validator where installed |
| R3.6 `r3/proof` | Proof PDFs (PRT-13) | Watermark and line numbers present; never the default export |
| R3.7 `r3/close` | Polish from use, CHANGELOG for 0.3.0 | The v0.1 test and the full `cargo xtask ci` |

### R4 (v1.0): publishing house

| Step | Builds | Tests |
| --- | --- | --- |
| R4.1 `r4/house-styles` | House styles (DSN-7) | A locked design cannot be changed in a book of that imprint or series |
| R4.2 `r4/imprints` | Imprint profiles (META-3) | The copyright page and title page from the imprint |
| R4.3 `r4/series` | Series view (META-4) | "Also by" built from the series; one design across the books |
| R4.4 `r4/isbns` | ISBN list (META-5; C11 promoted if the table is needed) | Each ISBN used once; checksums |
| R4.5 `r4/onix` | ONIX for Books 3.0 export (META-6) | The record validates against the published schema's structure |
| R4.6 `r4/engine` | Print moved to the newest core release's layout | Every golden file compared; changes reviewed and recorded |
| R4.7 `r4/close` | CHANGELOG for 1.0.0, guide complete | The v0.1 test and the full `cargo xtask ci` |

## 11. Testing and quality gates

| Gate | What | When it runs |
| --- | --- | --- |
| Every step | `cargo test` for affected crates, `cargo clippy --workspace --all-targets` (warnings are errors, including the core's complexity limit), `cargo xtask app-layers`, `cargo xtask privacy` | Every step |
| The core's gate | `cargo xtask ci`: format, lint roots, layers, pins, template, privacy, licenses, clippy, tests; `cargo deny check` | Before a release (and in CI when CI is used) |
| Format round trips | Book project and design files, both directions, random projects | Every step touching them |
| Word for word | Every importer and exporter on every fixture: import against source; PDF letter by letter; EPUB read back | Every step touching them |
| Golden output | Print PDF text, page breaks and page boxes; EPUB structure; per design on the v0.1 test books | Every step touching print, ebook or designs |
| Reproducible | Every build twice, same bytes | Every step touching print or ebook |
| No markup injection | Book text that looks like markup or field codes prints literally | Every step touching `book-typeset` |
| Data safety | Crash injection during save; damaged and hostile files (sizes, depth, counts, bad UTF-8, broken zip, hostile DOCX and EPUB) | Every step touching files |
| Never writes Manuscript | Every test that uses a linked source checks the Manuscript folder's bytes are unchanged | Every step touching sources |
| Scale | A 300,000-word book: opens under 2 s, typesets a 6 × 9 paperback under 30 s, first preview page under 1 s after a design change (recorded in `docs/performance.md`) | Each milestone from M5 |
| Interface | egui test harness for the outline, import review, design chooser, previews and preflight | Every step touching them |
| End to end | The test of v0.1 | From M9.4 |

Private manuscripts may be run through the same tests locally from `.private/`; their results are
never committed.

## 12. Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Changing an author's words | Fatal for trust | Word-for-word checks on every import and export; cleanup listed and reversible; linked sources never written |
| The embedded engine did not fit the core's patched PDF libraries | Print on the core's layout instead (decision 0003) | Done in M0.7; Book's passes add what a book needs; gaps are core changes or recorded as blocked |
| The core's layout changes between releases | Print output moves | Golden files; core moves are their own step (R4.6) |
| Word-processor layout falls short of book typography | Lower quality than a typesetting engine | Book's own passes (right-hand starts, gutter, optimal-fit breaking); designs reviewed by a person; revisit decision 0003 if the engine becomes buildable |
| No PDF/X output | Some print services ask for PDF/X | Most print-on-demand services take plain PDF with embedded fonts; PDF/X recorded as blocked upstream (PRT-14), not waited on |
| Manuscript has no release tags yet | Book pins a non-release tag (`book-view-1`) | Decision 0001 records it; Book moves to a release tag when a step needs one |
| Two copies of the core (Book ahead of Manuscript's pin) | Slower builds, more disk | No `jcj-` types cross the boundary; request Manuscript to move; move Book's Manuscript reference when one exists |
| Disk space (a large build cache) | Builds fail mid-step | `CARGO_INCREMENTAL=0`; `cargo clean` below 15 GB free; no other repositories' `target/` kept |
| Ebook rendering differs between reading systems | Ugly ebooks on some devices | Conservative CSS per design; store editions; the preview labelled as one; manual checks by a person before release |
| Messy word-processor files | Wrong structure, lost italics | Import review with editable rules; the "online word processor" fixture from M0.5 |
| Typography not good enough | Low adoption | Designs reviewed against SPEC's rules; golden files; a person reviews each design before release |
| Naming a company or product by habit (stores, print services, fonts) | Public-repo rule broken | Section 4 rules 2, 4, 5; `cargo xtask names` checks tracked files and commit messages against a word list kept only in `.private/names.txt` (the list itself would name them, so it is never committed), whole words only, with fonts' own files, `ATTRIBUTION.md` and the public-domain texts exempt; presets named by trim and format. Until 2026-11-26: project decision 0003 replaced the rule and removed the check |
| External validators not installed | Checks skipped | Book's own structural checks always run; external results are additional |
| Scope creep into writing | Duplicates Manuscript | Light fixes only; anything more opens Manuscript |
| Font licensing | Rejected files, legal exposure | Embedding bits read in preflight; bundled fonts only open-licensed |

## 13. Open questions

Each needs a person's answer; record it as a decision and update this plan.

None open for the build. Answered on 2026-11-24 (decisions 0001 and 0003 record them):

1. **Core changes.** Additive core changes with tests and a use in `core-testbed` are released as
   patch releases (0.1.x) and Book moves to them. A minor release, an upstream pin move or a fork
   change needs a person; until then Book builds around it. Promotions move code from an app into
   the core, never from one app into another.
2. **Manuscript reference.** Book depends on Manuscript's tag `book-view-1` (commit `952d226`), a
   non-release tag for the book view; it passes the core's source rules once C10 lists
   Manuscript's address.
3. **The typesetting engine** was to be embedded in Book (`book-typeset`, core-candidate C5), not in the
   core yet (section 5.6). The spike failed against the core's patched PDF libraries; print is built on the core
   instead (decision 0003).
4. **Core-candidates** each have a core issue; `docs/core-candidates.md` is the index here.
5. **Format versions**: version 1 is extended in place until Book's first release tag (section
   6.1).
6. **The hand-off review**: Book's side is written in `docs/handoff-review.md` (step M1.2) and
   signed off by a person.

Still open for people, not blocking the build: who reviews the built-in designs before 0.1 is
released; files saved by real word processors for the fixtures.

## 14. Third-party building blocks

Besides open-source-core, Wordsmith Manuscript's format crates and what they bring in. Licenses
are re-checked before each is added; `cargo xtask licenses` and `cargo deny` enforce the allowlist.

| Project | Use | License | When |
| --- | --- | --- | --- |
| [typst](https://crates.io/crates/typst), `typst-pdf`, `typst-render` | Not used: proposed for print, failed the M0.7 spike against the core's patched PDF libraries (decision 0003) | Apache-2.0 | – |
| [toml](https://crates.io/crates/toml) | `book.toml`, designs, print profiles (already in the core's lock file) | MIT OR Apache-2.0 | M1.1 |
| [uuid](https://crates.io/crates/uuid) | Ids for Book's own elements and offers | MIT OR Apache-2.0 | M1.1 |
| [zip](https://crates.io/crates/zip) | `.wsbkz` and EPUB packing, at the core's version and with its reading rules | MIT | M1.3 |
| [ttf-parser](https://crates.io/crates/ttf-parser) | Font names and embedding bits in preflight (likely already in the lock file) | MIT OR Apache-2.0 | M8.1 |
| [image](https://crates.io/crates/image) | Image sizes and resolution checks | MIT OR Apache-2.0 | M5.1 |
| [proptest](https://crates.io/crates/proptest) | Property tests (dev only) | MIT OR Apache-2.0 | M1.1 |
| Open-licensed fonts | Bundled faces for the designs, named in `ATTRIBUTION.md` | OFL-1.1 | M4.2 |
| The standard EPUB validator | Run as a separate program the user installed (with the runtime it needs) | BSD-3-Clause | M8.2 |
| An EPUB accessibility checker | Run as a separate program the user installed | MIT | R2 |
