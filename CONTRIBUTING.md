# Contributing

Thank you for helping. This repository is public: everything you commit, push, or write in an
issue, pull request or review can be read by anyone and stays in the history for good.

## Building and checking

```sh
cargo xtask install-hooks        # once: pre-commit and pre-push hooks
cargo xtask ci                   # the full gate (CI runs the same)
cargo xtask ci --changed         # only the packages your change affects, and their dependents
cargo xtask app-layers           # Book's crates depend only on the layers under them
cargo xtask fixtures             # write the Manuscript project fixture again
cargo xtask screenshots          # draw the README's pictures into docs/images
cargo run -p wordsmith-book      # the desktop app
cargo run -p wordsmith-book-cli -- help    # the command line
cargo xtask brand                # the files in assets/brand/ are the brand kit's (in the gate)
cargo xtask package              # this computer's release package, into dist/
```

`cargo xtask help` lists every task.

## Where things are

| Path | What it is |
| --- | --- |
| `crates/book-model` | Book's own layer: editions, design choices, overrides, fixes, metadata additions |
| `crates/book-format` | The Book project folder, the source, the offers file and the house file |
| `crates/book-design` | The design schema and the built-in designs |
| `crates/book-import` | Imports into an owned source, structure detection, cleanup |
| `crates/book-typeset` | Print typesetting on the core's layout (decision 0003) |
| `crates/book-epub` | The ebook writer |
| `crates/book-check` | Preflight, print profiles, external validators, export and the ONIX record |
| `crates/book-ui` | The views, commands, ribbon tabs, panels and agent tools |
| `apps/book`, `apps/book-cli` | The desktop app and the command line |
| `assets/` | Brand (copies of the Wordsmith brand kit, and Book's theme), fonts, designs, print profiles and interface catalogs |
| `docs/` | The spec, formats, decisions, guide (`docs/guide/`), golden-file and fixture records, progress, and the finished build plan (history) |
| `ROADMAP.md` | Where the plan is: the `v1` milestone of this repository's issues |
| `tests/fixtures/`, `tests/golden/`, `tools/` | Fixtures, the reviewed reference outputs, and the scripts that make fixtures and the brand mask |
| `xtask/` | The gate and the packaging: the core's shared modules; in `main.rs` Book's data, which includes what a release carries, and in `brand.rs` its check of the brand files |

<!-- BEGIN open-source-core: contributing -->
<!-- Owned by open-source-core (template/CONTRIBUTING.md). Change it there, never in a copy: its tooling rewrites the copies. -->
## Rules for a public repository

### Personal information

Never commit, or paste into an issue or pull request: anyone's name, email address or account
(other than your commit author line); local paths, user names, host names or machine details;
files from a real machine (preferences, recovery, auto-save or backup files, logs, desktop
screenshots); anyone's private writing; passwords, tokens or keys. Test fixtures are
public-domain texts or written for the test. The repository's privacy check
(`cargo xtask privacy` where it has `cargo xtask`) checks files and commit identities.

### Security problems

Follow `SECURITY.md`: report privately and keep the problem out of public places until a release
fixes it.

### Private working notes

Reviews, audits and working notes go in `.private/` (ignored by Git). One is published only when
a maintainer says so, under `docs/history/` where the repository has a `docs/` folder, with a
first line saying it is history
([core decision 0012](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0012-working-in-the-open.md)).
Write the public part as a decision record or changelog entry, giving the reason itself rather
than "as discussed".

## Commits, pushes and pull requests

- Every change lands on `main` through a pull request whose required checks pass; GitHub
  refuses a direct push. Its author, a person or an agent, merges it once they pass, and no
  approval is required. Run the repository's gate (`cargo xtask ci` where it has
  `cargo xtask`) before you push the branch
  ([core decision 0013](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0013-app-and-core.md)).
- A release is cut, and in the core an upstream pin is moved, only on a commit whose full gate
  has passed; a person or an agent may do either (core decisions
  [0004](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0004-follow-hold-own.md)
  and
  [0011](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0011-releases-and-versioning.md)).
- Contributions are licensed as the repository's license files say (MIT OR Apache-2.0 for
  code), with no extra terms. No CLA.
- Use your GitHub no-reply address (`git config user.email <id>+<login>@users.noreply.github.com`).
- One concern per commit or pull request; say what changed, why, and how you checked it.
- A pull request opened by Dependabot is merged by anyone with write access once its checks
  pass.
- Update the changelog and the decision records, where the repository keeps them, when behavior
  changes. A core change that apps must act on says so under "What apps must do".
- AI-assisted contributions are welcome; you are responsible for every line.
<!-- END open-source-core: contributing -->

## Wordsmith Book's own rules

`AGENTS.md`, "Wordsmith Book's own rules", has them in full. The ones a change most often meets:

- Another company, program, service or file format may be named where that is a fact a reader
  needs; interface names, designs and font role names stay our own, and people are never named
  ([project decision 0003](https://github.com/jamescjfoundation/wordsmith/blob/main/project/decisions/0003-naming-other-companies-programs-and-formats.md)).
- Nothing the author wrote is dropped or rewritten, and Book never writes a Manuscript project.
- A change that alters a golden file is deliberate and recorded in `docs/golden.md`.
  `UPDATE_GOLDEN=1` writes the golden files, and the guide's page of keys, again.
- Add a row to `docs/progress.md` for each piece of work that lands, with the test count. The
  step ids in older rows and commit subjects (`M1.1 m1/format`) are the finished plan's; a new
  commit's subject says what the change does.
- What Book needs from Wordsmith Manuscript is asked for in an issue in the Manuscript
  repository and listed in `docs/requests-to-manuscript.md`.
- What is planned is an issue under the `v1` milestone, never a list in a file (`ROADMAP.md`).
- A release is cut by the steps of "Releasing an app" in `AGENTS.md`: the core's workflow, from
  the data in `xtask/src/main.rs`. This repository has no release workflow and no packaging of
  its own (decision 0005).

## Conduct

Follow `CODE_OF_CONDUCT.md`.
