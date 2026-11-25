# 0004: The brand and the interface colors

Date: 2026-11-25. Status: accepted.

## Context

Wordsmith Book's identity is set in the Wordsmith brand kit (the foundation's `wordsmith`
repository, `brand/wordsmith-book/`): the name typed in Courier Prime Bold ending in a ribbon
cursor, the mark (the letter B and the ribbon), the app icon (a paper B and the ribbon on an ink
tile), and six colors: Iron Gall Ink #1C2340 (logos, text), Page White #FAFAF7 (pages), Ribbon
Green #2E9473 (#1F7358 for text; the cursor and small accents only, one per screen), Margin Blue
#3B4FA8 (links, buttons, focus), Slate #5A6080 (secondary text) and Night #12162B (dark
backgrounds). The build drew a placeholder in M0.4 (an open book in binding red and gilt, in
`assets/brand/book/`), as Script and Manuscript did before their brand arrived.

## Decision

**The assets.** The kit's production files (logos, favicon, palette card, the parent brand's
logos and the README on using them) and its app icons (32 to 1024 px, the macOS tile and the
Windows `.ico`) are copied unchanged into a flat `assets/brand/`, the same layout as the other
two apps. `tools/brand-mask.py` draws `mark-mask.png`, the mark's B as a white mask, from
`wordsmith-book-mark-ink.svg`; the app tints it with the theme's ink and paints the ribbon in
Ribbon Green, as Script paints its mark. `tools/brand-icons.py` and the placeholder are removed.

**The roles.** `assets/brand/tokens.json` names the palette once and the themes use it:

| Role | Light | Dark | Extra dark |
| --- | --- | --- | --- |
| `canvas` | #EEEFF4 | Night | #080A14 |
| `surface` | Page White | #191E36 | #0D1020 |
| `surface_high` | #FFFFFF | #1C2340 | #13172D |
| `text` | Iron Gall Ink | #E9EAF2 | #D6D8E2 |
| `text_dim` | Slate | #9EA3BD | #8F95B0 |
| `accent` | Margin Blue | #9AABF2 | #8392E6 |
| `danger` | #B3261E | #F28B82 | #E8857C |
| `brand` / `on_brand` | Ink / Page White | #2B3466 / Page White | #202752 / #EDEDE8 |
| `highlight` | Ribbon Green | #6CCBA6 | #5DB894 |

The light and dark values follow the kit's tokens (`surface-sunken`, `surface-raised`, `line`,
`ink`, `ink-muted`, `accent`, `danger`); the extra-dark theme and the steps the kit does not name
follow Manuscript's decision 0008, so the three apps look like one family. Ribbon Green is the
one small accent (`highlight`), never a fill for text.

**The programs.** The Windows program embeds `icon.ico` (`apps/book/build.rs`, winresource, MIT;
nothing happens for other targets), so its registered `.wsbkz` type shows the brand; the release
checks every icon image is in the program. The macOS bundle's icon is made from the kit's macOS
tile at every size up to 1024 px; Linux packages carry the 256 px icon.

## Consequences

Tests in `crates/book-ui/src/brand.rs` pin the palette, check WCAG AA for every text and focus
pair in the three themes, and check the icon, the mask and the Windows icon's sizes. The README's
pictures are drawn again (`cargo xtask screenshots`). Print and ebook output never use the brand:
books keep the fonts and colors of their own design.

## Revisit

When the brand kit changes: copy the new files, run `python3 tools/brand-mask.py` if the mark
changed, and keep the tokens in step with the kit's.
