# Attribution

Third-party assets bundled with Wordsmith Book, each with its license. Code dependencies are
listed by `cargo xtask third-party-licenses`; their licenses are checked by `cargo xtask licenses`
and `cargo deny` (open-source-core decision 0001).

This is the only file, besides the fonts' own files under `assets/fonts/`, that names the bundled
fonts; everywhere else they go by role names.

## Our own assets

| Path | What | License |
| --- | --- | --- |
| `assets/print/` | Print profiles: trims, gutters, margins, paper and limits, written by the project | MIT OR Apache-2.0 (this project) |
| `assets/designs/` | The built-in designs, written by the project | MIT OR Apache-2.0 (this project) |
| `assets/brand/book/` | Wordsmith Book's tokens, mark and app icons, drawn by `tools/brand-icons.py` | MIT OR Apache-2.0 (this project) |
| `assets/i18n/` | Interface catalogs (Spanish), written by the project | MIT OR Apache-2.0 (this project) |

## Fonts

| Path | Role | Family | Source | License |
| --- | --- | --- | --- | --- |
| `assets/fonts/classic-serif/` | Classic serif (regular, italic, bold, bold italic) | EB Garamond | https://github.com/octaviopardo/EBGaramond12 (static TTF, fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/reading-serif/` | Reading serif (regular, italic, bold, bold italic) | Source Serif 4 | https://github.com/adobe-fonts/source-serif (release 4.005, static TTF, fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/display/` | Display serif with small capitals (regular, italic, bold, bold italic) | Playfair Display SC | https://github.com/google/fonts/tree/main/ofl/playfairdisplaysc (fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/inscription/` | Small-capital display serif for titles (regular, bold) | Cormorant SC | https://github.com/google/fonts/tree/main/ofl/cormorantsc (fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/sans/` | Sans serif (regular, italic, bold, bold italic) | Fira Sans | https://github.com/google/fonts/tree/main/ofl/firasans (fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
