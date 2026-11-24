# Attribution

Third-party assets bundled with Wordsmith Book, each with its license. Code dependencies are
listed by `cargo xtask third-party-licenses`; their licenses are checked by `cargo xtask licenses`
and `cargo deny` (open-source-core decision 0001).

This is the only file, besides the fonts' own files under `assets/fonts/`, that names the bundled
fonts; everywhere else they go by role names.

## Our own assets

| Path | What | License |
| --- | --- | --- |
| `assets/brand/book/` | Wordsmith Book's tokens, mark and app icons, drawn by `tools/brand-icons.py` | MIT OR Apache-2.0 (this project) |

## Fonts

| Path | Role | Family | Source | License |
| --- | --- | --- | --- | --- |
| `assets/fonts/classic-serif/` | Classic serif (regular, italic, bold, bold italic) | EB Garamond | https://github.com/octaviopardo/EBGaramond12 (static TTF, fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
