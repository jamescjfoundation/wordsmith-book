# Structure: chapters, parts, front and back matter

**The outline** shows the book in reading order: front matter, the body, back matter.
The source's sections keep the writer's order; change that in Manuscript. Book's own elements
(dedication, epigraph, contents, about the author, and so on) move freely by dragging or with
commands, and every change can be undone.

**Chapters and parts.** Numbering is set once for the book: “Chapter One”,
“Chapter 1”, words, figures, roman numerals or none, with your own labels; parts number on
their own. A chapter can be unnumbered, have a subtitle or an epigraph, or appear only in print
or only in the ebook (Chapter Settings).

**Print only or ebook only.** A chapter, an element or a single paragraph can be set to
appear only in print or only in the ebook (say, a reading-group note for the ebook); each output's
word check allows for it.

**Front matter and back matter.** Book adds a half title, title page, copyright
page and contents, filled from the book's details; add a dedication, epigraph, foreword,
preface, introduction, prologue; and in the back an epilogue, afterword, acknowledgements,
about the author, also by, a newsletter invitation, an excerpt, a glossary, notes or reading
group questions. If the source already has one of these (its own title page or copyright page),
Book uses the source's and leaves its own out.

**Placing them.** Each kind goes to its conventional place when you add it (a prologue
last before the text, notes before a glossary, and so on); a prologue and epilogue open like
chapters; a glossary's terms are set in small capitals and questions are numbered; a Notes element
holds the notes when an edition puts them at the end.

**Element templates.** Generated pages are templates with placeholders such as
`{title}`, `{byline}`, `{year}`, `{holder}` and `{isbn_line}`, filled from the metadata; a placeholder Book does
not know is reported by preflight.

**Text elements.** Book prints what the source holds: paragraphs, headings inside
chapters, scene breaks, block quotes, verse with its line breaks, asides, notes (numbered per
chapter and printed at the chapter's end, linked both ways in the ebook), pictures with their
alternative text, and language changes. Raw blocks meant for another format are left out and
counted in the build report.

**Light fixes.** A typo can be fixed in Book without changing a linked manuscript: click
a paragraph and edit it. The fix is offered to the writer in Manuscript; until the writer
accepts it, Book's builds use it and preflight notes it. In a book's own source the fix is made
in the text.

**Box sets.** Several books in one: give the set's books, in order, with the command
`book.boxSet.set` (from the command palette or an agent): each a linked Manuscript project or a
source of the book's own. Each book becomes a part with its own title page and front and back
matter; the set gets one contents listing every book. Every word of every book is checked as
for a single book.
