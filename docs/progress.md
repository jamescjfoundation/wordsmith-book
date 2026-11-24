# Progress

One line per finished step on `main`, newest last. Feature ids are the plan's (section 7).
**Tests** is the sum of the "passed" counts of every `test result:` line of
`cargo test --workspace` at the end of the step.

| Date | Step | What changed | Tests |
| --- | --- | --- | --- |
| 2026-11-24 | M0.1 m0/template | Workspace from the core's template: eight crates and two apps with crate-root rules, licenses, NOTICE, SECURITY, CONTRIBUTING, conduct, ATTRIBUTION, AGENTS.md with Book's own rules; `app-layers`, `assets` and `names` tasks; Manuscript's format and model at `book-view-1`; C10 released as core v0.1.5 and the sync fix for two cores in one graph as core v0.1.6, Book pinned to v0.1.6; decision 0001; the empty app's window shows Book's name (harness test) | 31 |
| 2026-11-24 | M0.2 m0/name | Decision 0002: `.wsb` is taken on one desktop platform, so a Book project is `.wsbk` and the packed project `.wsbkz`; the plan changed to match. Note: the core's pre-push hook cannot diff against `origin/main` on the first push to an empty repository; the first push set a local `origin/main` to the plan commit so the hook checked everything | 31 |
| 2026-11-24 | M0.3 m0/spec | `docs/SPEC.md`: behavior for every 0.1 feature id (indexed), each statement marked Verified, Sourced or Designed, standards cited by address, service rules recorded generically; awaiting a person's review | 31 |
| 2026-11-24 | M0.4 m0/brand | `assets/brand/book/`: tokens (light, dark, extra-dark), the mark and app icons drawn by `tools/brand-icons.py`; the window uses the tokens, the mark and the icon; interface fonts are the core's bundled OFL faces | 33 |
| 2026-11-24 | M0.5 m0/fixtures | Three public-domain novels as text (*Waverley*, parts, notes and verse; *Gulliver's Travels*; *Middlemarch*, 316,000 words for scale), DOCX, ODT, RTF and Markdown with real notes and a DOCX shaped like an online word processor's download, all by `tools/make-fixtures.py`; `docs/fixtures.md` and a test that every fixture is listed; files from real word processors to add by a person | 34 |
| 2026-11-24 | M0.6 m0/core-issues | Core issues opened for C2, C4 to C9 (open-source-core #17 to #23; C5, C6 and C8 labelled `core-candidate`) beside C10's #15; every row of plan 5.5 links its issue; `docs/core-candidates.md` lists them all | 34 |
| 2026-11-24 | M0.7 m0/engine | The engine spike failed: the engine at 0.15.1 does not compile against the core's patched `krilla` and `hayro`; decision 0003 takes the plan's fallback (print on the core's layout, PDF and page images from `jcj-output`) and the plan changes with it. `book-typeset` builds the print document (sections, mirrored margins, heads, folios, small caps, drop caps, right-hand starts by blank pages over at most four passes); the classic serif bundled by role; core v0.1.7 (section settings and note kinds reachable, #24); a Waverley chapter prints letter for letter, two builds give the same bytes, page images at the trim's size; M0 done | 38 |
| 2026-11-24 | M1.1 m1/format | book-model (the Book layer: source, editions, design choice and mix, numbering, metadata additions, elements, settings per section, cleanup, fixes) and book-format (`book.toml` schema 1 with the version-0 migration path, designs, assets and build records, limits, changed-files-only atomic saving with `book.toml` last); `docs/format.md` with every file's example; round trips, property tests, hostile input, a newer schema refused | 47 |
| 2026-11-24 | M1.2 m1/sources | Linked Manuscript projects (folders and `.wsmz`) through the book view, other view versions refused, the owned source written and read through Manuscript's folder writer, word checksums per section and changed sections, outside changes noticed (FIL-2); the Manuscript project fixture by `cargo xtask fixtures`; `docs/handoff-review.md` (awaiting sign-off) and requests 2 to 7 to Manuscript; every test leaves the Manuscript folder's bytes unchanged | 53 |
| 2026-11-24 | M1.3 m1/save | Packed projects (`.wsbkz`: deterministic, `mimetype` first, sorted, fixed times; the owned source's files or a packed copy of a linked source inside), bounded unpacking into a new folder, recovery around every save (`save_safely`), rotating backups with a number kept; crash injection after every write reopens whole with the work saved or recoverable; hostile and damaged packs refused. The shell's auto-save is wired with the app's projects in M1.4 | 59 |
| 2026-11-24 | M1.4 m1/outline | C3 promoted into the core as `jcj_shell::tree` (core v0.1.8, #6) and Book moved to it; the app opens and creates books (start window templates, an open dialog, `file.openPath`, a path on the command line, `.wsbkz` unpacked beside it), auto-saves through the shell, offers recovered work; the outline panel (front matter, parts, chapters, back matter; Book's elements move by drag, the source's sections are read-only with "change this in Manuscript"); every change a command with undo; harness tests including 1,000 sections under 50 ms a frame; M1 done | 63 |
| 2026-11-24 | M1.5 m1/document | Book's own conversion between the book view's text (every block, mark and note) and `jcj_doc::Document` with `book.` keys, in book-model; property tests that text → document → text is exact; every section of the Manuscript fixture converts exactly; formatting the view cannot hold is dropped and listed | 66 |

## Waiting on a person

- **Requests to Manuscript:** `docs/requests-to-manuscript.md` (moving to the newest core, the contract's
  extra parts, comment anchors, a cover role, editions, raw blocks, a release tag).

- **Print engine (decision 0003):** the embedded typesetting engine at 0.15.1 does not build against
  the core's patched `krilla` and `hayro` (craft-libs' copies). Using it needs a person to move the
  core's patch block or craft-libs' copies, or to carry a fork; until then print is on the core's
  layout.

- **Fixtures:** DOCX, ODT and RTF saved by real word processors, and a DOCX downloaded from an
  online word processor (`docs/fixtures.md`).

- **Reviews:** `docs/SPEC.md` (M0.3); Book's side of the hand-off review, `docs/handoff-review.md` (M1.2). Every [Sourced] rule from a service's public file guide is to
  be rechecked against the current guide, its address added to `.private/sources.md`.

- **Names:** recheck decision 0002's finding that `.wsbk` and `.wsbkz` are free on the three desktop
  platforms before packaging (M9.4).

- **Core:** move Wordsmith Script and Wordsmith Manuscript to the newest core release (core
  v0.1.8 so far; every release in this run is additive).
