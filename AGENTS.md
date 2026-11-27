# Notes for coding agents (and people)

Wordsmith Book is built on [open-source-core](https://github.com/jamescjfoundation/open-source-core),
The James C. J. Foundation's shared base. The core holds what every app needs; this repository
holds only what is about this app.

The blocks below marked `open-source-core` are the core's rules, which every app carries word for
word. `cargo xtask sync` writes them from the pinned core release and `cargo xtask sync --check`
fails when one differs, so they are changed in the core, never here. The block marked
`wordsmith project block` is owned by the Wordsmith project map and is changed there. Everything
after the marked blocks is Book's own.

<!-- BEGIN wordsmith project block -->
## The Wordsmith project

Wordsmith is five repositories and two forks, all under [jamescjfoundation](https://github.com/jamescjfoundation). The full map, the steps of every route and the project's rules are in the [project map](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md), which the wordsmith repository owns.

| Repository | Owns |
| --- | --- |
| [wordsmith](https://github.com/jamescjfoundation/wordsmith) | The project map and rules (`project/`), the brand kit (`brand/`), the website (`website/`), the wiki (`website/wiki/`), the launcher (planned) |
| [wordsmith-script](https://github.com/jamescjfoundation/wordsmith-script) | Wordsmith Script, the screenplay app, and the `.wss` file format |
| [wordsmith-manuscript](https://github.com/jamescjfoundation/wordsmith-manuscript) | Wordsmith Manuscript, the novel and long-form app, the `.wsm` project format, and the book view that Book reads |
| [wordsmith-book](https://github.com/jamescjfoundation/wordsmith-book) | Wordsmith Book, the book design and typesetting app, and the `.wsbk` project format |
| [open-source-core](https://github.com/jamescjfoundation/open-source-core) | The shared base: the `jcj-` crates and the settings and tooling every app carries. It is general and names no product |
| [wordcraft](https://github.com/jamescjfoundation/wordcraft), [craft-libs](https://github.com/jamescjfoundation/craft-libs) | Our forks of the upstream projects the core builds on |

- Each app builds on open-source-core at one release tag and carries its shared files (`cargo xtask sync`). Only the core reaches the forks.
- Wordsmith Book also builds on Wordsmith Manuscript's model and format crates, at a book-view tag. Wordsmith Manuscript reads Wordsmith Script's `.wss` files.
- Each app takes its logos, icons and palette from the brand kit, unchanged, by the kit's tool. Each app's `docs/guide/` is its one user guide: the wiki's pages for the app are built from it. The website and the wiki read each app's latest release from its GitHub releases.

| The change | Where it goes | Route |
| --- | --- | --- |
| A fix in a `jcj-` crate, or something any app would share | open-source-core first, then the app's pin | [A](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-a) |
| Something one app needs from another app | An issue in the providing app's repository | [B](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-b) |
| A change users will see in an app | The app's `docs/guide/` and `CHANGELOG.md`; then wordsmith builds the wiki's pages from the guide | [C](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-c) |
| A wiki page | About one app: that app's `docs/guide/`, never the wiki's copy. About the project: wordsmith, `website/wiki/content/docs/start/` | [D](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-d) |
| Website text, or the words for an app's state on the site | wordsmith, `website/tools/build.py` and `website/data/apps/` | [E](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-e) |
| A logo, icon or brand color | wordsmith, `brand/source/`; then its tool writes the copies into each app's `assets/brand/` | [F](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-f) |
| Screenshots | Drawn in the app into `docs/images/`; wordsmith's tool carries them | [G](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-g) |
| A security problem | Never public: `.private/` and a maintainer | [H](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-h) |
| A core release, or moving an app to a newer core | open-source-core, then each app | [I](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-i) |
| An app release | The app; the website and wiki pick it up by themselves | [J](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-j) |
| A change in WordCraft or craft-libs | Our fork, then the core | [K](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-k) |
| A change that spans two repositories | The provider first; each change names the other | [L](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-l) |
| Working with all the repositories, or with only one | Side by side as sibling folders | [M](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-m) |
| A rule or instruction that several repositories carry | The block's owner: the core's `template/`, or the project map for this block | [N](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-n) |

Before changing a repository, read its own `AGENTS.md`. Its rules apply inside it; this block applies in all of them. The block is owned by the project map and copied from it by a tool: it is never edited in a copy.
<!-- END wordsmith project block -->

<!-- BEGIN open-source-core: app -->
<!-- Owned by open-source-core (template/AGENTS.md). Change it there, never in a copy: its tooling rewrites the copies. -->
## Before you commit

```sh
cargo xtask install-hooks   # once: pre-commit (fmt, privacy, no dev-link in a tracked file) and pre-push (ci --changed)
cargo xtask ci --changed    # what CI will run for your change; `cargo xtask ci` for everything
cargo xtask sync            # after moving to a new core release: shared files, blocks and patch block
```

What the core owns is changed in the core, never here:

- whole files: `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml` (but for the
  app's own sources in it, which `sync` writes from `xtask/src/main.rs`), `CODE_OF_CONDUCT.md`,
  `xtask/Cargo.toml`, every file in `xtask/src/` but `main.rs` and the modules the app adds,
  `.github/workflows/ci.yml` and `core-pin.yml`, `.github/pull_request_template.md`,
  `.github/dependabot.yml` and `hooks/`;
- the blocks marked `open-source-core` in files that are otherwise the app's: in `AGENTS.md`,
  `SECURITY.md` and `CONTRIBUTING.md`, and in `Cargo.toml` (lints and build profiles),
  `.cargo/config.toml`, `.gitignore` and `.gitattributes`;
- the patch block in `Cargo.toml`.

`cargo xtask sync --check` fails if any of them differs from the pinned core release.

`xtask/src/main.rs` is the app's own and holds only what is the app's: its data (the product's
name, the programs it ships, the repositories it builds on besides the core) and its own tasks.
The gate, its list of checks and how each is run are the core's (`xtask/src/gate.rs`), so a
check the core adds runs here after `cargo xtask sync`, with no edit. The app's layers are the
table marked `layer table` in this file: `cargo xtask app-layers` reads that table, so the table
is the rule, and a layer is changed there in the same change as the code. The two workflow
files only say when CI runs: the jobs are the core's shared workflows, which they call, so a fix
to CI is made in the core and reaches the app without a change here.

## House rules for code

- Every crate root: an SPDX line, `#![forbid(unsafe_code)]`, and `#![deny(...)]` of the six
  panicking lints. Product code returns errors; it never unwraps or panics.
- Depend only on the core's `jcj-` crates and, under a contract, another app's crates
  ([core decision 0015](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0015-apps-that-depend-on-apps.md));
  never on `jcj-upstream`, `wordcraft-*` or `craft-*` (`cargo xtask layers`).
- The app's brand (its name, logo, icons, fonts and colors) lives in the app's repository, or in
  its project's brand kit with copies in the app, never in the core. The app's interface theme
  is its own `tokens.json`, which the core's theming code reads.
<!-- END open-source-core: app -->

<!-- BEGIN open-source-core: public -->
<!-- Owned by open-source-core (template/AGENTS.md). Change it there, never in a copy: its tooling rewrites the copies. -->
## What is public

This repository is public, history included, forever
([core decision 0012](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0012-working-in-the-open.md)).
Before you write a commit, branch name, issue, pull request or comment, check it holds none of:

- anyone's name, email, account names, or details of their machine (local paths such as a home
  folder, user names, host names, IP addresses);
- what the user told you in conversation, quoted or summarised; write the reason for a change as
  its own argument;
- an open security problem (follow `SECURITY.md`: keep it in `.private/` and tell the user);
- secrets, or files from a real machine (preferences, recovery, auto-save, backups, logs,
  screenshots of a desktop);
- anyone's private writing, even as test data. Fixtures are public-domain or written for the test.

Commit with a GitHub no-reply address or a foundation bot identity. Where a repository has
`cargo xtask`, `cargo xtask privacy` checks its files and every commit. Never push with `--all`
or `--mirror`.

The Foundation's name is always written "The James C. J. Foundation", with its capital "The"
wherever it stands in a sentence, never in a shorter form.
<!-- END open-source-core: public -->

<!-- BEGIN open-source-core: app-and-core -->
<!-- Owned by open-source-core (template/AGENTS.md). Change it there, never in a copy: its tooling rewrites the copies. -->
## Building between an app and the core

"The core" is [open-source-core](https://github.com/jamescjfoundation/open-source-core); "an app"
is any repository built on it. A record of the core is cited from anywhere as "core decision
0011", with its repository named, never as a bare number: an app has records with the same
numbers.

### Step 1: decide where the change belongs

Ask in order; the first "yes" decides.

1. **Does it change how we use WordCraft, craft-libs or another upstream project?** The core's
   adapters (`jcj-upstream` and the `jcj-` crate that wraps it). If upstream code itself must
   change, it goes in our fork first (the core's `docs/UPSTREAM.md` and
   [core decision 0004](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0004-follow-hold-own.md)).
2. **Would it look or work the same in any foundation app?** The window, ribbon, panels, dialogs,
   File page, preferences, auto-save, updates, saving and backups, spelling, find and replace,
   undo, format import and export, PDF and printing, cards and boards, theming, entries kept
   beside documents, interface translations, agent control: the core.
3. **Does a second app already need it, or have a written plan to use it?** The core.
4. **Otherwise** the app. Screenplay elements, Fountain, scene numbers, (MORE)/(CONT'D), a
   manuscript binder's compile rules and a book's themes are app code.

When unsure, put it in the app: moving code into the core later is cheap; taking a wrong general
interface back out is not.

| Example | Where | Why |
| --- | --- | --- |
| Redesigned ribbon, new panel docking | Core (`jcj-shell`) | Every app has a ribbon and panels |
| A new app-specific ribbon tab | The app | Only the tab's contents are the app's |
| Faster auto-save, better backups | Core (`jcj-project`, `jcj-shell`) | Same for every app |
| Index cards a second app also wants | Core (`jcj-board`) once it needs them | Second app |
| DOCX import fix | Core (`jcj-interchange`), offered upstream too | General |
| Colors, fonts, logo | The app's repository or its project's brand kit | The core holds theming code, never a brand |

### Step 2: build it

- **How a change lands**
  ([core decision 0013](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0013-app-and-core.md)):
  while no app has a public release, a push to `main` is allowed once the gate passes; a pull
  request is optional and its author merges it when its checks pass.
- **App-only change:** one change in the app.
- **Core change an app needs:**
  1. Open a core issue describing the general need, not the app's feature.
  2. Check out both side by side. In the app: `cargo xtask dev-link ../open-source-core`
     (`cargo xtask dev-unlink` removes it). The link changes no file Git tracks: it is a folder
     Git ignores, `.cargo/dev-link/`, and `Cargo.lock` is left alone. While linked, builds,
     tests and the editor use the local core, and `cargo xtask ci --linked` runs the gate
     against it. The plain gate refuses to run linked, because CI builds the pinned release.
  3. Build the general part in the core with its own tests and a use in `apps/core-testbed`, so it
     is proven without any product.
  4. Build the app's use of it in the app.
  5. To have CI check the app's half before the core is released, give the app's branch a
     temporary pin. Push the core change (a branch will do). On the app's branch, on every
     `jcj-` line, write `rev = "<that core commit, all 40 characters>"` in place of the tag; then
     `cargo metadata --format-version 1 >/dev/null` and `cargo xtask sync`. The gate passes on
     that branch and in its pull request, with a warning. It fails on `main`, on a tag and in a
     release, so the pull request is not merged while it holds the pin.
  6. Land the core change first, with the core's gate green. Then release the core.
  7. Move the app's pin to that release's tag (`cargo xtask sync`), then land the app change,
     which names the core issue it needed.
- **Prototype first, promote later.** If the general shape is not clear yet, build it in the app,
  mark it `// core-candidate: open-source-core#N` and open that core issue, labelled
  `core-candidate`. Move it once it works and a second app wants it. A promotion is finished only
  when the app the code came from uses the core's copy and its own copy is deleted: the core
  issue stays open until then, and the marker on the app's copy names it. Never copy code
  between app repositories.
- **Something one app needs from another app**
  ([core decision 0015](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0015-apps-that-depend-on-apps.md)):
  open an issue in the providing app's repository. The provider's change lands and is tagged
  there first; the consumer then moves its pin. Never edit the provider as part of the consumer's
  change. To try the two together, the consumer names the provider in its `xtask/src/main.rs`
  (`providers`) and links a checkout of it like the core: `cargo xtask dev-link <name>=<folder>`.

### Step 3: let it reach the other apps safely

- Everything in the core is shared by default.
- A change that alters what users see in other apps goes behind an option (a shell setting or a
  Cargo feature) that defaults to the old behavior; each app turns it on when ready.
- Versioning
  ([core decision 0011](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0011-releases-and-versioning.md)):
  while at 0.x a patch release only adds or fixes; a minor release may break and its changelog
  says exactly what each app must do. A breaking change to a public `jcj-` interface says so in
  the title of its commit or pull request, and every app is updated in follow-up changes.
- A core change that breaks another app is fixed in the core, not worked around in the app.
- An app moves to the newest core before new work starts on it and before its own release. Every
  `jcj-` line names one tag. `cargo xtask core-pin` says how far behind an app is, and a
  scheduled check opens an issue when it is more than three releases behind.

### Releasing an app

One contract and one workflow for every app
([core decision 0019](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0019-app-releases.md);
the contract, with what each release file is named and holds, is the core's
[`docs/app-releases.md`](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/app-releases.md)).
The gate runs first and the workflow makes the tag, as for the core.

1. Move the app to the newest core. Write what the release changes under `## Unreleased` of
   `CHANGELOG.md`.
2. `cargo xtask release X.Y.Z` makes the changes of the release commit: the version in
   `Cargo.toml` and `Cargo.lock`, and the changelog heading with the date. Run `cargo xtask ci`,
   commit, push to `main`.
3. `gh workflow run release.yml --ref main`. It checks the release commit, runs the whole gate
   on Linux, macOS and Windows, builds and checks the package of each system and
   `SHA256SUMS.txt`, and only then makes the tag `vX.Y.Z` and publishes. A red job leaves no
   tag: fix the cause on `main` and start it again.
4. To try it without releasing: `gh workflow run release.yml -f dry-run=true`. The same dry run
   starts by itself every week. `cargo xtask package` makes this computer's package in `dist/`.

What a release carries is data in `xtask/src/main.rs` (`package`). The packaging and the
workflow are the core's: an app has no release workflow and no packaging script of its own.

### Things agents must not do

- Name WordCraft or craft-libs types in an app repository. Use the `jcj-` interfaces; if one is
  missing, add it to the core.
- Put a product name, color, logo, or screenplay, manuscript or book concept into the core's
  code, interface text or `template/`.
- Change a core interface from a change made in an app repository.
- Copy code from one app repository into another.
- Release the core, or move an upstream pin, on a commit whose full gate has not passed. A
  person or an agent may do either once it has (core decisions
  [0004](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0004-follow-hold-own.md)
  and
  [0011](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0011-releases-and-versioning.md)).
- Push a `v*` tag, or create or edit a release, by hand: in the core and in an app the release
  workflow makes both, after the gate. A tag cannot be taken back.
- Edit WordCraft or craft-libs code anywhere but our forks' topic branches.
- Edit a file or a marked block the core owns anywhere but in the core.
- Commit personal details, private files or secrets.

### Where each kind of record goes

| Record | Lives in |
| --- | --- |
| Decisions about shared interfaces, upstream, tooling | The core's `docs/decisions/` |
| Decisions about one app's behavior | That app's `docs/decisions/` |
| Pins, carried patches, follow / hold / own | The core's `docs/UPSTREAM.md` |
| What changed for apps in each core release | The core's `CHANGELOG.md`, with "What apps must do" |
| Ideas waiting to be promoted to the core | Core issues labelled `core-candidate` |
| A request to another app | An issue in that app's repository |
| Reviews, audits, working notes | `.private/`, or `docs/history/` once a maintainer publishes one |
<!-- END open-source-core: app-and-core -->

## Wordsmith Book's own rules

Behavior comes from `docs/SPEC.md`; the files are specified in `docs/format.md` and
`docs/design-format.md`. Decisions about this app go in `docs/decisions/`. What is planned is
the [`v1` milestone](https://github.com/jamescjfoundation/wordsmith-book/milestone/1) of this
repository's issues (`ROADMAP.md`): open work is an issue there, never a list in a file. The
build plan, `docs/PLAN.md`, is history: every milestone in it is done, and where its ground
rules (section 4) differ from this file, this file holds. Each piece of work that lands on
`main` adds a row to `docs/progress.md`; a change users will see also goes in `CHANGELOG.md`
(under `## Unreleased`) and `docs/guide/` ([Route C](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-c)).

1. **Naming others** ([project decision 0003](https://github.com/jamescjfoundation/wordsmith/blob/main/project/decisions/0003-naming-other-companies-programs-and-formats.md)).
   Code, docs, commit messages and fixtures may name another company, program, service or file
   format where that is a fact a reader needs: what Book opens or writes, what a store or a
   printer requires, where a rule comes from. Book is never described as replacing, following or
   being inspired by another product. People are never named, not even in technical terms
   ("optimal-fit paragraph breaking").
2. **Our own names and design.** Every interface name, menu label, help text, design and icon is
   our own, never another product's name or its distinctive feature name. Book's names:
   **Designs**, **Chapter openings**, **Scene breaks**, **Editions**, **Store editions**,
   **Preflight**, **House styles**. Print and ebook presets are named by trim size, binding and
   format ("6 × 9 in paperback, cream paper", `pod-paperback`), never by a company.
3. **Sources.** A published standard (W3C, ISO) is cited by its address. A rule taken from a
   service's public file guide names the service, gives the guide's address and the date it was
   checked, and is kept as data in `assets/print/` so it can be corrected without code.
   **Today:** the rules recorded before 2026-11-26 still say "a print-on-demand service's public
   file guide" (`docs/SPEC.md`, `assets/print/`); name the source when a rule is next checked
   against its guide ([issue 14](https://github.com/jamescjfoundation/wordsmith-book/issues/14)).
4. **Fonts.** Bundled fonts have open licenses (OFL-1.1 or the core allowlist) and each is listed
   in `ATTRIBUTION.md`. In the interface and in design files they go by our role names
   (`classic-serif`, `reading-serif`, `display`, `inscription`, `sans`; `docs/design-format.md`), never by
   family: a design names a role. Preflight refuses fonts whose embedding bits forbid embedding.
5. **The author's words are sacred.** Nothing the author wrote is dropped or rewritten by
   importing, building or exporting. Every importer and exporter proves it with a word-for-word
   test. Cleanup happens only as listed changes the user accepted. Light fixes to a linked
   Manuscript project are offers the writer decides on, never edits.
6. **Book never writes a Manuscript project.** Every test that uses a linked source checks the
   project folder's bytes are unchanged.
7. **Golden files are the judge.** Print PDFs (text, page breaks, page boxes) and EPUB structure
   are compared with reviewed reference files. A change that alters them is deliberate, reviewed
   and recorded in `docs/golden.md`. `UPDATE_GOLDEN=1` writes them again, and the guide's page
   of keys with them: the one switch of the three Wordsmith apps.
8. **Fixtures** are public-domain texts or written for the test, made by scripts in `tools/`
   (standard library only) or by `cargo xtask fixtures`, and recorded in `docs/fixtures.md`.
9. **Print stays in `book-typeset`** (decision 0003): it builds a print document in the core's
   model and the core lays it out and writes it. Book's text is only ever the text of runs, never
   instructions; print uses only Book's bundled fonts and fonts added to the project.
10. **The brand comes from the kit.** The logos, icons and palette card in `assets/brand/` are
    files of the Wordsmith brand kit (the wordsmith repository, `brand/`), written here by the
    kit's tool with a record, `assets/brand/kit.json`: the kit commit they were taken at and
    what the folder holds
    ([project decision 0007](https://github.com/jamescjfoundation/wordsmith/blob/main/project/decisions/0007-brand-files-reach-the-apps-by-one-route.md)).
    They are never edited, recolored or redrawn here, and the record is never edited by hand: a
    change is made in the kit and taken again
    ([Route F](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-f)):
    with the wordsmith repository checked out beside this one, `npm run brand -- --app book` in
    its `website/wiki`, and the result committed here, naming the wordsmith commit.
    Book's own files there are three: `tokens.json`, the interface theme in the core's token
    format, whose palette holds the kit's seven colors (decision 0004); `mark-mask.png`, drawn
    from the kit's mark by `tools/brand-mask.py` and drawn again when the mark changes; and
    `README.md`, the kit's guide with rows added for those two and the icon sizes. A new file of
    Book's own in that folder is added to the kit's list (`brand/apps.json`) first.
    `cargo xtask brand`, in the gate, holds the folder to the record: each kit file by its
    checksum, no file the record does not name, and the theme's palette. `cargo xtask brand
    --kit` asks whether the kit has moved since; the workflow `brand-kit.yml` runs it every
    week and is red until the files are taken again (`xtask/src/brand.rs`).

11. **The command line and the agent tools follow the core's convention**
    ([core decision 0017](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0017-command-lines-and-agent-tools.md),
    the core's `docs/command-lines-and-agent-tools.md`): exit codes 0 done, 1 failed, 2 usage,
    3 a check of the content did not pass; results on standard output and
    `wordsmith-book-cli: <reason>` on standard error; `mcp [<file>] [--read-root <folder>] [--write-root <folder>]`; every
    tool named `book_<command>`, served through the core's adapter. A test runs the built
    program to hold it there (`apps/book-cli/tests/command_line.rs`). A command that takes a
    path names the parameter in `files_of` (`crates/book-ui/src/agents.rs`), and one that reads
    or writes somewhere no parameter names is checked in `BookTools`: otherwise an agent kept
    to folders is not held to them for that path.

12. **Releases, names and versions** (decision 0005). A release is cut by the steps of
    "Releasing an app" above; this repository has no release workflow and no packaging of its
    own. What a release carries is `package` in `xtask/src/main.rs`. Two names there are also
    in the program, and a test on each side holds them (`apps/book/src/main.rs`): the release
    id, `wordsmith-book`, which is the desktop program, the stem of every release file, the id
    of the update check's source and the settings folder; and the app id,
    `org.jcjfoundation.book`, which the window is given. Every other program is named
    `wordsmith-book-<name>`. The desktop program hands its arguments to the updater first of
    all, closes its window when the shell asks, and tells the shell last that it ends: drop one
    of the three and an update is installed and never started. The first public release is
    0.9.0 and 1.0.0 follows; the stages 0.1 to 1.0 of the plan were never released, and
    `CHANGELOG.md` keeps them under "Before the first release". The gate runs on macOS and
    Windows only in the release workflow, so a test must not assume a Unix path, a line end, a
    file permission or a font of the system (`[cfg(unix)]` the whole test file when its
    stand-ins exist only there).

### Book and Wordsmith Manuscript

Book builds on Manuscript's `wordsmith-manuscript-model` and `wordsmith-manuscript-format`
crates, as git dependencies at a `book-view-1.N` tag (decision 0001; `Cargo.toml` names the
current one). The contract is Manuscript's
[`docs/book-handoff.md`](https://github.com/jamescjfoundation/wordsmith-manuscript/blob/main/docs/book-handoff.md),
and Manuscript owns it.

- **What Book uses is listed in the contract, by name.** The contract is the list of what
  Book's code really names in the two crates (types, functions, constants), with the project
  format and the packed format. Book names nothing of Manuscript's that the list lacks. Code
  here that needs one more item asks for it first (next point) and uses it once the contract
  names it and a tag carries it.
- **A need goes in as an issue in the Manuscript repository**, as the block above says
  ([Route B](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-b)).
  Anyone working here, an agent included, may open it. `docs/requests-to-manuscript.md` is
  Book's index of them, with what Book does meanwhile; the requests for something new in the
  book view are Manuscript issues #9, #10, #11 and #13. A Book change never edits the
  Manuscript repository.
- **Book moves to a new `book-view` tag only when Manuscript has cut it.** `main` never names
  a branch or a commit of Manuscript's. A change in Manuscript that Book must follow arrives
  as a new tag and, where Book has work to do, an issue here.
- **Manuscript's CI builds Book.** A job on Manuscript's side builds Book against Manuscript's
  branch, so a change there that breaks Book fails before it lands. For that to say anything,
  Book's `main` must stay buildable against Manuscript's newest `book-view` tag: move to a new
  tag soon after it is cut, and never leave `main` needing something only an older tag has.
- **The versions Book reads are stated in one place**: `MANUSCRIPT_SCHEMAS` (the project
  formats, the `schema` of a project's `project.toml`) and `BOOK_VIEWS` (the book view's
  versions) in `crates/book-format/src/lib.rs`. Every source is held to both when it is opened,
  folder or packed, linked or owned, and one outside them is refused in Book's own words, naming
  the version found and the versions Book reads. A test keeps the newest format equal to the one
  the pinned crates read, so a tag with a new format fails it: raise the range on purpose, once
  Book has been checked against that format, with a line in `CHANGELOG.md` and in the guide
  (`docs/guide/manuscript.md`, "Versions").
- **One core.** Book and the Manuscript tag it names pin the same core release, and
  `cargo xtask core-pin` fails when `Cargo.lock` holds two. So Book moves to a newer core only
  after Manuscript has moved and cut the next `book-view-1.N` tag on it; then both pins move in
  one change here ([Route I](https://github.com/jamescjfoundation/wordsmith/blob/main/project/README.md#route-i)).
  Until that tag exists, Book can still take the core's files and marked blocks:
  `cargo xtask sync <this folder>`, run in a checkout of the core at that release, writes them,
  and Book's own `cargo xtask sync --check` passes once its pin has moved.

The list is the contract's "What Book may use". Manuscript's test
`crates/manuscript-format/tests/book_contract.rs` uses each item the way Book does, and its
workflow "Book consumer" runs Book's whole gate against a change to the two crates.

**Today:** nothing here compares Book's code with the list ([issue 24](https://github.com/jamescjfoundation/wordsmith-book/issues/24)). An
item Book starts to use without asking is found only when a change in Manuscript breaks it, so
check a new `wordsmith_manuscript_` name against the list by hand.

### Where each crate's code goes

<!-- BEGIN layer table (cargo xtask app-layers reads it) -->
| Crate | Layer | Holds | May depend on |
| --- | --- | --- | --- |
| `book-model` | 1 | Editions, design choices, overrides and fixes keyed by ids, metadata additions, cleanup decisions; no files, no interface | `wordsmith-manuscript-model` (types only), `jcj-doc`, `serde` |
| `book-format`, `book-design` | 2 | The project folder, the source, offers; the design schema, built-in designs, mixing | the model, `wordsmith-manuscript-format`, `jcj-project` |
| `book-import`, `book-epub` | 3 | Imports and structure detection; the ebook writer | layers 1–2, `jcj-interchange`, `jcj-layout`, `jcj-output`, `jcj-proof`, `jcj-features` |
| `book-typeset` | 3 | Print typesetting on the core's layout (decision 0003) | the same; and, should an embedded typesetting engine ever be added, it stays in this crate: `typst` (only here), `typst-pdf` (only here), `typst-render` (only here) |
| `book-check` | 4 | Preflight, print profiles, external validators | layers 1–3 |
| `book-ui` | 5 | Views, commands, ribbon tabs, panels, agent tools | everything below, `jcj-engine`, `jcj-shell`, `jcj-theme`, `jcj-agents`, `eframe`, `egui_kittest` |
| `apps/*` | 6 | The desktop app, the command line | the crates above |
<!-- END layer table -->

`book-typeset`, `book-epub` and `book-check` never depend on interface code, so the command line
builds books with no window. `cargo xtask app-layers` checks this, and reads the table above to
do it: the table is the rule, with no second copy in code. A package depends only on packages of
a lower layer; the two rows of layer 3 are one layer. A crate named in backticks in the last
column is used only from its layer up, and one marked `(only here)` only by that row's
packages. A core crate or one of Manuscript's that the table does not name fails; any other
library the table does not name is not checked. Code that a second app may want is
marked `// core-candidate: open-source-core#N` and listed in `docs/core-candidates.md`.
