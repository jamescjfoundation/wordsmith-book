// SPDX-License-Identifier: MIT OR Apache-2.0
//! The user guide (plan M9.3): the keys page is made from the commands and no two commands share
//! a key; every 0.1 feature id appears in the guide. `UPDATE_GOLDEN=1` writes the keys page again.

use std::path::{Path, PathBuf};

use wordsmith_book_ui::{every_command, keys_page};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn no_two_commands_share_a_key() {
    let mut seen: Vec<(String, String)> = Vec::new();
    for c in every_command() {
        for k in c.shortcut.split(" / ").map(str::trim).filter(|k| !k.is_empty()) {
            let norm = k.to_lowercase();
            if let Some((_, other)) = seen.iter().find(|(s, _)| *s == norm) {
                panic!("{k} is the key of both {other} and {}", c.id);
            }
            seen.push((norm, c.id.clone()));
        }
    }
    assert!(seen.len() > 10);
}

#[test]
fn the_keys_page_is_made_from_the_commands() {
    let path = root().join("docs/guide/keys.md");
    let page = keys_page();
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &page).unwrap();
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap_or_default(), page, "docs/guide/keys.md is out of date: run with UPDATE_GOLDEN=1");
    assert!(page.contains("| Mod+S | Save |"));
}

#[test]
fn every_0_1_feature_id_appears_in_the_guide() {
    let plan = std::fs::read_to_string(root().join("docs/PLAN.md")).unwrap();
    let mut guide = String::new();
    for e in std::fs::read_dir(root().join("docs/guide")).unwrap() {
        guide.push_str(&std::fs::read_to_string(e.unwrap().path()).unwrap());
    }
    let mut ids = Vec::new();
    for line in plan.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // Feature rows: | ID | name | what | release |
        if cells.len() == 6
            && cells[1].split_once('-').is_some_and(|(a, n)| {
                a.chars().all(|c| c.is_ascii_uppercase()) && !a.is_empty() && n.chars().all(|c| c.is_ascii_digit()) && !n.is_empty()
            })
            && cells[4].contains("0.1")
        {
            ids.push(cells[1].to_owned());
        }
    }
    assert!(ids.len() > 40, "{ids:?}");
    let missing: Vec<&String> = ids.iter().filter(|id| !guide.contains(id.as_str())).collect();
    assert!(missing.is_empty(), "the guide does not mention {missing:?}");
}
