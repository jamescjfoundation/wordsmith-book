# Print

**Trims:** eighteen sizes from the print profile: the common paperback sizes from
4.25 × 6.87 in (mass market) to 8.5 × 11 in, and the metric A5, Demy, Royal, B5 and A4.

**Margins:** mirrored, with an inside margin (gutter) that grows with the page count, as
a print-on-demand service's file guide asks: up to 150 pages 0.375 in, then 0.5, 0.625, 0.75
and 0.875 in over 700 pages. Book lays the book out, reads the page count, sets the gutter and
lays it out again until it settles.

**Page rules:** chapters start on a right-hand page or the next page, as the design says;
front matter folios are lower-case roman and the body arabic from 1; blank pages carry no folio
or running head; runs of blank pages stay within the profile.

**Type:** sizes and leading come from the design; nothing prints under 7 pt.

**Paper:** white, cream or colour, each with its thickness per page for the spine width.

**The print PDF:** single pages at trim size, every font embedded, no crop marks,
annotations, links or bookmarks. The same project gives the same PDF every time.

**Bleed and full-page pictures:** turn on bleed for an edition and its pages
are 0.125 in larger outside, top and bottom, with trim and bleed boxes for the printer;
full-page pictures then fill their page to the edge. Without bleed they fill the text block.

**Notes:** footnotes at the foot of the page that calls them (a long one continues on
the next page), or, per edition, a list at each chapter's end or a Notes section at the book's end.

**Line breaks and hyphens.** Book does not hyphenate words by itself. In print a word is
broken at a line's end only at a place you allowed inside it. Such a place comes from two
sources: a soft hyphen in the manuscript (an invisible mark, kept when the
text comes into Book), and your own hints. So justified text is set without a broken word
unless you ask for one, and a line with long words can be loose; preflight lists loose lines by
page. An ebook's reading app hyphenates as it likes, using the places you allowed first.

Your hints are in Details, under *Line breaks*: the words and phrases that must never be split
across two lines (a name with its title, a number with its unit), one a line, and the words that
may break inside, written with a hyphen at each place (`man-u-script`). A word listed there
without a hyphen is never broken, whatever the manuscript holds. Book applies the hints when it
builds the book, in print and in the ebook, without changing your manuscript.

**Hardcover:** a hardcover edition uses the `pod-hardcover` profile: case-laminate sizes,
wider margins and gutter for the hinge, and 75 to 550 pages.

**Print profiles:** `pod-paperback` (a print-on-demand paperback: trims, gutter table,
margins, page counts, paper) and `generic-pdf` (any trim, fonts embedded, no annotations), as
data files that say when they were last checked.

**Cover:** Book makes the paperback's or hardcover's full-wrap cover template (PDF and
PNG) from the page count and paper: the back, spine and front with bleed, safe areas, whether the
spine is wide enough for text, and the barcode area. Check a finished cover (PDF, PNG or JPEG)
against it before sending it to the printer.

**Large print:** in Details, choose Add Large Print. That edition is set in the
large-print variant of your book's design: 16-point type and up, ragged right, no word broken
at a line's end, headings grown to match. Preflight checks that nothing in it is smaller. A long novel in large
print can run past the printer's page limit; preflight says so.

**PDF/A and PDF/UA:** tick *PDF/A and PDF/UA* on a print edition in Details and its PDF
declares PDF/A-2b (for archives) and PDF/UA-1 (accessible). If you have installed a PDF
validator, set its path in Preferences and preflight checks the file against both.
