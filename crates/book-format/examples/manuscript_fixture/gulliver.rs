// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gulliver's Travels as a Wordsmith Manuscript project, for the fixture (plan M1.2). Shared by
//! the `manuscript_fixture` example (which writes it) and the tests (which check the book view
//! against it). Deterministic: ids come from a counter.

#![allow(dead_code)]

use wordsmith_manuscript_model::{Area, Block, Inline, Item, ItemId, ItemText, Kind, Marks, Note, Place, Project, Role, Text};

/// A section as the fixture text gives it: (heading, paragraphs as lines).
type Section = (String, Vec<Vec<String>>);

fn blocks_of(lines: &[&str]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for l in lines {
        if l.trim().is_empty() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(l.trim_end().to_owned());
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn split(lines: &[&str], heading: impl Fn(&str) -> bool) -> Vec<Section> {
    let idx: Vec<usize> = lines.iter().enumerate().filter(|(_, l)| heading(l)).map(|(i, _)| i).collect();
    idx.iter()
        .enumerate()
        .map(|(n, &i)| {
            let j = idx.get(n + 1).copied().unwrap_or(lines.len());
            (lines[i].trim().to_owned(), blocks_of(&lines[i + 1..j]))
        })
        .collect()
}

/// Text with `_italic_` and `[n]` note references into inlines (references only for `notes`).
pub fn inlines(text: &str, notes: &[String]) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut italic = false;
    let mut buf = String::new();
    let mut chars = text.char_indices().peekable();
    let flush = |buf: &mut String, italic: bool, out: &mut Vec<Inline>| {
        if !buf.is_empty() {
            out.push(Inline::marked(std::mem::take(buf), if italic { Marks::italic() } else { Marks::default() }));
        }
    };
    while let Some((i, c)) = chars.next() {
        if c == '_' {
            flush(&mut buf, italic, &mut out);
            italic = !italic;
            continue;
        }
        if c == '[' {
            let rest = &text[i + 1..];
            if let Some(end) = rest.find(']') {
                let label = &rest[..end];
                if notes.iter().any(|n| n == label) {
                    if buf.ends_with(' ') {
                        buf.pop();
                    }
                    flush(&mut buf, italic, &mut out);
                    out.push(Inline::NoteRef { label: label.to_owned(), marks: Marks::default() });
                    for _ in 0..=end {
                        chars.next();
                    }
                    continue;
                }
            }
        }
        buf.push(c);
    }
    flush(&mut buf, italic, &mut out);
    out
}

fn is_verse(par: &[String]) -> bool {
    par.len() >= 2 && par.iter().all(|l| l.trim().chars().count() <= 60) && par.iter().any(|l| l.starts_with("  "))
}

fn block(par: &[String], notes: &[String]) -> Block {
    if is_verse(par) {
        let joined = par.iter().map(|l| l.trim()).collect::<Vec<_>>().join("\n");
        return Block::Fenced { kind: wordsmith_manuscript_model::Fence::Verse, paras: vec![inlines(&joined, notes)] };
    }
    Block::Para(inlines(&par.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" "), notes))
}

/// The book: (front sections, parts of (title, chapters), notes by label).
pub struct Parsed {
    pub front: Vec<Section>,
    pub parts: Vec<(String, Vec<Section>)>,
    pub notes: Vec<(String, String)>,
}

pub fn parse(text: &str) -> Parsed {
    let lines: Vec<&str> = text.lines().collect();
    let first_part = lines.iter().position(|l| l.starts_with("PART I.")).unwrap_or(0);
    let notes_at = lines
        .iter()
        .position(|l| l.starts_with('[') && l[1..].split(']').next().is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())))
        .unwrap_or(lines.len());
    let front = split(&lines[..first_part], |l| l.starts_with("THE PUBLISHER TO THE READER") || l.starts_with("A LETTER FROM CAPTAIN GULLIVER"));
    let parts =
        split(&lines[first_part..notes_at], |l| l.starts_with("PART ") && l.contains(". ")).into_iter().map(|(head, _)| head).collect::<Vec<_>>();
    let part_starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(i, l)| *i >= first_part && *i < notes_at && l.starts_with("PART ") && l.contains(". "))
        .map(|(i, _)| i)
        .collect();
    let mut out_parts = Vec::new();
    for (n, head) in parts.into_iter().enumerate() {
        let a = part_starts[n] + 1;
        let b = part_starts.get(n + 1).copied().unwrap_or(notes_at);
        let chapters = split(&lines[a..b], |l| l.starts_with("CHAPTER ") && l.ends_with('.'));
        out_parts.push((head, chapters));
    }
    let mut notes = Vec::new();
    for par in blocks_of(&lines[notes_at..]) {
        let first = &par[0];
        if let Some(rest) = first.strip_prefix('[')
            && let Some((label, body)) = rest.split_once("] ")
        {
            let mut t = body.to_owned();
            for l in &par[1..] {
                t.push(' ');
                t.push_str(l.trim());
            }
            notes.push((label.to_owned(), t));
        }
    }
    Parsed { front, parts: out_parts, notes }
}

