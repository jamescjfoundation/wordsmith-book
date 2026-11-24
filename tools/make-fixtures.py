#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Make Wordsmith Book's document fixtures (standard library only, deterministic).

    python3 tools/make-fixtures.py texts <download dir>   # e-texts -> tests/fixtures/novels/*.txt
    python3 tools/make-fixtures.py documents              # *.txt -> DOCX, ODT, RTF, Markdown

`texts` takes the public-domain e-texts as downloaded (`<number>.txt`, see docs/fixtures.md and
.private/sources.md) and keeps each one's body: the lines between its START and END markers, with
line ends made LF. Nothing else is changed.

`documents` reads those bodies, finds their structure (parts, chapters, notes, verse) by the rules
of each novel below, and writes each novel in the simplest valid form of each format:

- DOCX with Heading 1 for parts and Heading 2 for chapters, italic runs, footnotes as real
  footnotes, verse as one paragraph per stanza with line breaks;
- ODT with the same, notes as `text:note`;
- RTF with heading styles, `\\i` italic and `\\footnote`;
- Markdown with `#` and `##` headings, `*italic*`, `[^n]` notes and verse lines ending in a
  backslash;
- one DOCX shaped like a download from an online word processor (fonts and sizes on every run,
  no heading styles, bold "Chapter N" lines, page breaks, uneven scene-break marks).

