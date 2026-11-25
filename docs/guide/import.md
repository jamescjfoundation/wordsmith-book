# Importing a manuscript

Book reads a Wordsmith Manuscript project directly (**IMP-1**): link it when you make the book
and Book follows it, never writing to it. Book also imports files into a source of the book's own:
DOCX, ODT and RTF from word processors (**IMP-2**), and Markdown and plain text (**IMP-3**).

**Review before import.** Import a File… shows what Book found before anything is made:

- **Structure (IMP-4):** chapters and parts found from headings, lines such as “CHAPTER IV.” and
  numbered titles, scene breaks from lines of asterisks or ornaments, front and back matter from
  their titles. Each rule can be turned off.
- **Formatting (IMP-5):** what is kept (italic, bold, small capitals, notes, block quotes,
  verse, pictures, language) and what is dropped (fonts, sizes, colours, empty paragraphs, page
  breaks), counted.

**Cleanup (IMP-6).** For a book's own source, the Cleanup panel proposes typographic fixes:
straight quotes to curly ones (with elisions such as ’tis), double hyphens to dashes, three dots
to an ellipsis, and doubled or trailing spaces. Each change shows the words around it; make them
one by one or all at once, and undo them. *Keep exactly as written* proposes nothing. A linked
Manuscript project is never cleaned up.

**Word-for-word check (IMP-7).** Every import is checked word for word against the file it came
from. If a word was lost or changed, Book says which, with the words around it, and nothing is
made.

**EPUB (IMP-9).** An EPUB imports too: Book takes its parts, chapters, front and back matter
and notes from the ebook's own structure, leaves out its contents page (Book makes the contents
again) and lists what it left out in the review.

**A new version (IMP-8).** When the writer sends a revised file, *Import a New Version* compares it
with the book word by word and lists every difference first; taking it keeps the design, the
book's details, front and back matter and each chapter's settings. It can be undone.
