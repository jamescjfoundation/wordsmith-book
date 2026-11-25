// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mixing and the user's own designs (plan M4.3): every pair of built-in designs mixes into a
//! valid design for every part, and export → import is exact.

use std::collections::BTreeMap;

use wordsmith_book_design::design::{self, problems};
use wordsmith_book_design::mix::{find, free_id, mix, resolve};
use wordsmith_book_design::{builtin, known_role};
use wordsmith_book_model::{DesignChoice, DesignPart};

#[test]
fn every_pair_of_built_in_designs_mixes_into_a_valid_design() {
    let all = builtin::all();
    let mut n = 0;
    for a in &all {
        for b in &all {
            for part in DesignPart::ALL {
                let m = mix(a, &[(part, b)]);
                assert!(problems(&m, &known_role).is_empty(), "{} with {}'s {part:?}", a.id, b.id);
                n += 1;
            }
            let everything: Vec<_> = DesignPart::ALL.iter().map(|p| (*p, b)).collect();
            let m = mix(a, &everything);
            assert_eq!(m.body, b.body);
            assert_eq!(m.chapter, b.chapter);
            assert_eq!(m.id, a.id);
        }
    }
    assert_eq!(n, all.len() * all.len() * DesignPart::ALL.len());
    assert!(all.len() >= 12);
}

#[test]
fn a_mixed_choice_resolves_and_saves_and_exports_exactly() {
    let choice = DesignChoice {
        id: "classic".into(),
        mix: BTreeMap::from([(DesignPart::ChapterOpening, "fantasy".into()), (DesignPart::RunningHeads, "literary".into())]),
    };
    let mut own = BTreeMap::new();
    let d = resolve(&choice, &own).unwrap();
    assert_eq!(d.chapter, builtin::get("fantasy").unwrap().chapter);
    assert_eq!(d.heads, builtin::get("literary").unwrap().heads);
    assert_eq!(d.body, builtin::get("classic").unwrap().body);
    // Saved as the user's own, under a free id, it reads back the same.
    let mut mine = d.clone();
    mine.id = free_id("Classic", &own);
    assert_eq!(mine.id, "classic-2", "a built-in id is not reused");
    mine.name = "My Classic".into();
    let text = design::write(&mine).unwrap();
    own.insert(mine.id.clone(), text.clone());
    assert_eq!(find(&mine.id, &own).unwrap(), mine);
    // Exported as one file and imported again: exact.
    assert_eq!(design::read(&text, &known_role).unwrap(), mine);
    assert_eq!(design::write(&design::read(&text, &known_role).unwrap()).unwrap(), text);
    assert!(resolve(&DesignChoice { id: "nope".into(), mix: BTreeMap::new() }, &own).is_err());
}
