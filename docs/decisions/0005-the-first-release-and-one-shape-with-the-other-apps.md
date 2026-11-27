# 0005: The first public release is 0.9.0, in one shape with the other Wordsmith apps

Date: 2026-11-27 · Status: accepted

## Context

Book's records spoke of releases nobody could find. The changelog had four sections, 0.1.0 to
1.0.0, each "(unreleased)"; `Cargo.toml` said 0.1.0; the README said 1.0 was prepared; the guide
said it covered version 1.0. The repository had no release and no release tag. The four numbers
were the stages of the build plan (`docs/PLAN.md`), built in one stretch, and the plan still
read as the list of what was to come although every milestone in it was done. What was really
open sat in a list called "Waiting on a person" at the end of the progress log.

The release workflow written for those stages had never run, and would have published a release
short of what its readers need: it named the macOS file in a way the update check does not find,
published no checksum file for the update check to hold a download against, and put no
third-party license texts in any package, because the list of shipped programs named a program
where a package was wanted. It ran the gate on Linux alone, and said publishing waited for an
approval that was never set up.

Meanwhile the three Wordsmith apps had each made the same choices differently, by order of
creation and not by intent: how a release is cut, how the changelog is headed, what the
command-line program is called, which switch rewrites a golden file, where the plan is kept.
Someone who has worked in one app should find the same thing in the same place in the next.

## Decision

**The first public release is 0.9.0**, a preview: a version people can download to try the app,
its downloads and its update check before 1.0.0. **1.0.0 follows it**, when the `v1` milestone
is done. The three Wordsmith apps do the same.

- **The workspace's version is 0.9.0.** 0.1.0, 0.2.0, 0.3.0 and 1.0.0 were stages of the build
  plan, never released. `CHANGELOG.md` keeps their notes under "Before the first release",
  marked as never released: they are the record of what each stage built. Where code comments,
  tests, golden folders and older records say "0.1", "0.2" or "0.3" (`the test of v0.1`,
  `tests/golden/v0.1/`, "from 0.2"), they mean those stages.
- **The changelog has the core's form.** `## Unreleased` is first. When a release is cut the
  section becomes `## <version> (<date>)` and its tag is `v<version>`; only a released version
  has a heading of that kind. The gate's `release-data` check holds the newest section to it.
- **Releases are made by the core's contract and workflow**
  ([core decision 0019](https://github.com/jamescjfoundation/open-source-core/blob/main/docs/decisions/0019-app-releases.md)):
  three files and `SHA256SUMS.txt`, named so that the update check, the launcher and the website
  find them; started by hand on `main`; the gate first, on Linux, macOS and Windows; the tag
  made by the workflow. This repository has no release workflow and no packaging of its own:
  what a release carries is data in `xtask/src/main.rs`. The builds are unsigned until the
  organisation has certificates.
- **Every program is named after the app**: `wordsmith-book` and `wordsmith-book-cli` (it was
  `book-cli`). Two Wordsmith apps installed into one folder then share no name, and the update
  check knows which programs are its own. This amends decision 0002, which named the command
  line `book-cli`.
- **The project format is fixed from this release on.** `docs/format.md` said version 1 would be
  extended in place until the first release tag. From `v0.9.0` a change to the format raises its
  version and brings a migration and a fixture of the old version.
- **What is planned is the `v1` milestone** of this repository's issues. `ROADMAP.md` links to
  it and holds no list. `docs/PLAN.md` says in its first lines that it is history. The
  "Waiting on a person" list of `docs/progress.md` is closed: each item still open is an issue.
  A gap is closed by closing its issue; no file is edited to say so.

Where Book differed from the other two apps by accident, it takes the common form:

| What | Before | Now |
| --- | --- | --- |
| The command-line program | `book-cli` | `wordsmith-book-cli` |
| The switch that rewrites a reviewed file | `BOOK_BLESS=1` | `UPDATE_GOLDEN=1`, for the golden files and the guide's page of keys alike |
| The changelog | a heading for each planned version, "(unreleased)" | `## Unreleased` first, in the core's form |
| The version | 0.1.0 under a changelog at 1.0.0 | 0.9.0 in both |
| What is planned | `docs/PLAN.md` and a list at the end of `docs/progress.md` | The `v1` milestone; `ROADMAP.md` links to it |
| A release | The app's own workflow and `packaging/`, started by a pushed tag | The core's shared workflow, from data |
| The brand files | Copied by hand, compared with nothing | Written by the brand kit's tool, with `assets/brand/kit.json`; `cargo xtask brand` in the gate |
| The guide | No page on installing, updates or files; feature ids in the text | `installing.md` and `files.md`, as the other apps name them; the ids in one index page |

What was already the common form and stays: fixtures and golden files at the root
(`tests/fixtures/`, `tests/golden/`, each listed in `docs/fixtures.md` and `docs/golden.md`);
`docs/guide/keys.md` and `docs/guide/command-line-and-agents.md`; decision records with a date
and a status; the layer table in `AGENTS.md` that `cargo xtask app-layers` reads.

What Book keeps that the others lack: `docs/design-format.md`, because designs are files people
share; `docs/requests-to-manuscript.md`, because Book builds on Wordsmith Manuscript; and
`cargo xtask fixtures`, because one fixture is written through Manuscript's own writer.

## Consequences

- Scripts and agent configurations that name `book-cli`, and habits that type `BOOK_BLESS=1`,
  must change. Nothing was published under the old names, so only people who built from source
  are affected; the changelog says so.
- A release, its tag and its changelog section agree from the first one on, because there is
  nothing earlier to agree with.
- The gate runs on macOS and Windows for the first time at a release. A test that only ever ran
  on Linux can fail there and stop the release before any tag exists; the fix lands on `main`
  and the workflow is started again.
- The plan's sign-offs that never happened (the specification, Book's side of the hand-off
  review, the look at the golden files) are issues in the milestone, not lines in a finished
  plan.

## Revisit when

- The organisation has signing certificates: the guide, the README and the changelog stop
  saying the builds are unsigned, and Windows updates in place.
- The core's template gives apps a `tests/` layout or a changelog rule that differs from this.