Every file is written byte for byte the same on every run (fixed zip times, sorted entries).
"""

import os
import re
import sys
import zipfile

NOVELS = os.path.join("tests", "fixtures", "novels")

# The e-texts: (download name, fixture name).
TEXTS = [("5998.txt", "waverley"), ("829.txt", "gullivers-travels"), ("145.txt", "middlemarch")]

ROMAN = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100}


def roman(s):
    total, prev = 0, 0
    for ch in reversed(s.upper()):
        v = ROMAN.get(ch, 0)
        total = total - v if v < prev else total + v
        prev = max(prev, v)
    return total


# ---------------------------------------------------------------------------------------------
# texts


def cut_body(raw):
    lines = raw.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("*** START OF"))
    end = next(i for i, l in enumerate(lines) if l.startswith("*** END OF"))
    return "\n".join(lines[start + 1 : end]).strip("\n") + "\n"


def make_texts(src):
    os.makedirs(NOVELS, exist_ok=True)
    for download, name in TEXTS:
        with open(os.path.join(src, download), encoding="utf-8") as f:
            body = cut_body(f.read())
        with open(os.path.join(NOVELS, name + ".txt"), "w", encoding="utf-8", newline="\n") as f:
            f.write(body)
        print("wrote", name + ".txt", len(body.split()), "words")


# ---------------------------------------------------------------------------------------------
# structure


def blocks_of(lines):
    """Paragraphs (runs of non-blank lines) as lists of lines."""
    out, cur = [], []
    for l in lines:
        if l.strip():
            cur.append(l.rstrip())
        elif cur:
            out.append(cur)
            cur = []
    if cur:
        out.append(cur)
    return out


def is_verse(par):
    return len(par) >= 2 and max(len(l.strip()) for l in par) <= 60 and any(l.startswith("  ") for l in par)


def para_block(par):
    if is_verse(par):
        return ("verse", [l.strip() for l in par])
    return ("p", " ".join(l.strip() for l in par))


class Book:
    def __init__(self, title, author):
        self.title = title
        self.author = author
        self.front = []  # (title, blocks)
        self.parts = []  # (title, [(title, blocks)])
        self.back = []  # (title, blocks)
        self.notes = {}  # label -> [paragraph text]


def caps_line(s):
    s = s.strip()
    return bool(s) and s.upper() == s and any(c.isalpha() for c in s) and len(s) < 80


def split_sections(lines, heading):
    """Split lines at headings; returns [(heading line index, heading text, following lines)]."""
    idx = [i for i, l in enumerate(lines) if heading(l)]
    out = []
    for n, i in enumerate(idx):
        j = idx[n + 1] if n + 1 < len(idx) else len(lines)
        out.append((lines[i].strip(), lines[i + 1 : j]))
    return idx[0] if idx else len(lines), out


def chapters_of(lines, chapter_re, title_caps):
    first, secs = split_sections(lines, lambda l: re.match(chapter_re, l) is not None)
    chapters = []
    for head, rest in secs:
        pars = blocks_of(rest)
        title = head
        if title_caps and pars and len(pars[0]) == 1 and caps_line(pars[0][0]):
            title = head + " " + pars[0][0].strip()
            pars = pars[1:]
        chapters.append((title, [para_block(p) for p in pars if p != ["[Illustration]"]]))
    return lines[:first], chapters


def waverley(text):
    b = Book("Waverley", "Sir Walter Scott")
    lines = text.split("\n")
    start = lines.index("Volume I.")
    notes_at = lines.index("NOTES—Volume I.")
    body = lines[start:notes_at]
    _, parts = split_sections(body, lambda l: re.match(r"^(Volume|VOLUME) [IVX]+\.$", l) is not None)
    for head, rest in parts:
        _, chapters = chapters_of(rest, r"^CHAPTER [IVXLC]+\.$", True)
        b.parts.append((head, chapters))
    label = None
    for par in blocks_of(lines[notes_at:]):
        first = par[0].strip()
        m = re.match(r"^NOTE (\d+)$", first)
        if m and len(par) == 1:
            label = m.group(1)
            b.notes[label] = []
        elif first.startswith("NOTES—") or par == ["[Illustration]"]:
            label = None
        elif label:
            b.notes[label].append(" ".join(l.strip() for l in par))
    return b


def gulliver(text):
    b = Book("Gulliver’s Travels", "Jonathan Swift")
    lines = text.split("\n")
    first_part = next(i for i, l in enumerate(lines) if l.startswith("PART I."))
    notes_at = next(i for i, l in enumerate(lines) if re.match(r"^\[\d+\] ", l))
    front = lines[:first_part]
    _, secs = split_sections(front, lambda l: l.startswith("THE PUBLISHER TO THE READER") or l.startswith("A LETTER FROM CAPTAIN GULLIVER"))
    for head, rest in secs:
        b.front.append((head, [para_block(p) for p in blocks_of(rest)]))
    _, parts = split_sections(lines[first_part:notes_at], lambda l: re.match(r"^PART [IVX]+\. ", l) is not None)
    for head, rest in parts:
        _, chapters = chapters_of(rest, r"^CHAPTER [IVXLC]+\.$", False)
        b.parts.append((head, chapters))
    for par in blocks_of(lines[notes_at:]):
        m = re.match(r"^\[(\d+)\] (.*)$", par[0])
        if m:
            b.notes[m.group(1)] = [" ".join([m.group(2)] + [l.strip() for l in par[1:]])]
    return b


def middlemarch(text):
    b = Book("Middlemarch", "George Eliot")
    lines = text.split("\n")
    prelude = lines.index("PRELUDE.")
    book1 = lines.index("BOOK I.")
    finale = lines.index("FINALE.")
    b.front.append(("PRELUDE.", [para_block(p) for p in blocks_of(lines[prelude + 1 : book1])]))
    end = next((i for i in range(finale + 1, len(lines)) if lines[i].strip() == "THE END"), len(lines))
    b.back.append(("FINALE.", [para_block(p) for p in blocks_of(lines[finale + 1 : end])]))
    _, parts = split_sections(lines[book1:finale], lambda l: re.match(r"^BOOK [IVX]+\.$", l) is not None)
    for head, rest in parts:
        pre, chapters = chapters_of(rest, r"^CHAPTER [IVXLC]+\.$", False)
        title = " ".join(l.strip() for l in pre if l.strip())
        b.parts.append((head + (" " + title if title else ""), chapters))
    return b


PARSERS = {"waverley": waverley, "gullivers-travels": gulliver, "middlemarch": middlemarch}

# ---------------------------------------------------------------------------------------------
# inline text: _italic_ and [n] note references

TOKEN = re.compile(r"_([^_]+)_|\s?\[(\d+)\]")


def inlines(text, notes):
    """[(kind, value)]: ("t", text), ("i", italic text), ("n", label)."""
    out, pos = [], 0
    for m in TOKEN.finditer(text):
        if m.group(2) is not None and m.group(2) not in notes:
            continue
        if m.start() > pos:
            out.append(("t", text[pos : m.start()]))
        if m.group(1) is not None:
            out.append(("i", m.group(1)))
        else:
            out.append(("n", m.group(2)))
        pos = m.end()
    if pos < len(text):
        out.append(("t", text[pos:]))
    return out


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")


# ---------------------------------------------------------------------------------------------
# zip


def write_zip(path, entries):
    """entries: [(name, bytes)]; `mimetype` (if any) first and stored, the rest sorted."""
    with zipfile.ZipFile(path, "w") as z:
        first = [e for e in entries if e[0] == "mimetype"]
        rest = sorted(e for e in entries if e[0] != "mimetype")
        for name, data in first + rest:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            info.create_system = 3
            z.writestr(info, data if isinstance(data, bytes) else data.encode("utf-8"))


# ---------------------------------------------------------------------------------------------
# DOCX

W = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"'


def docx_runs(text, notes, used, rpr=""):
    out = []
    for kind, v in inlines(text, notes):
        if kind == "t":
            out.append('<w:r>%s<w:t xml:space="preserve">%s</w:t></w:r>' % (("<w:rPr>%s</w:rPr>" % rpr) if rpr else "", esc(v)))
        elif kind == "i":
            out.append('<w:r><w:rPr>%s<w:i/></w:rPr><w:t xml:space="preserve">%s</w:t></w:r>' % (rpr, esc(v)))
        else:
            used.append(v)
            out.append('<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="%s"/></w:r>' % v)
    return "".join(out)


def docx_block(block, notes, used, rpr=""):
    kind, v = block
    if kind == "p":
        return "<w:p>%s</w:p>" % docx_runs(v, notes, used, rpr)
    if kind == "verse":
        parts = []
        for n, line in enumerate(v):
            if n:
                parts.append("<w:r><w:br/></w:r>")
            parts.append(docx_runs(line, notes, used, rpr))
        return '<w:p><w:pPr><w:pStyle w:val="Verse"/></w:pPr>%s</w:p>' % "".join(parts)
    if kind == "break":
        return '<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r>%s<w:t>%s</w:t></w:r></w:p>' % (("<w:rPr>%s</w:rPr>" % rpr) if rpr else "", esc(v))
    return ""


def docx_heading(level, text):
    return '<w:p><w:pPr><w:pStyle w:val="Heading%d"/></w:pPr><w:r><w:t xml:space="preserve">%s</w:t></w:r></w:p>' % (level, esc(text))


DOCX_STYLES = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:styles %s>'
    '<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>'
    '<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/></w:rPr></w:style>'
    '<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/></w:rPr></w:style>'
    '<w:style w:type="paragraph" w:styleId="Verse"><w:name w:val="Verse"/><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="720"/></w:pPr></w:style>'
    '<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>'
    "</w:styles>" % W
)


def docx_package(body, footnotes):
    ct = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
        '<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>'
        '<Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>'
        "</Types>"
    )
    rels = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
        "</Relationships>"
    )
    doc_rels = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
        '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>'
        "</Relationships>"
    )
    document = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:document %s><w:body>%s<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>' % (W, body)
    fns = ['<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>', '<w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>']
    for label, paras in footnotes:
        ps = []
        for n, text in enumerate(paras):
            ref = '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r>' if n == 0 else ""
            ps.append("<w:p>%s%s</w:p>" % (ref, docx_runs(text, {}, [])))
        fns.append('<w:footnote w:id="%s">%s</w:footnote>' % (label, "".join(ps)))
    footxml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:footnotes %s>%s</w:footnotes>' % (W, "".join(fns))
    return [
        ("[Content_Types].xml", ct),
        ("_rels/.rels", rels),
        ("word/_rels/document.xml.rels", doc_rels),
        ("word/document.xml", document),
        ("word/styles.xml", DOCX_STYLES),
        ("word/footnotes.xml", footxml),
    ]


def all_sections(b):
    """(level, title, blocks): front and back matter and chapters at level 2, parts at 1."""
    out = [(2, t, bl) for t, bl in b.front]
    for pt, chapters in b.parts:
        out.append((1, pt, []))
        out += [(2, t, bl) for t, bl in chapters]
    out += [(2, t, bl) for t, bl in b.back]
    return out


def write_docx(b, path):
    used, body = [], []
    for level, title, blocks in all_sections(b):
        body.append(docx_heading(level, title))
        body += [docx_block(bl, b.notes, used) for bl in blocks]
    seen = []
    for l in used:
        if l not in seen:
            seen.append(l)
    write_zip(path, docx_package("".join(body), [(l, b.notes[l]) for l in seen]))


ONLINE_RPR = '<w:rFonts w:ascii="Document Serif" w:hAnsi="Document Serif" w:cs="Document Serif"/><w:sz w:val="24"/><w:szCs w:val="24"/>'
BREAKS = ["* * *", "***", "#", "*  *  *", "~"]


def write_online_docx(b, path):
    """Gulliver's travels as an online word processor writes a download: fonts and sizes on every
    run, no heading styles, bold "Chapter N" lines after a page break, part titles as bold
    capitals, an empty paragraph after each heading and uneven scene-break marks every twelfth
    paragraph."""
    used, body, n, chapter = [], [], 0, 0
    body.append('<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr>%s<w:b/><w:sz w:val="40"/></w:rPr><w:t>%s</w:t></w:r></w:p>' % (ONLINE_RPR.replace('<w:sz w:val="24"/>', ""), esc(b.title)))
    for pt, chapters in b.parts:
        body.append('<w:p><w:r><w:br w:type="page"/></w:r></w:p>')
        body.append('<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr>%s<w:b/></w:rPr><w:t xml:space="preserve">%s</w:t></w:r></w:p>' % (ONLINE_RPR, esc(pt)))
        for _, blocks in chapters:
            chapter += 1
            body.append('<w:p><w:r><w:br w:type="page"/></w:r></w:p>')
            body.append('<w:p><w:r><w:rPr>%s<w:b/></w:rPr><w:t xml:space="preserve">Chapter %d</w:t></w:r></w:p>' % (ONLINE_RPR, chapter))
            body.append("<w:p/>")
            for k, bl in enumerate(blocks):
                if k and k % 12 == 0:
                    body.append(docx_block(("break", BREAKS[n % len(BREAKS)]), b.notes, used, ONLINE_RPR))
                    n += 1
                body.append(docx_block(bl, b.notes, used, ONLINE_RPR))
    seen = []
    for l in used:
        if l not in seen:
            seen.append(l)
    write_zip(path, docx_package("".join(body), [(l, b.notes[l]) for l in seen]))


# ---------------------------------------------------------------------------------------------
# ODT

ODT_NS = (
    'xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" '
    'xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" '
    'xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" '
    'xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.3"'
)


def odt_inlines(text, notes, state):
    out = []
    for kind, v in inlines(text, notes):
        if kind == "t":
            out.append(esc(v))
        elif kind == "i":
            out.append('<text:span text:style-name="Italic">%s</text:span>' % esc(v))
        else:
            if v in state:
                continue
            state.add(v)
            body = "".join("<text:p>%s</text:p>" % odt_inlines(p, {}, set()) for p in notes[v])
            out.append('<text:note text:id="n%s" text:note-class="footnote"><text:note-citation>%s</text:note-citation><text:note-body>%s</text:note-body></text:note>' % (v, v, body))
    return "".join(out)


def write_odt(b, path):
    state, body = set(), []
    for level, title, blocks in all_sections(b):
        body.append('<text:h text:style-name="Heading_20_%d" text:outline-level="%d">%s</text:h>' % (level, level, esc(title)))
        for kind, v in blocks:
            if kind == "p":
                body.append("<text:p>%s</text:p>" % odt_inlines(v, b.notes, state))
            elif kind == "verse":
                body.append('<text:p text:style-name="Verse">%s</text:p>' % "<text:line-break/>".join(odt_inlines(l, b.notes, state) for l in v))
    content = (
        '<?xml version="1.0" encoding="UTF-8"?>\n<office:document-content %s>'
        '<office:automatic-styles><style:style style:name="Italic" style:family="text"><style:text-properties fo:font-style="italic"/></style:style></office:automatic-styles>'
        "<office:body><office:text>%s</office:text></office:body></office:document-content>" % (ODT_NS, "".join(body))
    )
    styles = (
        '<?xml version="1.0" encoding="UTF-8"?>\n<office:document-styles %s><office:styles>'
        '<style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:default-outline-level="1"/>'
        '<style:style style:name="Heading_20_2" style:display-name="Heading 2" style:family="paragraph" style:default-outline-level="2"/>'
        '<style:style style:name="Verse" style:family="paragraph"><style:paragraph-properties fo:margin-left="0.5in"/></style:style>'
        "</office:styles></office:document-styles>" % ODT_NS
    )
    manifest = (
        '<?xml version="1.0" encoding="UTF-8"?>\n<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">'
        '<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>'
        '<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>'
        '<manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>'
        "</manifest:manifest>"
    )
    write_zip(path, [("mimetype", b"application/vnd.oasis.opendocument.text"), ("content.xml", content), ("styles.xml", styles), ("META-INF/manifest.xml", manifest)])


# ---------------------------------------------------------------------------------------------
# RTF


def rtf_text(s):
    out = []
    for ch in s:
        o = ord(ch)
        if ch in "\\{}":
            out.append("\\" + ch)
        elif o < 128:
            out.append(ch)
        else:
            out.append("\\u%d?" % (o if o < 32768 else o - 65536))
    return "".join(out)


def rtf_inlines(text, notes, state):
    out = []
    for kind, v in inlines(text, notes):
        if kind == "t":
            out.append(rtf_text(v))
        elif kind == "i":
            out.append("{\\i %s}" % rtf_text(v))
        else:
            if v in state:
                continue
            state.add(v)
            body = "\\par ".join(rtf_inlines(p, {}, set()) for p in notes[v])
            out.append("{\\super\\chftn}{\\footnote\\pard\\plain{\\super\\chftn} %s}" % body)
    return "".join(out)


def write_rtf(b, path):
    state = set()
    out = [
        "{\\rtf1\\ansi\\ansicpg1252\\deff0\\uc1{\\fonttbl{\\f0\\froman Serif;}}"
        "{\\stylesheet{\\s0 Normal;}{\\s1\\outlinelevel0\\b heading 1;}{\\s2\\outlinelevel1\\b heading 2;}}\n"
    ]
    for level, title, blocks in all_sections(b):
        out.append("\\pard\\plain\\s%d\\outlinelevel%d\\b %s\\par\n" % (level, level - 1, rtf_text(title)))
        for kind, v in blocks:
            if kind == "p":
                out.append("\\pard\\plain %s\\par\n" % rtf_inlines(v, b.notes, state))
            elif kind == "verse":
                out.append("\\pard\\plain\\li720 %s\\par\n" % "\\line ".join(rtf_inlines(l, b.notes, state) for l in v))
    out.append("}\n")
    with open(path, "w", encoding="ascii", newline="\n") as f:
        f.write("".join(out))


# ---------------------------------------------------------------------------------------------
# Markdown


def md_inlines(text, notes, used):
    out = []
    for kind, v in inlines(text, notes):
        if kind == "t":
            out.append(re.sub(r"([\\`*_\[\]<>#])", r"\\\1", v))
        elif kind == "i":
            out.append("*%s*" % re.sub(r"([\\`*_\[\]<>#])", r"\\\1", v))
        else:
            used.append(v)
            out.append("[^%s]" % v)
    return "".join(out)


def write_md(b, path):
    out, used = [], []
    for level, title, blocks in all_sections(b):
        out.append("#" * level + " " + title + "\n")
        for kind, v in blocks:
            if kind == "p":
                out.append(md_inlines(v, b.notes, used) + "\n")
            elif kind == "verse":
                out.append("\\\n".join(md_inlines(l, b.notes, used) for l in v) + "\n")
    seen = []
    for l in used:
        if l not in seen:
            seen.append(l)
    for l in seen:
        paras = [md_inlines(p, {}, []) for p in b.notes[l]]
        out.append("[^%s]: %s\n" % (l, "\n\n    ".join(paras)))
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out))


# ---------------------------------------------------------------------------------------------


def make_documents():
    for _, name in TEXTS:
        with open(os.path.join(NOVELS, name + ".txt"), encoding="utf-8") as f:
            b = PARSERS[name](f.read())
        base = os.path.join(NOVELS, name)
        write_docx(b, base + ".docx")
        if name != "middlemarch":
            write_odt(b, base + ".odt")
            write_rtf(b, base + ".rtf")
            write_md(b, base + ".md")
        if name == "gullivers-travels":
            write_online_docx(b, os.path.join(NOVELS, name + "-online.docx"))
        chapters = sum(len(c) for _, c in b.parts)
        print("%s: %d front, %d parts, %d chapters, %d back, %d notes" % (name, len(b.front), len(b.parts), chapters, len(b.back), len(b.notes)))


def main():
    if len(sys.argv) >= 3 and sys.argv[1] == "texts":
        make_texts(sys.argv[2])
    elif len(sys.argv) == 2 and sys.argv[1] == "documents":
        make_documents()
    else:
        print(__doc__)
        sys.exit(2)


if __name__ == "__main__":
    main()