/// The part's title without its number: "PART I. A VOYAGE TO LILLIPUT." → "A VOYAGE TO LILLIPUT".
pub fn part_title(head: &str) -> String {
    head.split_once(". ").map_or(head, |(_, t)| t).trim_end_matches('.').to_owned()
}

/// Ids the fixture gives, by role, for the tests.
pub struct Ids {
    pub title_page: ItemId,
    pub front: Vec<ItemId>,
    pub parts: Vec<ItemId>,
    pub chapters: Vec<ItemId>,
    pub cut: ItemId,
    pub about: ItemId,
    pub research: ItemId,
}

/// The Manuscript project, and the ids it gave.
pub fn project(text: &str) -> (Project, Ids) {
    let parsed = parse(text);
    let note_labels: Vec<String> = parsed.notes.iter().map(|(l, _)| l.clone()).collect();
    let mut n = 0u64;
    let mut next = || {
        n += 1;
        ItemId::from_counter(n)
    };
    let mut p = Project::new();
    p.settings.book.title = "Gulliver’s Travels".into();
    p.settings.book.subtitle = "into Several Remote Nations of the World".into();
    p.settings.book.author = "Jonathan Swift".into();
    p.settings.book.language = "en".into();
    p.settings.book.rights = "This text is in the public domain.".into();
    p.settings.book.description = "Four voyages to strange nations, told by a ship's surgeon.".into();
    p.settings.book.publisher = "Fixture Press".into();
    let title_page = next();
    let mut ids =
        Ids { title_page, front: Vec::new(), parts: Vec::new(), chapters: Vec::new(), cut: title_page, about: title_page, research: title_page };
    let add = |p: &mut Project, place: Place, item: Item, text: Option<Text>| {
        let id = item.id;
        let _ = p.binder.insert(place, item);
        if let Some(t) = text {
            p.texts.insert(id, ItemText::with_body(t));
        }
    };
    add(
        &mut p,
        Place::end_of(Area::Manuscript),
        Item::with_id(title_page, Kind::Text, "Title Page").role(Role::FrontMatter),
        Some(Text::from_paragraphs(&["Gulliver’s Travels", "by Jonathan Swift"])),
    );
    for (head, pars) in &parsed.front {
        let id = next();
        ids.front.push(id);
        let mut t = Text { blocks: pars.iter().map(|par| block(par, &note_labels)).collect(), notes: Vec::new() };
        t.normalize();
        add(&mut p, Place::end_of(Area::Manuscript), Item::with_id(id, Kind::Text, head.trim_end_matches('.')).role(Role::FrontMatter), Some(t));
    }
    for (pi, (head, chapters)) in parsed.parts.iter().enumerate() {
        let pid = next();
        ids.parts.push(pid);
        add(&mut p, Place::end_of(Area::Manuscript), Item::with_id(pid, Kind::Folder, part_title(head)).role(Role::Part), None);
        for (ci, (_, pars)) in chapters.iter().enumerate() {
            let cid = next();
            ids.chapters.push(cid);
            let mut t = Text { blocks: pars.iter().map(|par| block(par, &note_labels)).collect(), notes: Vec::new() };
            for (label, body) in &parsed.notes {
                if t.blocks
                    .iter()
                    .any(|b| matches!(b, Block::Para(c) if c.iter().any(|i| matches!(i, Inline::NoteRef { label: l, .. } if l == label))))
                {
                    t.notes.push(Note { label: label.clone(), paras: vec![inlines(body, &[])] });
                }
            }
            // A comment on the first words of the first chapter: the view keeps no comments.
            if pi == 0
                && ci == 0
                && let Some(Block::Para(c)) = t.blocks.first_mut()
                && let Some(Inline::Span { marks, .. }) = c.first_mut()
            {
                marks.comment = Some("c1".into());
            }
            t.normalize();
            add(&mut p, Place::inside(Area::Manuscript, pid), Item::with_id(cid, Kind::Text, ""), Some(t));
            if let Some(item) = p.binder.get_mut(cid) {
                item.role = Role::Chapter;
            }
        }
        if pi == 0 {
            // An item left out of export: not a section of the book.
            let cut = next();
            ids.cut = cut;
            let mut item = Item::with_id(cut, Kind::Text, "A Scene Cut from Lilliput").role(Role::Chapter);
            item.include = false;
            add(&mut p, Place::inside(Area::Manuscript, pid), item, Some(Text::from_paragraphs(&["These words were cut and never print."])));
        }
    }
    let about = next();
    ids.about = about;
    add(
        &mut p,
        Place::end_of(Area::Manuscript),
        Item::with_id(about, Kind::Text, "About the Author").role(Role::BackMatter),
        Some(Text::from_paragraphs(&["Jonathan Swift was a satirist and a dean."])),
    );
    let research = next();
    ids.research = research;
    add(
        &mut p,
        Place::end_of(Area::Research),
        Item::with_id(research, Kind::Text, "Notes on Ships"),
        Some(Text::from_paragraphs(&["A research note, never part of the book."])),
    );
    (p, ids)
}
