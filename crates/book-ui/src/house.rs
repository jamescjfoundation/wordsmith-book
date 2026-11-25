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
    b.edit(|f| {
        if let Some((choice, text)) = design {
            if let Some(text) = text {
                f.designs.insert(choice.id.clone(), text);
            }
            f.project.design = choice;
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
