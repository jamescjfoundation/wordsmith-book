# Wordsmith Book: production assets

The files needed to use the Wordsmith Book identity, and the Wordsmith & Ink parent brand it belongs to. This folder is meant to work on its own.

Wordsmith Book is a free, open-source writing app for print books and ebooks, from Wordsmith & Ink, a project of The J. C. J. Foundation. Its logo is the name typed in Courier Prime Bold, ending in a cursor: a **Ribbon Green** ribbon bookmark (#2E9473) that drops below the line and ends in a V notch.

**Ink and paper.** File names ending in `-ink` are dark artwork (Iron Gall Ink #1C2340) for light backgrounds. Names ending in `-paper` are light artwork (Page White #FAFAF7) for dark backgrounds. All SVG logos are outlined shapes, so no font needs to be installed.

## Product assets

| File | What it is | Use it for |
| --- | --- | --- |
| `favicon.svg` | The app icon, tuned for small sizes: a paper B and the cursor on an ink tile | Website favicon, browser tabs, small UI icons |
| `icon-1024.png` | The full app icon, 1024 x 1024 px, rounded ink tile | App stores, social avatars, presentations, anywhere a raster icon is needed |
| `icon-512.png`, `icon-256.png`, `icon-32.png` | The full app icon at smaller sizes, from the brand kit | The window's icon (256 px), Linux packages, small raster uses |
| `icon-macos-1024.png` | The app icon drawn for macOS's icon grid, 1024 x 1024 px | The macOS app bundle's icon |
| `icon.ico` | The app icon for Windows, 16 to 256 px in one file, from the brand kit | The Windows program and its file types |
| `mark-mask.png` | The mark's letter B as a white mask, drawn from `wordsmith-book-mark-ink.svg` by `tools/brand-mask.py` | The app paints it in the theme's ink, with the ribbon in Ribbon Green |
| `tokens.json` | The interface colors, type sizes and spacing, in the brand palette | The app's light, dark and extra-dark themes |
| `wordsmith-book-mark-ink.svg` | The mark: the letter B and the cursor, no tile | Avatars, watermarks and tight spaces on light backgrounds |
| `wordsmith-book-mark-paper.svg` | The mark, light version | The same on dark backgrounds |
| `wordsmith-book-stacked-ink.svg` | The name on two lines, cursor after the second | Splash screens, posters, social cards and square or narrow spaces, on light backgrounds |
| `wordsmith-book-stacked-paper.svg` | Stacked logo, light version | The same on dark backgrounds |
| `wordsmith-book-wordmark-ink.svg` | **The primary logo**: the name on one line with the cursor | Headers, website, documents, anywhere there is room, on light backgrounds |
| `wordsmith-book-wordmark-paper.svg` | Primary logo, light version | The same on dark backgrounds |
| `wordsmith-book-palette.svg` | Swatch card of the app's colors with hex and RGB values | Picking colors in design work |

## Parent brand assets (Wordsmith & Ink)

Wordsmith & Ink is the house that makes the Wordsmith apps and may publish books. Its logo is its name typed the same way, with a cursor that lets fall a drop of ink. It is always one color: the apps bring the color, the parent is ink on paper.

| File | What it is | Use it for |
| --- | --- | --- |
| `wordsmith-and-ink-icon.svg` | The parent icon: a paper ampersand and the ink-drop cursor on an ink tile | Organization avatars (GitHub, social), the parent's favicon or app tile |
| `wordsmith-and-ink-family-ink.svg` | The parent wordmark above the three app icons (Script, Manuscript, Book) on a light background | Showing the whole family: website, README headers, slides |
| `wordsmith-and-ink-family-paper.svg` | The family lockup on a dark background | The same on dark pages |
| `wordsmith-and-ink-imprint-ink.svg` | "Wordsmith & Ink" on two centered lines | Title pages, the publisher line on a cover, end cards, on light backgrounds |
| `wordsmith-and-ink-imprint-paper.svg` | Imprint lockup, light version | The same on dark backgrounds |
| `wordsmith-and-ink-mark-ink.svg` | The parent mark: ampersand and ink-drop cursor | Book spines, colophons, watermarks, small credits on light backgrounds |
| `wordsmith-and-ink-mark-paper.svg` | Parent mark, light version | The same on dark backgrounds |

To show that Wordsmith Book belongs to the house, put a line of text under or near it, such as "a Wordsmith & Ink app". Never merge the two logos or add the ink drop to the app's name.

## Rules

- **Clear space:** keep the height of the capital letters empty on every side of any logo.
- **Minimum size:** wordmark 140 px wide on screen or 35 mm in print. Below that, use the mark. Under 48 px, use the icon.
- **Don't** retype the name in another font, change its case or spacing, recolor it, stretch or rotate it, move the cursor, add shadows or outlines, or put the dark logo on a busy or mid-tone photo.
- **Writing the name:** Wordsmith Book, and Book on second mention. Wordsmith & Ink, or Wordsmith and Ink in running text. Never "WordSmith" or all capitals.

## Colors

| Color | Hex | Role |
| --- | --- | --- |
| Iron Gall Ink | #1C2340 | Logos, icon tile, text |
| Page White | #FAFAF7 | Pages, light logos |
| Ribbon Green | #2E9473 (for text: #1F7358) | Wordsmith Book's cursor and small accents only, one per screen |
| Margin Blue | #3B4FA8 | Links, buttons, focus |
| Slate | #5A6080 | Secondary text |
| Night | #12162B | Dark backgrounds |

Fonts: Courier Prime Bold for logos and short headings, Inter for everything else. Both are free under the SIL Open Font License 1.1.

## License

The app's code is open source. The names and logos are trademarks of The J. C. J. Foundation. They may be used unmodified to refer to the project, but not for forks or other products. The full brand guidelines, with every logo version, icon size and palette file, are in the Wordsmith brand kit.
