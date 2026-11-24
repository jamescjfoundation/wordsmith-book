# Wordsmith Book

Wordsmith Book is the J. C. J. Foundation's open-source app for designing and typesetting books.
A writer or a small press brings a finished manuscript, picks a book design, sets the trim size,
fonts, chapter openings and front and back matter, and exports a print-ready PDF for paperback and
hardcover and a valid, accessible EPUB for ebook stores, every word checked against the source.

It is the third app on [open-source-core](https://github.com/jamescjfoundation/open-source-core),
after [Wordsmith Script](https://github.com/jamescjfoundation/wordsmith-script) and
[Wordsmith Manuscript](https://github.com/jamescjfoundation/wordsmith-manuscript). Manuscript is
where a book is written; Book is where it becomes a product.

## Status

Being built from [the plan](docs/PLAN.md); progress is in [docs/progress.md](docs/progress.md).

## Building

```sh
cargo run -p wordsmith-book       # the desktop app
cargo run -p wordsmith-book-cli   # the command line (book-cli)
cargo xtask install-hooks         # pre-commit and pre-push hooks
cargo xtask ci                    # the full gate
```

Rust is pinned in `rust-toolchain.toml`. See [CONTRIBUTING.md](CONTRIBUTING.md) and
[AGENTS.md](AGENTS.md).

## License

MIT OR Apache-2.0, at your option. Built on open-source-core, based on WordCraft by the ArtCraft
team (see [NOTICE](NOTICE)). Bundled fonts and other assets are listed with their own licenses in
[ATTRIBUTION.md](ATTRIBUTION.md).
