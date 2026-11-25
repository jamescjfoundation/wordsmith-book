// SPDX-License-Identifier: MIT OR Apache-2.0
//! A book in a publishing house (DSN-7, META-3, META-4): joining an imprint and a series from
//! the house file, which brings the house design and, when the imprint or series locks it, keeps
//! it; leaving them, which frees it.

use std::path::{Path, PathBuf};

use wordsmith_book_model::DesignChoice;
use wordsmith_book_model::house::{DesignLock, House, HouseLink};

use crate::doc::OpenBook;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The house file at `path`.
pub fn load(path: &Path) -> Result<House, String> {
    wordsmith_book_format::house::read(path).map_err(err)
}

/// The design a house names: a built-in design, or one in the house folder's `designs/`, which
/// the book then keeps among its own (its id and text).
fn design_of(house_file: &Path, id: &str) -> Result<(DesignChoice, Option<String>), String> {
    let choice = DesignChoice { id: id.to_owned(), ..DesignChoice::default() };
    if wordsmith_book_design::builtin::get(id).is_some() {
        return Ok((choice, None));
    }
    let bytes = wordsmith_book_format::house::beside(house_file, &format!("designs/{id}.toml")).map_err(err)?;
    let text = String::from_utf8(bytes).map_err(|_| format!("the house design “{id}” is not text"))?;
    let d = wordsmith_book_design::design::read(&text, &wordsmith_book_design::known_role).map_err(|e| format!("the house design “{id}”: {e}"))?;
    if d.id != id {
        return Err(format!("the house design file for “{id}” is the design “{}”", d.id));
    }
    Ok((choice, Some(text)))
}

/// Join an imprint, a series (its imprint too, when it has one), or both, from the house file;
/// gives what the book now is in.
pub fn join(b: &mut OpenBook, house_file: &Path, imprint: Option<&str>, series: Option<&str>, number: Option<u32>) -> Result<HouseLink, String> {
    let house = load(house_file)?;
    let s = match series {
        Some(id) => Some(house.series(id).ok_or_else(|| format!("the house has no series “{id}”"))?.clone()),
        None => None,
    };
    let imprint_id = imprint.map(str::to_owned).or_else(|| s.as_ref().map(|s| s.imprint.clone()).filter(|i| !i.is_empty()));
    let im = match &imprint_id {
        Some(id) => Some(house.imprint(id).ok_or_else(|| format!("the house has no imprint “{id}”"))?.clone()),
        None => None,
    };
    if im.is_none() && s.is_none() {
        return Err("name an imprint, a series, or both".into());
    }
    // The design: the series' over the imprint's; locked when the one that gives it locks it.
    let given = match (&s, &im) {
        (Some(s), _) if !s.design.is_empty() => Some((s.design.clone(), s.lock_design, format!("the series “{}”", s.name))),
        (_, Some(i)) if !i.design.is_empty() => Some((i.design.clone(), i.lock_design, format!("the imprint “{}”", i.name))),
        _ => None,
    };
    let design = match &given {
        Some((id, _, _)) => Some(design_of(house_file, id)?),
        None => None,
    };
    // The imprint's details (META-3): its logo kept among the book's pictures.
    let details = match &im {
        Some(i) => {
            let logo = if i.logo.trim().is_empty() {
                None
            } else {
                let bytes = wordsmith_book_format::house::beside(house_file, &i.logo).map_err(err)?;
                let ext = match wordsmith_book_model::picture::size(&bytes) {
                    Some(_) if bytes.starts_with(b"\x89PNG") => "png",
                    Some(_) => "jpg",
                    None => return Err(format!("the imprint's logo “{}” is not a PNG or JPEG", i.logo)),
                };
                Some((format!("imprint-logo.{ext}"), bytes))
            };
            let imprint = wordsmith_book_model::Imprint {
                name: i.name.clone(),
                address: i.address.clone(),
                website: i.website.clone(),
                logo: logo.as_ref().map(|(n, _)| format!("assets/{n}")).unwrap_or_default(),
                copyright: i.copyright.clone(),
            };
            Some((imprint, logo))
        }
        None => None,
    };
    let link = HouseLink {
        imprint: im.as_ref().map(|i| i.id.clone()).unwrap_or_default(),
        series: s.as_ref().map(|s| s.id.clone()).unwrap_or_default(),
        series_number: number,
        lock: match (&given, &design) {
            (Some((_, true, by)), Some((choice, _))) => Some(DesignLock { by: by.clone(), design: choice.clone() }),
            _ => None,
        },
    };
    let out = link.clone();
    let series_details = s.clone();
    b.edit(|f| {
        if let Some((choice, text)) = design {
            if let Some(text) = text {
                f.designs.insert(choice.id.clone(), text);
            }
            f.project.design = choice;
        }
        // A series (META-4): the copyright page's series line, and "also by" from its other
        // books (in order), in place of any the book listed for that series before.
        if let Some(s) = &series_details {
            let line = match number {
                Some(n) => format!("{}, Book {n}", s.name),
                None => s.name.clone(),
            };
            f.project.metadata.overrides.insert("series".into(), line);
            f.project.metadata.also_by.retain(|b| b.series != s.name);
            let mut books: Vec<_> = s.books.iter().filter(|b| Some(b.number) != number && !b.title.trim().is_empty()).collect();
            books.sort_by_key(|b| b.number);
            f.project
                .metadata
                .also_by
                .extend(books.into_iter().map(|b| wordsmith_book_model::OtherBook { title: b.title.clone(), series: s.name.clone() }));
        }
        if let Some((imprint, logo)) = details {
            if let Some((name, bytes)) = logo {
                let path = format!("assets/{name}");
                f.assets.insert(name, bytes);
                if !f.project.decorative.contains(&path) {
                    f.project.decorative.push(path);
                }
            }
            f.project.metadata.imprint = Some(imprint);
        }
        f.project.house = link;
        Ok(())
    })?;
    Ok(out)
}

