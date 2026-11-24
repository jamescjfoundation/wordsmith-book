// SPDX-License-Identifier: MIT OR Apache-2.0
//! Writes the Manuscript project fixture (plan M1.2) through Manuscript's public folder writer:
//! `cargo xtask fixtures` runs it. Usage: `manuscript_fixture <fixture text> <project folder>`.

mod gulliver;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(src), Some(out)) = (args.first(), args.get(1)) else {
        eprintln!("usage: manuscript_fixture <gullivers-travels.txt> <out .wsm folder>");
        return ExitCode::from(2);
    };
    let text = match std::fs::read_to_string(src) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{src}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (project, _) = gulliver::project(&text);
    let out = PathBuf::from(out);
    if out.exists()
        && let Err(e) = std::fs::remove_dir_all(&out)
    {
        eprintln!("{}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    match wordsmith_manuscript_format::folder::Folder::create(&out, &project) {
        Ok(_) => {
            println!("wrote {}", out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
