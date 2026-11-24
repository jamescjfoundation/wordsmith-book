// SPDX-License-Identifier: MIT OR Apache-2.0
//! ISBN checksums (plan M3.3): every ISBN made with its check digit passes, any single changed
//! digit fails, and ISBN-10s turn into the same book's ISBN-13.

use proptest::prelude::*;
use wordsmith_book_model::isbn::{check, check10, check13};

proptest! {
    #[test]
    fn isbn_13_check_digits_catch_any_one_changed_digit(prefix in prop_oneof![Just(978u32), Just(979u32)], rest in proptest::collection::vec(0u32..10, 9), at in 3usize..13, by in 1u32..10) {
        let mut d: Vec<u32> = vec![prefix / 100, (prefix / 10) % 10, prefix % 10];
        d.extend(rest);
        let c = check13(&d);
        d.push(c);
        let s: String = d.iter().map(u32::to_string).collect();
        prop_assert_eq!(check(&s), Ok(s.clone()));
        let mut bad = d.clone();
        bad[at] = (bad[at] + by) % 10;
        let b: String = bad.iter().map(u32::to_string).collect();
        prop_assert!(check(&b).is_err());
    }

    #[test]
    fn isbn_10s_check_and_become_isbn_13s(body in proptest::collection::vec(0u32..10, 9)) {
        let c = check10(&body);
        let s: String = body.iter().map(u32::to_string).collect::<String>() + &if c == 10 { "X".to_owned() } else { c.to_string() };
        let thirteen = check(&s).unwrap();
        prop_assert!(thirteen.starts_with("978"));
        prop_assert_eq!(&thirteen[3..12], &s[..9]);
        prop_assert_eq!(check(&thirteen), Ok(thirteen.clone()));
    }
}
