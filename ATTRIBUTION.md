# Attribution

Third-party assets bundled with Wordsmith Book, each with its license. Code dependencies are
listed by `cargo xtask third-party-licenses`; their licenses are checked by `cargo xtask licenses`
and `cargo deny` (open-source-core decision 0001).

The bundled fonts are listed here by family. In the interface and in design files they go by
role names (`docs/design-format.md`).

## Our own assets

| Path | What | License |
| --- | --- | --- |
| `assets/print/` | Print profiles: trims, gutters, margins, paper and limits, written by the project | MIT OR Apache-2.0 (this project) |
| `assets/designs/` | The built-in designs, written by the project | MIT OR Apache-2.0 (this project) |
| `assets/brand/tokens.json` | Wordsmith Book's design tokens, in the brand kit's palette | MIT OR Apache-2.0 (this project) |
| `assets/brand/README.md` | How to use the brand assets, from the Wordsmith brand kit | MIT OR Apache-2.0 (this project) |
| `assets/brand/kit.json` | The record the brand kit's tool leaves: the kit commit the files were taken at, and what the folder holds | MIT OR Apache-2.0 (this project) |
| `assets/brand/mark-mask.png` | The mark's letter as a white mask, drawn from `assets/brand/wordsmith-book-mark-ink.svg` by `tools/brand-mask.py` | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/favicon.svg` | The app icon tuned for small sizes, from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/icon-1024.png` | The app icon, 1024 px, from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/icon-512.png` | The app icon, 512 px, from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/icon-256.png` | The app icon, 256 px, from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/icon-32.png` | The app icon, 32 px, from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/icon-macos-1024.png` | The app icon drawn for macOS's icon grid, 1024 px, from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/icon.ico` | The Windows program's icon (16 to 256 px), from the brand kit | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-mark-ink.svg` | The mark (B and ribbon cursor), dark | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-mark-paper.svg` | The mark, light | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-wordmark-ink.svg` | The primary logo, dark | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-wordmark-paper.svg` | The primary logo, light | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-stacked-ink.svg` | The stacked logo, dark | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-stacked-paper.svg` | The stacked logo, light | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-book-palette.svg` | The app's color swatches | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-icon.svg` | The parent brand's icon | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-mark-ink.svg` | The parent brand's mark, dark | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-mark-paper.svg` | The parent brand's mark, light | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-imprint-ink.svg` | The imprint lockup, dark | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-imprint-paper.svg` | The imprint lockup, light | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-family-ink.svg` | The family lockup, dark | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/brand/wordsmith-and-ink-family-paper.svg` | The family lockup, light | Trademark of The James C. J. Foundation; use unmodified (`assets/brand/README.md`) |
| `assets/i18n/` | Interface catalogs (Spanish), written by the project | MIT OR Apache-2.0 (this project) |

## Fonts

| Path | Role | Family | Source | License |
| --- | --- | --- | --- | --- |
| `assets/fonts/classic-serif/` | Classic serif (regular, italic, bold, bold italic) | EB Garamond | https://github.com/octaviopardo/EBGaramond12 (static TTF, fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/reading-serif/` | Reading serif (regular, italic, bold, bold italic) | Source Serif 4 | https://github.com/adobe-fonts/source-serif (release 4.005, static TTF, fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/display/` | Display serif with small capitals (regular, italic, bold, bold italic) | Playfair Display SC | https://github.com/google/fonts/tree/main/ofl/playfairdisplaysc (fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/inscription/` | Small-capital display serif for titles (regular, bold) | Cormorant SC | https://github.com/google/fonts/tree/main/ofl/cormorantsc (fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
| `assets/fonts/sans/` | Sans serif (regular, italic, bold, bold italic) | Fira Sans | https://github.com/google/fonts/tree/main/ofl/firasans (fetched 2026-11-24) | OFL-1.1 (`OFL.txt` beside the files) |
