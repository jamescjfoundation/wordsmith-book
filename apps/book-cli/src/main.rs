// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book on the command line. The commands arrive in step M9.2; this prints the usage.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::process::ExitCode;

fn main() -> ExitCode {
    println!("usage: book-cli <command>  (commands arrive with step M9.2)");
    ExitCode::SUCCESS
}
