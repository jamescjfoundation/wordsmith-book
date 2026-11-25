# Ebook

**EPUB 3.3 (EBK-1):** one XHTML file per section, one stylesheet made from the design's ebook
choices (sizes in em, so reading apps' own settings work), a package document with the ebook
ISBN or a lasting identifier, and an NCX for older reading systems. The same project gives the
same EPUB every time.

**Navigation (EBK-2):** a table of contents with parts holding their chapters, landmarks (cover,
title page, contents, start of the book, copyright, back matter), and `epub:type` and ARIA roles
on every section.

**Accessibility metadata (EBK-3):** access modes, features, hazards, a summary and conformance,
made from what the ebook holds. Every picture needs alternative text unless it is marked
decorative; export waits until it has some.

**Embedded fonts (EBK-4):** on or off per edition. When on, the design's body font, and its
heading font when the design uses one, go into the ebook, only when their licences allow it.

**Cover (EBK-5):** the ebook edition names its cover image (a PNG or JPEG in the project). Book
declares it, shows it first, and warns when its long side is under 1,600 pixels, it is over 4 MB,
or it is not about 1.6 times as tall as it is wide.

**Notes (EBK-7):** pop-up footnotes where reading systems show them (the default), endnotes at
each chapter's end, or a Notes document at the end of the book, per edition; each note links back
to where it was called.
