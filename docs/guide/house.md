# A publishing house

A small press keeps its shared details in one **house file** (`house.toml`): its imprints, its
series and the ISBNs it owns. Set its path in Preferences, under *Publishing house*. The file's
form is in `docs/format.md`.

**House styles (DSN-7).** *Join an Imprint or Series* puts the book in an imprint, a series, or
both. The house design comes with them (a series' design over its imprint's). When the imprint or
series locks its design, the book keeps it: choosing, mixing or saving another design is refused
until the book leaves (*Leave the Imprint and Series*). The book remembers the lock, so it holds
even where the house file is not.

**Imprints (META-3).** An imprint in the house file has its name, address, website, logo and its
own copyright wording. A book that joins it takes them: the title page shows the imprint and its
logo, and the copyright page uses the imprint's wording. The book keeps a copy, so it builds the
same anywhere; join again to take changes from the house file.

**Series (META-4).** Join a series as its book 2, say, and the copyright page says "Voyages,
Book 2" and the "Also by" page lists the series' other books in order. *Series View* lists the
series' books with the design each uses and names any that strays from the series' design.

**ISBNs (META-5).** List the ISBNs the house owns in the house file. *Take an ISBN from the
House* (in Details, beside an edition without one) gives the edition the next free ISBN and
writes in the house file which book and edition use it, so it is never given twice. A list with a
mistyped or repeated ISBN is refused until it is put right.

