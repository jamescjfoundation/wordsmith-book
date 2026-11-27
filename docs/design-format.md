# The design format

Version 1 (`schema = 1`). A design is a small TOML file: Book's built-in designs ship in the app
(`assets/designs/`), the user's own are `designs/<id>.toml` in a project, and a design can be
exported and imported as one file (DSN-4). A design is data, never code: print and the ebook read
its values. Version 1 was extended in place until Book's first release, 0.9.0; from that release
on, a change to the format raises the version and brings a migration (decision 0005, plan
section 6.1). Version 0 is a draft that was never released, kept to test the migration path.

Fonts are named by role, never by family: `classic-serif`, `reading-serif`, `display`,
`inscription`, `sans` (the families are listed in `ATTRIBUTION.md`).

Reading never trusts the file: at most 64 KB; unknown keys are refused (so a misspelt key is not
silently ignored); every value is checked and each problem is reported in plain words. A design
from a newer version is refused with a message naming both versions.

## Every key

Every table and key but `schema`, `id` and `name` may be left out; the defaults are shown.

<!-- example: full -->
```toml
schema = 1
id = "example"
name = "Example"
about = "A quiet design for literary fiction."
genres = ["literary", "classic"]
moods = ["quiet", "traditional"]

[body]
font = "classic-serif"    # a font role
size = 11.0               # points, 7 to 24
leading = 1.3             # line height, 1.0 to 2.0 times the size
indent = 1.2              # first-line indent, ems, 0 to 4
justify = true
hyphenation = "normal"    # off, normal, strict
orphans = 2               # fewest lines left at a page's foot, 1 to 4
widows = 2                # fewest lines carried to a page's head, 1 to 4

[chapter]
font = "classic-serif"
start = "recto"           # recto (next right-hand page) or any
sink = 0.25               # how far down the text block the opening starts, 0 to 0.5
number_size = 11.0
number_case = "small-caps"  # as-written, small-caps, capitals
title_size = 20.0
title_case = "as-written"
align = "center"          # center or left
ornament = "❧"            # a glyph under the title, or "" for none
first_line = "small-caps" # none, small-caps, drop-cap
drop_lines = 3            # a drop cap's height, 2 to 4 lines

[scene_break]
style = "mark"            # mark, or space (a blank line; a mark where it falls at a page edge)
mark = "* * *"

[part]
number_size = 14.0
title_size = 24.0
sink = 0.33

[heads]
style = "author-title"    # author-title, title-chapter, none
folio = "foot"            # foot or head
size = 9.0
small_caps = true

[ebook]
justify = true
indent = 1.2
first_line = "small-caps" # a drop cap shows as small capitals in an ebook
heading_font = true

# Asides by kind (TXT-2): letters, notes, signs, messages, and `default` for any other kind.
# style: inset, italic, framed (rules above and below), centered; font: a role, or "" for the
# body font; size: 0.7 to 1.3 times the body size.
[asides.default]
style = "inset"
size = 0.95
small_caps = false

[asides.letter]
style = "italic"
size = 1.0
small_caps = false

[asides.note]
style = "inset"
size = 0.95
small_caps = false

[asides.sign]
style = "centered"
size = 1.0
small_caps = true

[asides.message]
style = "framed"
font = "sans"
size = 0.9
small_caps = false
```

## The smallest design

<!-- example: smallest -->
```toml
schema = 1
id = "plain"
name = "Plain"
```

## Version 0 (a draft, never released)

Body text was in a `[text]` table. Book reads it and saves it as version 1.

<!-- example: version 0 -->
```toml
schema = 0
id = "old"
name = "Old"

[text]
size = 10.5
```
