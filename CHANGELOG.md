# Changelog

Each release says what changed for people who make books with Wordsmith Book.

## 0.2.0 (unreleased)

Parity for most novels: every common trim, hardcovers, bleed, notes, asides, more front and back
matter, store editions, covers, re-import, typographic checks, an accessibility checker,
line-break hints, a Spanish interface and screen-reader names throughout.

### Added

- **Trims.** Eighteen sizes in the print-on-demand and generic profiles, metric ones included,
  each with its gutter table and page limits.
- **Hardcover.** A hardcover edition on its own profile (`pod-hardcover`): case-laminate sizes,
  wider margins for the hinge, its page limits.
- **Bleed and full-page pictures.** An edition set to bleed gets pages larger than the trim, trim
  and bleed boxes in the PDF (core 0.1.11), and pictures that fill their page to the edge.
- **Notes.** Footnotes at the foot of the page that calls them, continued on the next page when
  long; or notes at each chapter's end or the book's end. In the ebook, pop-up footnotes or
  endnotes per edition, linked both ways. The core's EPUB writer keeps notes too (core 0.1.12).
- **Asides.** Letters, notes, signs and messages styled by the design (inset, italic, framed or
  centred), in print and in the ebook.
- **More front and back matter.** Foreword, preface, introduction, prologue, epilogue,
  afterword, a newsletter or website page, an excerpt from the next book, a glossary, endnotes and
  discussion questions, each placed where convention puts it.
- **Print-only and ebook-only.** Per chapter, per element and per paragraph.
- **Store editions.** Ebook editions named for a store, so links and pages meant for one store
  show only in its edition.
- **Page list.** An ebook edition carries the print edition's page numbers, so readers and
  teachers can cite the same page.
- **Covers.** The full-wrap cover template for a paperback or hardcover (PDF and PNG: trim,
  spine, bleed, safe areas, the barcode area), and a supplied cover checked against it.
- **Re-import.** A new version of the file a book was imported from, compared section by section
  and word by word before it replaces the old text; Book's settings are kept.
- **Typographic warnings.** Widows, orphans, stacked hyphens, loose lines and rivers, by page, in
  preflight.
- **Accessibility checker.** An EPUB accessibility checker you installed runs in preflight beside
  the standard EPUB validator; its findings link to their files.
- **Line-break hints.** Words and phrases never split across lines, and words hyphenated only
  where you say, applied in print and kept in the ebook without changing your manuscript.
- **Interface languages.** English and Spanish, through the core's translation catalogs (core
  0.1.13); the language follows your computer unless you choose one in Preferences.
- **Screen readers.** Every button, field, link, tab and page preview has a name a screen reader
  reads (the core named its last two unnamed fields in 0.1.14).

### Changed

- The ebook's page list names its source with `dc:source`, refined as the source of the
  pagination, which both the standard EPUB validator and accessibility checkers read.
- Book's word checks leave soft hyphens out of words, so hinted words still match the source.

### Known limits

- Command replies (what agents read), preflight findings and build messages are in English.
- The core's layout does not hyphenate on its own; words break only at soft hyphens (from the
  manuscript or from Book's hints).

## 0.1.0 (unreleased)

The first release: a finished manuscript to a print-ready paperback PDF and a valid, accessible
EPUB, every word checked against the source.

### Added

- **Sources.** Link a Wordsmith Manuscript project (read, never written, followed when it
  changes), or import a DOCX, ODT, RTF, Markdown or text file into a source of the book's own,
  with a review of the structure found, the formatting kept and dropped, a cleanup of quotes,
  dashes, ellipses and spaces, and a word-for-word check of every import.
- **Structure.** The outline; chapter and part numbering; a half title, title page, copyright
  page and contents made from the book's details; twenty kinds of front and back matter; Book's
  own settings per chapter (subtitle, epigraph, unnumbered, print or ebook only); light fixes
  offered back to the writer.
- **Designs.** Six designs (Classic, Literary, Romance, Thriller, Fantasy, Non-fiction), parts
  mixed between them, your own designs saved, exported and imported, and a chooser that previews
  your own first chapter.
- **Print.** Four trims, mirrored margins with a gutter by page count, right-hand chapter
  openings, drop caps and small-capital lead-ins, running heads and folios, notes at chapter
  ends, pictures; a print-on-demand paperback profile and a generic one; PDFs with every font
  embedded and no annotations or bookmarks.
- **Ebook.** EPUB 3.3 with navigation, landmarks, an NCX, accessibility metadata, required
  alternative text, embedded fonts per edition and a cover.
- **Previews.** Print spreads with guides, drawn in the background; an ebook preview at phone,
  e-ink reader and tablet sizes with text sizes and a dark page.
- **Preflight and export.** Every rule checked on the built files, each finding linked to its
  page or section; the standard EPUB validator run when installed; export that writes nothing if
  a check fails; a project bundle; the same files from the same project every time.
- **Automation.** `book-cli` (build, check, verify, convert, report, run, commands, mcp) and
  every command as an agent tool.
- The user guide (`docs/guide/`), packages for three desktop platforms with Book's project
  types registered, and update checks from this repository's releases.
