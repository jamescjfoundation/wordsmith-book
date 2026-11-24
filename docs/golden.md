# Golden files

Golden files hold what an output looked like when a person last approved it. A test compares
each new build with its golden file and fails on any difference. When a change is meant to move
the output, a person looks at the new output, then the golden files are written again with
`BOOK_BLESS=1` and committed with the change.

## Print PDFs (M5.4)

`tests/golden/print/<book>-<design>-<profile>.txt`, checked by
`crates/book-check/tests/print_pdf.rs`. Each file is a summary of one print PDF, read back from
the PDF itself:

- the page count, the page size in points, and the gutter used;
- for every page: its number, its folio, the section it belongs to, and either `blank` or its
  line count with the first and last four words it prints (running heads and folios included,
  contents leaders shown as `…`).

So the summaries catch changes to the text, to every page break, and to the page boxes. The same
test checks PRT-6 on every PDF: one page box at trim size (no crop marks or slug), crop box equal
to the media box, every font embedded, no annotations, links or bookmarks (read with the core's
`jcj_output::read_facts`, core v0.1.10).

| Book | Profile | Trim | Designs |
| --- | --- | --- | --- |
| Gulliver's Travels (the Manuscript fixture) | `pod-paperback` | 5.5 × 8.5 in | all six |
| Waverley (imported from its DOCX) | `pod-paperback` | 6 × 9 in | all six |
| Gulliver's Travels | `generic-pdf` | 5 × 8 in | all six |

**State:** written 2026-11-24 from the builds of M5.4; **awaiting a person's look** at the PDFs
they summarise (build one with `cargo test -p wordsmith-book-check --test pages_for_review --
--ignored`, or from the app once export lands in M8).
