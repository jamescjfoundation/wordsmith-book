# Wordsmith Book: behavior spec

Status: **awaiting a person's review.** Version 1, 2026-11-24.

This is what Wordsmith Book does, feature by feature (plan section 7 ids). It is built from
published standards, general typographic practice and our own design. Every statement is marked:

- **[Verified]**: two or more public sources agree;
- **[Sourced]**: one public source;
- **[Designed]**: our own design, where sources are silent or leave a choice.

Published standards are cited by address. Rules from a print or ebook service's public file
guide are recorded generically ("a print-on-demand service's public file guide"); their
addresses are kept privately for a person to recheck, and every such rule is data in
`assets/print/` so it can be corrected without code. No company or product is named here.

Sources used:

- [EPUB 3.3](https://www.w3.org/TR/epub-33/) and [EPUB Reading Systems 3.3](https://www.w3.org/TR/epub-rs-33/) (W3C).
- [EPUB Accessibility 1.1](https://www.w3.org/TR/epub-a11y-11/) and [its techniques](https://www.w3.org/TR/epub-a11y-tech-11/) (W3C).
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) (W3C).
- [EPUB 3 Structural Semantics Vocabulary](https://www.w3.org/TR/epub-ssv-11/) (W3C).
- [Schema.org accessibility properties](https://www.w3.org/2021/a11y-discov-vocab/latest/) (W3C community group vocabulary).
- PDF/UA-1, [ISO 14289-1](https://www.iso.org/standard/64599.html); PDF/A-2, [ISO 19005-2](https://www.iso.org/standard/50655.html); PDF 2.0, [ISO 32000-2](https://www.iso.org/standard/75839.html).
- The ISBN, [ISO 2108](https://www.iso.org/standard/65483.html).
- [Unicode Line Breaking Algorithm, UAX #14](https://www.unicode.org/reports/tr14/).
- Public file guides of print-on-demand services and of online ebook stores, recorded from their
  published rules as known on 2026-11-24. Each such rule is marked [Sourced], kept as data, and
  waits for a person to recheck it against the current guide.
- General typographic practice, as described in common typography references and visible in
  professionally made trade books.

## Feature index (0.1)

| Feature | Section | Feature | Section | Feature | Section |
| --- | --- | --- | --- | --- | --- |
| IMP-1 | 1.1, 11 | STR-4 | 2.3 | PRT-6 | 5.10 |
| IMP-2 | 1.2 | STR-5 | 2.4 | EBK-1 | 6.1, 6.2 |
| IMP-3 | 1.2 | STR-6 | 2.4 | EBK-2 | 6.3, 6.4 |
| IMP-4 | 1.3 | STR-7 | 2.5, 11 | EBK-3 | 6.5, 6.6 |
| IMP-5 | 1.4 | TXT-1 | 3 | EBK-4 | 6.9 |
| IMP-6 | 1.5 | DSN-1 to DSN-5 | 4 | EBK-5 | 6.10 |
| IMP-7 | 1.6 | PRT-1 | 5.1 | EBK-8 | 6.11 |
| STR-1 | 2.1 | PRT-2 | 5.2–5.4 | PRV-1 | 7 |
| STR-2 | 2.2 | PRT-3 | 5.5, 5.6 | PRE-1 | 8.1–8.6 |
| STR-3 | 2.2 | PRT-4 | 5.7 | PRE-2 | 8.7 |
| FIL-1 | 12.4 | PRT-5 | 5.8, 5.9 | PRE-4 | 5.11 |
| FIL-2 | 1.1.4 | EXP-1 | 9.1, 9.4 | META-1 | 10.1 |
| AUT-1 | 12.2 | EXP-2 | 9.2, 9.3 | META-2 | 10.2–10.4 |
| AUT-2 | 12.1 | ACC-1 | 12.3 | | |

## 1. Import and cleanup (IMP-1 to IMP-7)

### 1.1 Sources

1. A book has exactly one source: a **linked** Manuscript project (folder or packed) or an
   **owned** source inside the Book project. [Designed]
2. A linked project is read through Manuscript's book view, version 1 only; any other version is
   refused with a message naming the version found and the versions Book reads. [Designed]
3. Book never writes a linked project or anything inside its folder. [Designed]
4. A linked project changed on disk is read again when Book notices it (on focus and on open),
   and the changed sections are listed word by word against the last build (FIL-2). [Designed]

### 1.2 Imports (IMP-2, IMP-3)

1. DOCX, ODT, RTF, Markdown and plain text are read through the core's readers and converted into
   an owned source. [Designed]
2. A DOCX downloaded from an online word processor is read like any DOCX; its inline formatting
   (fonts and sizes instead of styles, bold lines instead of headings) is handled by structure
   detection. [Designed]
3. Plain text is split into paragraphs at blank lines (a single line break inside a paragraph is
   kept as a space unless the text is verse-like, see 3.6) and into chapters by the chapter
   patterns of 1.3. [Designed]

### 1.3 Structure detection (IMP-4)

1. Headings with a heading style (level 1, 2) become chapters, or parts when a higher level also
   occurs above chapters. [Designed]
2. A paragraph that matches a chapter pattern becomes a chapter heading: "Chapter" followed by a
   number in figures, words or roman numerals, optionally followed by a separator and a title;
   a number alone on its line ("12", "XII"); a line of only capitals under 60 characters that is
   preceded by a page break or two empty paragraphs. "Part" followed by a number is a part.
   [Designed]
3. A paragraph made only of symbols (`*`, `#`, `~`, `•`, `⁂`, dashes), with or without spaces,
   and nothing else, is a scene break. Empty paragraph runs of two or more inside a chapter are a
   scene break when the source has no symbol breaks. [Designed]
4. Before the first chapter, headings or paragraphs titled "Dedication", "Epigraph",
   "Contents", "Copyright", "Prologue" and similar become front matter of that kind; after the
   last chapter, "Acknowledgements", "About the Author", "Also by", "Afterword" and similar become
   back matter. [Designed]
5. Every rule can be switched off or changed in the import review, and the choices are kept in
   `book.toml` so a re-import uses them. [Designed]

### 1.4 Formatting kept and dropped (IMP-5)

1. Kept: italic, bold, small caps, superscript and subscript where the source marks them,
   strikethrough, language spans, links, notes, images with their alternative text, block
   quotes, lists (as paragraphs with their markers kept as text), centered or right alignment on
   short paragraphs (kept as a centered or right-aligned paragraph only for verse, epigraphs and
   signatures). [Designed]
2. Underline is offered as italic (underline stood in for italic on typewriters). [Sourced]
3. Dropped and listed in the report: fonts, sizes, colors, highlights, manual spacing before and
   after paragraphs, first-line indents made of tabs or spaces, page breaks inside chapters,
   empty paragraphs, manual line breaks at the ends of paragraphs. Nothing with words is ever
   dropped. [Designed]

### 1.5 Cleanup (IMP-6)

1. Proposed changes, each listed with its section, paragraph and the words around it:
   straight quotes to curly quotes by the book's language; `--` to an em dash (`—`), a spaced
   hyphen between words to a spaced en dash only when the book's style says so; `...` and
   `. . .` to an ellipsis (`…`); two or more spaces to one; spaces at the ends of paragraphs
   removed; scene-break marks of different kinds made one. [Designed]
2. Cleanup never changes a letter, a digit or the order of words. [Designed]
3. Each change is accepted or undone on its own, or all at once; "Keep exactly as written"
   proposes nothing. [Designed]
4. English double quotes open after a space, a line start or an opening bracket and close
   otherwise; an apostrophe inside or at the end of a word is a right single quote (U+2019).
   [Verified]

### 1.6 The word-for-word check (IMP-7)

1. The words of the source and of the owned source written from it are compared in reading
   order; words are runs of letters, digits and joining apostrophes and hyphens. Typographic
   variants that cleanup may change (quote and dash forms) compare equal only when the user
   accepted that cleanup. [Designed]
2. An import whose words differ is refused, showing the first difference with ten words of
   context on each side. [Designed]

## 2. Structure and front and back matter (STR-1 to STR-7)

### 2.1 Outline (STR-1)

1. The outline shows front matter, parts, chapters (and scenes under them) and back matter in
   reading order. [Designed]
2. Book's own elements (generated front and back matter) reorder by drag; a linked source's own
   sections show in the writer's order, read-only, with "Change this in Manuscript". [Designed]

### 2.2 Chapters and parts (STR-2, STR-3)

1. Chapter numbers can be on or off; their style is one of: words with a label ("Chapter One"),
   words ("One"), figures ("1"), roman ("I"), or label with figures ("Chapter 1"). [Designed]
2. Numbers in words are written in English up to one hundred and twenty and beyond ("One
   Hundred Twenty-One"), with a hyphen between tens and units. [Sourced]
3. Roman numerals use the standard subtractive form (4 = IV, 9 = IX, 40 = XL, 90 = XC); front
   matter folios are lower-case roman. [Verified]
4. A chapter has a title, an optional subtitle and an optional epigraph (quotation and
   attribution). [Designed]
5. Parts are numbered on their own, independently of chapters; each part has a part page with
   its number and title. [Designed]

### 2.3 Front matter (STR-4)

The order of front matter in a print book is: half title, (also by), title page, copyright page,
dedication, epigraph, contents, then the body. [Verified] An ebook puts the copyright page and
"also by" at the back unless the edition says otherwise. [Designed]

1. **Half title**: the title only. [Verified]
2. **Title page**: title, subtitle, byline (the pen name, else the author), imprint name.
   [Verified]
3. **Copyright page**: the copyright line ("Copyright © 2026 Name"), the rights statement, the
   edition's ISBN with its format ("ISBN 978-…  (paperback)"), the imprint and its address, an
   edition line, and the "printed in" line for print. Generated from metadata; editable.
   [Verified]
4. **Dedication** and **epigraph**: the author's text, set apart and centered by the design.
   [Designed]
5. **Contents**: parts and chapters with page numbers in print, links in the ebook. A short
   novel may leave the printed contents out; the ebook always has its navigation. [Designed]
6. A linked source's own front matter is used as it is; Book's generated element of the same kind
   is not added beside it (a source's own copyright page is never duplicated). [Designed]

### 2.4 Back matter (STR-5, STR-6)

1. **About the author**, **also by** and **acknowledgements**, generated from metadata where they
   can be and editable. [Designed]
2. "Also by" lists the author's other books from the metadata, grouped by series. [Designed]
3. Element templates use placeholders (`{title}`, `{byline}`, `{year}`, `{holder}`, `{isbn}`,
   `{imprint}`) filled from metadata; an unknown placeholder is shown as written and reported.
   [Designed]

### 2.5 Light fixes (STR-7)

1. A fix replaces exact words within one paragraph of one section; the words must occur exactly
   once there. [Designed, following Manuscript's offers contract]
2. Owned source: applied as an undoable edit, recorded in the import report. Linked source:
   written as an offer beside the Manuscript project and applied in Book's builds while waiting.
   [Designed]
3. Decision states: accepted (and found in the text) → the fix retires; rejected → removed from
   builds and the user told; waiting → listed by preflight; stale (the words are no longer there
   exactly once) → shown with the new text. Never a silent overwrite. [Designed]

## 3. Text elements (TXT-1)

1. **Body paragraphs** are justified with a first-line indent of about 1–1.5 em; paragraphs are
   separated by the indent, not by space. [Verified]
2. **The first paragraph** of a chapter and after a scene break has no indent. [Verified]
3. **Drop caps** span two to four lines; the first word or line after a drop cap or at a chapter
   start may be set in small caps (a small-caps lead-in). [Verified]
4. **Block quotes** are indented on both sides or set smaller, without quotation marks added.
   [Verified]
5. **Verse** keeps its line breaks; a line that does not fit wraps with a hanging indent.
   [Verified]
6. **Subheadings** inside chapters are set smaller than chapter titles, never at the foot of a
   page (kept with the next lines). [Verified]
7. **Lists** keep their markers as text. [Designed]
8. **Links** are live in the ebook; in print the link text is printed and the address is left
   out unless the edition says to print addresses. [Designed]
9. **Images** (inline, wide, full page) carry alternative text; an image without alternative text
   blocks ebook export unless marked decorative. [Sourced: EPUB Accessibility 1.1 via WCAG 2.2
   1.1.1]
10. Real small caps come from the font's small-caps glyphs; a font without them is not offered
    for small caps. [Designed]
11. Typographic characters: curly quotes, em and en dashes, the ellipsis character, non-breaking
    spaces where the language requires them. [Verified]

## 4. Designs and chapter openings (DSN-1 to DSN-5)

1. A design is a versioned TOML file holding body and heading fonts by role, sizes, leading,
   indents, justification and hyphenation strictness, widow and orphan rules, the chapter opening,
   scene-break style, part page, running heads and folios, front and back matter styles, and the
   ebook mapping. [Designed]
2. **Body type** for trade fiction is 10–12 pt with 1.2–1.45 leading. [Verified]
3. **Line length** of 55–75 characters is preferred for continuous reading. [Verified]
4. **Hyphenation**: words of six or more letters, at least two letters before the break and three
   after, no more than two consecutive hyphenated lines (strict) or three (normal). [Verified]
5. **Widows and orphans**: a paragraph's first line never ends a page alone and its last line
   never starts a page alone; a page may run one line short to avoid them. [Verified]
6. **Chapter openings**: start on a new page (recto in print when the design says so), sunk by a
   design-set amount (commonly one quarter to one third of the page), with the number, title,
   optional subtitle and ornament, and the first-line treatment (drop cap, small-caps lead-in or
   none). [Verified]
7. **Scene breaks**: an ornament, a glyph row, or a blank line followed by a small-caps lead-in;
   a scene break that falls at the foot or head of a page always shows its ornament so the break
   is not lost. [Verified]
8. **Running heads**: author on the verso and title on the recto, or title and chapter title, or
   none; never on chapter openings, part pages or blank pages; folios at head or foot.
   [Verified]
9. **Mixing** (DSN-3): the chapter opening, body, scene breaks and running heads of a design may
   each come from another design; the result must validate. [Designed]
10. **Own designs** (DSN-4): a changed design is saved under the user's name in the project and
    can be exported and imported as one `.toml` file. [Designed]
11. **The chooser** (DSN-5) filters by genre (literary, romance, thriller, fantasy, non-fiction,
    classic) and mood, and previews the book's own first chapter. [Designed]
12. **Built-in designs at 0.1**: literary, romance, thriller, fantasy, non-fiction, classic,
    each following the rules of this section; their own choices are listed in section 4.1.
    [Designed]

### 4.1 The built-in designs (DSN-2)

Six designs at 0.1, written by the project (`assets/designs/`). All follow the rules of section 4:
justified body text with a first-line indent and no space between paragraphs, the first paragraph
of a chapter and after a scene break not indented, two lines at least on either side of a page
break, hyphenation by the book's language, running heads off on openings and blank pages.
[Designed]

| Design | Body | Openings | Scene breaks | Heads and folios |
| --- | --- | --- | --- | --- |
| Classic | Classic serif 11/14.5 pt, indent 1.2 em | Right-hand page, sunk a quarter; number in small capitals, title 20 pt, a fleuron; small-capital lead-in | A fleuron | Author left, title right, in small capitals; folio at the foot |
| Literary | Reading serif 10.5/14.7 pt, indent 1 em, strict hyphenation | Right-hand page, sunk a third, set left; plain number and title | A blank line, the next scene starting in small capitals | Title left, chapter right; folio in the head |
| Romance | Classic serif 11/15.2 pt | Right-hand page; display title 26 pt; a fleuron; three-line drop cap | A fleuron | Author and title in small capitals; folio at the foot |
| Thriller | Reading serif 10.5/13.7 pt, indent 1 em | Right-hand page, sunk a fifth, set left; 40 pt sans-serif number, title in capitals | Three asterisks | Title and chapter; folio in the head |
| Fantasy | Classic serif 11.5/15.5 pt | Right-hand page; display capitals for the title, ornaments, three-line drop cap; part titles 30 pt | Three ornaments | Author and title in small capitals; folio at the foot |
| Non-fiction | Reading serif 10.5/14.7 pt, indent 1 em | Any page, sunk a little, set left; sans-serif number and title 22 pt | Three asterisks | Title and chapter; folio in the head |

## 5. Print rules and profiles (PRT-1 to PRT-6, PRE-4)

1. **Trims at 0.1**: 5 × 8 in, 5.25 × 8 in, 5.5 × 8.5 in, 6 × 9 in. [Verified]
2. **Gutter (inside margin)** by page count, from a print-on-demand service's public file guide:
   up to 150 pages 0.375 in; 151–300: 0.5 in; 301–500: 0.625 in; 501–700: 0.75 in; over 700:
   0.875 in. Outside margins at least 0.25 in without bleed and 0.375 in with bleed. The table is
   data (`assets/print/pod-paperback.toml`). [Sourced]
3. The gutter depends on the page count, so a build lays out, reads the count, sets the gutter
   and lays out again until the count is stable, at most three passes. [Designed]
4. **Margins** are mirrored; per-side overrides are allowed down to the profile's minimums.
   [Verified]
5. **Page rules**: chapters start on a right-hand (recto) page or the next page, as the design
   says; front matter folios are lower-case roman, the body arabic from 1 on the first page of the
   first chapter or part; blank pages carry no folio or running head. [Verified]
6. **Blank-page runs**: no more than two consecutive blank pages inside the book and no more
   than ten at the end. [Sourced]
7. **Type**: size and leading presets per trim; nothing printed under 7 pt. [Sourced]
8. **Paper**: white, cream or color, each with a thickness per page used for the spine width:
   white 0.002252 in, cream 0.0025 in, color 0.002347 in per page. [Sourced]
9. **Page counts**: paperback 24 to 828 pages on white paper and 24 to 776 on cream, rounded up
   to an even number. [Sourced]
10. **The print PDF** (PRT-6): single pages at trim size (or trim plus bleed when the book has
    bleed), every font embedded, no crop marks, annotations, links, bookmarks, form fields or
    comments, images at 300 dpi or more at their printed size. [Verified]
11. **Profiles** (PRE-4): `pod-paperback` (the rules above) and `generic-pdf` (any trim, no page
    count limits, fonts embedded, no annotations), as data in `assets/print/`, each recording
    when it was last checked and against what kind of source. [Designed]

## 6. Ebook rules (EBK-1 to EBK-5, EBK-8)

1. **Package** (EBK-1): EPUB 3.3; `mimetype` first and stored uncompressed with
   `application/epub+zip`; `META-INF/container.xml`; a package document with `dc:identifier`
   (the ebook ISBN as `urn:isbn:…`, else a UUID), `dc:title`, `dc:language`,
   `dcterms:modified`, `dc:creator`; manifest and spine; one XHTML file per section; one CSS
   file. [Verified: EPUB 3.3 §3–§5]
2. **NCX** for older reading systems, with the same entries as the navigation document.
   [Sourced: EPUB 3.3 §5.4.1 allows it for compatibility]
3. **Navigation** (EBK-2): a navigation document with a `toc` nav (every part and chapter and
   the main front and back matter), a `landmarks` nav (cover, title page, contents, start of the
   body, copyright and back matter where present) and, from 0.2, a `page-list` nav. [Verified]
   The ebook leaves out the half title (a printed book's page); the contents page is the
   navigation document itself, read where the contents stand. [Designed]
4. **Semantics**: `epub:type` on every front and back matter section (`titlepage`,
   `copyright-page`, `dedication`, `epigraph`, `toc`, `acknowledgments`, `colophon`,
   `bodymatter`, `chapter`, `part`, `footnote`, `endnote`, `noteref`), with the matching ARIA
   roles (`doc-chapter`, `doc-dedication`, …). [Verified]
5. **Accessibility metadata** (EBK-3): `schema:accessMode` (`textual`, plus `visual` when there
   are images), `schema:accessModeSufficient`, `schema:accessibilityFeature` (`structuralNavigation`,
   `tableOfContents`, `readingOrder`, `alternativeText` when there are images, `pageNavigation`
   when there is a page list), `schema:accessibilityHazard` (`none`),
   `schema:accessibilitySummary`, and `dcterms:conformsTo` "EPUB Accessibility 1.1 - WCAG 2.2
   Level AA" with `a11y:certifiedBy` set to the publisher. [Verified: EPUB Accessibility 1.1 §2]
6. **Alternative text** is required for every image not marked decorative; export is blocked
   without it. [Sourced: WCAG 2.2 1.1.1]
7. **Language**: `xml:lang` and `lang` on every document; language spans carried. [Verified]
8. **CSS**: conservative (no fixed sizes for body text, `em` units, no floats except drop caps,
   no positioning), so reading systems' own settings work. [Designed]
9. **Embedded fonts** (EBK-4): on or off per edition; when on, only fonts whose license allows
   embedding, obfuscation not used. [Designed]
10. **Cover** (EBK-5): an image of at least 1600 px on its long side and no more than 4 MB,
    ratio about 1.6:1, declared with `properties="cover-image"`, and a cover XHTML page.
    [Sourced: an online ebook store's public file guide]
11. **Ebook preview** (EBK-8): Book draws the chapter in the ebook mapping of the design at phone,
    e-ink reader and tablet sizes with text size and dark-page toggles, and labels it a preview.
    "Open in a reading app" hands the exported EPUB to the reading app the user installed.
    [Designed]

## 7. Preview (PRV-1)

1. Print spreads (verso left, recto right, the first page alone on the right) drawn from the
   engine's page images, with trim, margin and bleed guides that can be turned off. [Designed]
2. Clicking a page opens its section; a section's "show in preview" scrolls to its first page.
   [Designed]
3. Builds run in the background; the interface never waits on one, and only pages in view are
   drawn. [Designed]

## 8. Preflight (PRE-1, PRE-2)

1. Preflight runs on the built files, not on settings: it reads the PDF and the EPUB back.
   [Designed]
2. Each finding has a severity (**blocks export**, **warning**, **note**), a plain message and a
   link to its page or section. [Designed]
3. Print checks: margins against the profile for the page count; every font embedded and
   licensed for embedding (fonts whose embedding bits forbid it block export); nothing under the
   minimum type size; images at 300 dpi or more; blank-page runs within limits; page count within
   the profile; no annotations; the words of the PDF match the source letter by letter. [Designed,
   from section 5] The smallest type is read from the laid-out text; superscript note
   numbers are not counted. Pages are blank when they print no letters. [Designed]
4. Ebook checks: container, package, manifest and spine, navigation targets, ids unique, links
   resolve, XHTML well-formed, CSS within the supported subset, alternative text present,
   accessibility metadata present and consistent with the content, the words of the EPUB match
   the source. [Designed, from section 6]
5. Metadata: what `book::missing_metadata` reports, plus an ISBN per edition that has one, with a
   valid checksum. [Designed]
6. Fixes waiting on the writer are listed as notes ("1 fix not yet accepted by the writer").
   [Designed]
7. **External validators** (PRE-2): the standard EPUB validator, when the user has installed it
   and set its path in the preferences, runs as a separate program with a time limit; its errors
   become findings. "Not installed" is a note, never a failure. [Designed]

## 9. Export (EXP-1, EXP-2)

1. The export page lists the editions and formats (print PDF, EPUB, project bundle) and where
   they go. [Designed]
2. The words are checked before any file is written; a failed check writes nothing. [Designed]
3. The same project with the same app version gives byte-identical files: fixed dates from the
   project, sorted zip entries, fixed times, no random ids. [Designed]
4. The project bundle is a packed Book project with its designs, fonts the user added and
   pictures, and a packed copy of a linked source, so it opens elsewhere whole. [Designed]

## 10. Metadata and editions (META-1, META-2)

1. Book metadata comes from the source (title, subtitle, byline, series and number, language,
   description, keywords, subjects, copyright year and rights) with Book's additions
   (contributors with roles, subject codes, imprint), which are never written to a linked source.
   [Designed]
2. **Editions at 0.1**: paperback and ebook, each with its own ISBN, trim (print), paper (print),
   design overrides and price notes. [Designed]
3. **ISBN-13**: 13 digits, prefix 978 or 979, check digit from weights 1 and 3 alternating, mod
   10. **ISBN-10**: 9 digits and a check character from weights 10 down to 2, mod 11, `X` for
   10. Hyphens and spaces are ignored when checking. [Verified: ISO 2108]
4. Each edition needs its own ISBN; the same ISBN on two editions is an error. [Verified]

## 11. Linked sources and offers (IMP-1, FIL-2, STR-7)

1. Offers are written to `<project folder name>-offers.json` beside the Manuscript folder,
   version 1, appended and never rewritten once written; ids are unique. [Designed, following
   Manuscript's contract]
2. Book reads the writer's decisions from the project's settings through Manuscript's own
   reader. [Designed]
3. Each build records a checksum of the words of each section, so "changed since last export" is
   shown section by section and word by word. [Designed]

## 12. Automation and access (AUT-1, AUT-2, ACC-1, FIL-1)

1. Every command has an id, a label and, where useful, a key; every command is an agent tool
   (AUT-2). [Designed]
2. The command line (AUT-1) builds, checks, verifies, converts and reports with exit codes:
   0 success, 1 a check found blocking problems, 2 bad usage, 3 a file could not be read or
   written. [Designed]
3. Light and dark themes and every command by keyboard come from the shell (ACC-1). [Designed]
4. Auto-save, backups and recovery follow the core's timing (FIL-1). [Designed]
