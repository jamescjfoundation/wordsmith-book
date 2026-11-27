# Export

File › Export lists the editions (each paperback as a print PDF, each ebook as an
EPUB) and the project bundle, and the folder they go to. Every edition is built, checked word for
word and preflighted first; if anything blocks export, nothing at all is written and Preflight
says why.

The **project bundle** is the whole project in one `.wsbkz` file: the book, its designs, pictures,
fonts you added, and a packed copy of a linked Manuscript project, so it opens whole on another
machine.

**Reproducible builds.** The same project with the same version of Book gives
byte-identical files: dates come from the project, zip entries are sorted with fixed times, and
nothing is random.

**Proofs.** Below the export, *Write a Proof* makes a print edition's proof: the same
pages with PROOF across each and every line numbered, so readers can mark "page 12, line 7". A
proof is never part of an export and never goes to a printer.

**For distributors.** *Write the ONIX Record* writes the book's details as an ONIX for
Books 3.0 file, the form stores and distributors read: one product for each edition with its
ISBN, form, title, series, contributors, language, page count, description and imprint.