/// Leave the house's imprint and series: the design stays, free to change.
pub fn leave(b: &mut OpenBook) -> Result<(), String> {
    b.edit(|f| {
        f.project.house = HouseLink::default();
        Ok(())
    })
}

/// The house file the preferences name, or the one given.
pub fn file(given: Option<&str>, prefs: &str) -> Result<PathBuf, String> {
    let p = given.map(str::trim).filter(|g| !g.is_empty()).unwrap_or(prefs.trim());
    if p.is_empty() {
        return Err("no house file: give its path, or set it in Preferences".into());
    }
    Ok(PathBuf::from(p))
}

/// One book of the book's series, as the series view shows it.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SeriesRow {
    /// Its number.
    pub number: u32,
    /// Its title.
    pub title: String,
    /// Its Book project, from the house file's folder (empty: not made in Book).
    pub project: String,
    /// The design its project uses (empty: not read).
    pub design: String,
    /// Whether it keeps the series' design (or, without one, this book's).
    pub matches: bool,
    /// Why its design could not be read, when it could not.
    pub problem: String,
}

/// The book's series (META-4): its books in order with the design each project uses, checked
/// against the series' one design. This book is read as it stands; the others from their
/// `book.toml`.
pub fn series_view(b: &OpenBook, house_file: &Path) -> Result<(String, Vec<SeriesRow>), String> {
    let house = load(house_file)?;
    let id = &b.files.project.house.series;
    let s = house
        .series(id)
        .ok_or_else(|| if id.is_empty() { "the book is in no series".to_owned() } else { format!("the house has no series “{id}”") })?;
    let want = if s.design.is_empty() { b.files.project.design.id.clone() } else { s.design.clone() };
    let mut rows = Vec::new();
    let mut books = s.books.clone();
    books.sort_by_key(|x| x.number);
    for x in books {
        let this = Some(x.number) == b.files.project.house.series_number;
        let (design, problem) = if this {
            (b.files.project.design.id.clone(), String::new())
        } else if x.project.trim().is_empty() {
            (String::new(), String::new())
        } else {
            match wordsmith_book_format::house::beside(house_file, &format!("{}/book.toml", x.project.trim_end_matches('/')))
                .map_err(err)
                .and_then(|bytes| String::from_utf8(bytes).map_err(|_| "its book.toml is not text".to_owned()))
                .and_then(|text| wordsmith_book_format::toml_file::read(&text).map_err(err))
            {
                Ok(p) => (p.design.id, String::new()),
                Err(e) => (String::new(), e),
            }
        };
        let matches = design.is_empty() || design == want;
        rows.push(SeriesRow { number: x.number, title: x.title, project: x.project, design, matches, problem });
    }
    Ok((s.name.clone(), rows))
}
