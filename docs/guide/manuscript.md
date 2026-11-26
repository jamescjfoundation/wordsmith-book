# Working with Wordsmith Manuscript

A Book project can link a Manuscript project (a folder or a `.wsmz` file). Book reads the
manuscript's book view, the sections, their roles and the text, and **never writes to it**.

**When the manuscript changes (FIL-2).** When you come back to Book, it notices that the linked
project changed, reads it again and says which sections changed. Book's settings for a section
stay with it, because sections are matched by their lasting ids.

**Fixes go back to the writer.** A light fix made in Book is offered to the writer, who accepts or
declines it in Manuscript. Until then, Book's builds use it; once accepted, the manuscript has it
and the fix retires; if declined, Book's builds leave it out.

**Moving a book.** Save as One File, or the project bundle on the Export page, carries a packed
copy of the linked manuscript, so the book opens on another machine even where the manuscript is
not.

**Versions.** Book says which Manuscript projects it reads: project formats 1 to 3 (the format
is the `schema` number in the project's `project.toml`), and book view version 1. When a project
is outside that, because a newer Manuscript saved it, Book does not open its text and says so,
naming the project's format and the formats it reads: update Wordsmith Book, and the book opens
again with nothing lost. This holds for a linked project, a packed one and a book's own source.
