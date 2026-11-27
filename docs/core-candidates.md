# Core changes and core-candidates

Every core change Book needs and every `// core-candidate:` in this repository, with its core
issue and state (plan section 5.5). States: **released in core vX** (promoted or changed in the
core), **in core vX, used here** (the core has it and Book uses the core's; its issue stays open
while another app still holds a copy of its own, as the core's rule for a promotion says),
**app-first** (built here, marked `// core-candidate: open-source-core#N`, waiting for a second
app), **open** (not started), **not needed**.

The rows, the markers in the code and the core's issues are kept in step: a row says "released"
only when its issue is closed, and each app-first row's issue is open, labelled `core-candidate`
and names the file here that holds the code. Last checked against the issues on 2026-11-27.

| Id | What | `// core-candidate:` location | Core issue | State |
| --- | --- | --- | --- | --- |
| C1 | Word-for-word verification (`jcj_features::verify`) | – | [#10](https://github.com/jamescjfoundation/open-source-core/issues/10) | released in core v0.1.9 |
| C2 | Word-level comparison of two texts (`jcj_features::wordiff`) | – | [#17](https://github.com/jamescjfoundation/open-source-core/issues/17) | released in core v0.1.9 |
| C3 | A tree widget with drag and drop (`jcj_shell::tree`) | – | [#6](https://github.com/jamescjfoundation/open-source-core/issues/6) | released in core v0.1.8 |
| C4 | Interface translation catalogs (`jcj-i18n`; the shell's texts through them) | – | [#18](https://github.com/jamescjfoundation/open-source-core/issues/18) | released in core v0.1.13 |
| C5 | Book typesetting (a book layer over the core's layout, decision 0003) | `crates/book-typeset/src/lib.rs` | [#19](https://github.com/jamescjfoundation/open-source-core/issues/19) | app-first |
| C6 | EPUB 3 package layer | `crates/book-epub/src/package.rs` | [#20](https://github.com/jamescjfoundation/open-source-core/issues/20) | app-first |
| C7 | The core's EPUB writer drops notes | – | [#21](https://github.com/jamescjfoundation/open-source-core/issues/21) | released in core v0.1.12 |
| C8 | Structure detection for imports | `crates/book-import/src/detect.rs` | [#22](https://github.com/jamescjfoundation/open-source-core/issues/22) | app-first |
| C9 | PDF facts for preflight | – | [#23](https://github.com/jamescjfoundation/open-source-core/issues/23) | released in core v0.1.10 (`jcj_output::read_facts`; brought forward to M5.4) |
| C10 | Manuscript's repository allowed as a git source in `deny.toml` | – | [#15](https://github.com/jamescjfoundation/open-source-core/issues/15) | released in core v0.1.5 |
| – | `cargo xtask sync` finds the app's own core when two core releases are in the graph | – | [#16](https://github.com/jamescjfoundation/open-source-core/issues/16) | released in core v0.1.6 |
| – | The shell's auto-save number fields named for screen readers (found by R2.17's test) | – | [#26](https://github.com/jamescjfoundation/open-source-core/issues/26) | released in core v0.1.14 |
| – | PDF/A-2b and PDF/UA-1 declarations on exported PDFs (`jcj_output::declare`, R3.5) | – | [#27](https://github.com/jamescjfoundation/open-source-core/issues/27) | released in core v0.1.15 |
| C11 | A sortable, editable table widget | – | [#7](https://github.com/jamescjfoundation/open-source-core/issues/7) | not needed (R4.4: the house's ISBN list is edited in the house file and shown by `book.isbn.list`; each edition takes its ISBN from Details); the issue stays open for another app |
| C12 | The PDF names the app that made it | – | [#5](https://github.com/jamescjfoundation/open-source-core/issues/5) | open; no step needs it. Book's PDFs come from `jcj-output` (decision 0003), whose settings have no field for it, so they do not name Book. The name is kept ready, unused (`CREATOR` in `crates/book-typeset/src/lib.rs`) |
| – | `jcj-doc`: section settings and note kinds reachable | – | [#24](https://github.com/jamescjfoundation/open-source-core/issues/24) | released in core v0.1.7 |
| – | Trim and bleed boxes on exported PDFs (`jcj_output::set_page_boxes`) | – | [#25](https://github.com/jamescjfoundation/open-source-core/issues/25) | released in core v0.1.11 |
| – | EPUB import (`jcj_interchange::import` with `Format::Epub`, used by IMP-9) | – | [#13](https://github.com/jamescjfoundation/open-source-core/issues/13) | released in core v0.1.3, before Book began; the issue is still open. The reader keeps no notes and gives back no structure from the package, so `crates/book-import/src/epub.rs` takes the notes out before the core reads the file and puts them back, and reads the package's roles itself |
| – | The translation lookups through the shell (`jcj_shell::i18n`: `tr`, `tr_with`, `trn`, `tl`, and the catalog test's `Catalog`, `texts_in`, `singulars_in`, `compare`) | – | [#32](https://github.com/jamescjfoundation/open-source-core/issues/32) | in core v0.1.20, used here: since 2026-11-26 `crates/book-ui` has no `jcj-i18n` line and the layer table does not name it |
| – | The system's file dialogs through the shell (`Shell::ask_file`, `jcj_shell::files`) | – | [#28](https://github.com/jamescjfoundation/open-source-core/issues/28) | in core v0.1.20, used here: since 2026-11-26 the window's *Open…* is the system's dialog, in place of a box to type a path into |
| – | An app's own commands as agent tools, and one convention for command lines (`jcj_agents::AppCommands`, `AgentServer::for_app`, `jcj_agents::cli`; core decision 0017) | – | – | released in core v0.1.20; since 2026-11-26 Book's own adapter (engine commands standing for Book's, around a global) is gone and `crates/book-ui/src/agents.rs` gives Book's commands to the core's |
| – | A file dialog that cannot be shown runs neither of its commands, so an app cannot fall back to a typed path | – | [#43](https://github.com/jamescjfoundation/open-source-core/issues/43) | open; no step waits on it. Where the system can show no dialog, *Open…* says so in the status bar; a book still opens from the recent list or from a path given when the app starts |
| – | A check for files an app carries from elsewhere, against a record with checksums (the brand kit's files in `assets/brand/`) | `xtask/src/brand.rs` | [#47](https://github.com/jamescjfoundation/open-source-core/issues/47) | app-first: `cargo xtask brand`, in the gate since 2026-11-27. Wordsmith Manuscript holds the same check for its own folder |
| – | One start-up for every desktop app (`jcj_shell::desktop`), and brand loading beside the theme | – | [#46](https://github.com/jamescjfoundation/open-source-core/issues/46) | in core v0.1.21, not used here yet: `apps/book/src/main.rs` keeps its own start-up, with the updater's three steps since 2026-11-27. Moving onto the core's is [issue 9](https://github.com/jamescjfoundation/wordsmith-book/issues/9) here |
| – | One release contract and one pipeline for every app (`cargo xtask package`, `cargo xtask release`, the shared release workflow; core decision 0019) | – | – | released in core v0.1.22, used here since 2026-11-27: `packaging/` and Book's own release workflow are gone, and what a release carries is data in `xtask/src/main.rs` |
