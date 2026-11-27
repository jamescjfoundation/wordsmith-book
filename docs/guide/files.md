# Files, saving and backups

## A book is a folder

A Book project is a folder whose name ends in `.wsbk`. Everything in it is plain text but the
pictures and fonts, so it can be read, compared and kept in version control.

| In the folder | What it is |
| --- | --- |
| `book.toml` | Book's own layer: where the words come from, the editions, the design, the pages Book makes, each chapter's settings, the details you added, cleanup choices and fixes |
| `designs/` | Your own designs, one file each |
| `source.wsm/` | The book's own source, when you imported a document: a Wordsmith Manuscript project, which Manuscript can open |
| `assets/` | Cover images, pictures added in Book and fonts you added |
| `builds/` | A record of each edition's last build |

A book that links a Manuscript project holds no copy of its words: `book.toml` names the
project, and Book reads it where it is and never writes to it
([Working with Wordsmith Manuscript](manuscript.md)).

To open a book, choose its `book.toml` in the Open dialog, or the `.wsbk` folder where your
system shows it as one document. After the install steps of
[Installing and updating](installing.md), a book also opens from your file manager.

## A book in one file

**Save as One File** (in the command palette, Mod+Shift+P) packs the whole project into one file
ending in `.wsbkz`, for sending or keeping. It holds the folder's files and, for a book that
links a manuscript, a packed copy of that manuscript, so the book opens whole on a computer
where the manuscript is not. The **project bundle** on the Export page is the same file.

Opening a `.wsbkz` file unpacks it into a folder of the same name beside it, and the book is
then that folder. The packed file is left as it was. Book never unpacks over a folder that
already holds something.

## Saving

- **Save** (Mod+S) writes only the files that changed. Each is written whole beside the old one
  and then put in its place, `book.toml` last, so a save that is cut off leaves the book as it
  was before.
- **Book also saves on its own** while you work, as set in File › Preferences.
- **The same book gives the same files.** Nothing in a project depends on the time or the
  computer, so two saves of the same book are byte for byte the same.

## After a crash

While a book has unsaved changes, Book keeps them in a hidden folder inside the project
(`.recovery`). If the app or the computer stops before a save, the book opens as it was last
saved, and Book says "Unsaved work from last time was recovered. Save to keep it." Save, and the
recovered work is the book; close without saving, and it is gone.

## Backups

**Back Up Now** (in the command palette) writes the whole project as one packed file into a
folder beside it, `<the book's name> Backups`, named with the date and time. The newest ten are
kept. A backup opens like any other `.wsbkz` file.

A backup holds what the project holds. For a linked manuscript that is a copy as it was at that
moment; the writer's own project is backed up by Manuscript.

## Other versions of the app

- **A book saved by a newer Wordsmith Book** is not opened: Book says which format the book has
  and which it reads, and changes nothing. Update the app and it opens.
- **A book saved by an older version** opens, and the next save writes it in the current
  format.
- **The manuscript a book links** has its own format. Which Manuscript projects this version
  reads, and what Book says about one it cannot, is under "Versions" in
  [Working with Wordsmith Manuscript](manuscript.md).

The format itself is written down in `docs/format.md` of the app's repository.
