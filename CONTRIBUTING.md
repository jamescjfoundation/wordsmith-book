# Contributing

Thank you for helping. This repository is public: everything you commit, push, or write in an
issue, pull request or review can be read by anyone and stays in the history for good.

## Building and checking

```sh
cargo xtask install-hooks        # once: pre-commit and pre-push hooks
cargo xtask ci                   # the full gate (CI runs the same)
cargo xtask ci --changed         # only the packages your change affects, and their dependents
cargo xtask app-layers           # Book's crates depend only on the layers under them
cargo xtask names                # no company, product or bundled font family named
cargo run -p wordsmith-book      # the desktop app
```

`cargo xtask help` lists every task.

## Where things are

| Path | What it is |
| --- | --- |
| `crates/book-model` | Book's own layer: editions, design choices, overrides, fixes, metadata additions |
| `crates/book-format` | The Book project folder, the source and the offers file |
| `crates/book-design` | The design schema and the built-in designs |
| `crates/book-import` | Imports into an owned source, structure detection, cleanup |
| `crates/book-typeset` | Print typesetting through the embedded engine |
| `crates/book-epub` | The ebook writer |
| `crates/book-check` | Preflight |
| `crates/book-ui` | The views |
| `apps/book`, `apps/book-cli` | The desktop app and the command line |
| `assets/` | Brand, fonts, designs and print profiles |
| `docs/` | The plan, spec, formats, decisions, guide and progress |
| `tests/fixtures/`, `tools/` | Fixtures and the scripts that make them |

## Rules for a public repository

- Never commit anyone's name, email address or account (other than your commit author line),
  local paths, files from a real machine, anyone's private writing, or secrets. `cargo xtask
  privacy` checks files and commit identities.
- Never name another company or commercial product (AGENTS.md, "Wordsmith Book's own rules").
- Security problems: follow `SECURITY.md`.
- Private working notes go in `.private/` (ignored by Git).

## Commits and pull requests

- Contributions are licensed as the project is, MIT OR Apache-2.0, with no extra terms. No CLA.
- Use your GitHub no-reply address.
- Commit subjects name the plan step (`M1.1 m1/format: the book project folder`).
- AI-assisted contributions are welcome; you are responsible for every line.

## Conduct

Follow `CODE_OF_CONDUCT.md`.
