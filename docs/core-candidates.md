# Core changes and core-candidates

Every core change Book needs and every `// core-candidate:` in this repository, with its core
issue and state (plan section 5.5). States: **released in core vX** (promoted or changed in the
core), **app-first** (built here, marked `// core-candidate:`, waiting for a second app),
**open** (not started), **not needed**.

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
| C11 | A sortable, editable table widget | – | [#7](https://github.com/jamescjfoundation/open-source-core/issues/7) | open (R4, if needed) |
| C12 | The PDF names the app that made it | – | [#5](https://github.com/jamescjfoundation/open-source-core/issues/5) | not needed by Book (its engine sets the producer) |
| – | `jcj-doc`: section settings and note kinds reachable | – | [#24](https://github.com/jamescjfoundation/open-source-core/issues/24) | released in core v0.1.7 |
| – | Trim and bleed boxes on exported PDFs (`jcj_output::set_page_boxes`) | – | [#25](https://github.com/jamescjfoundation/open-source-core/issues/25) | released in core v0.1.11 |
