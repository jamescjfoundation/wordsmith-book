// SPDX-License-Identifier: MIT OR Apache-2.0
//! What only Book does at start-up and while it runs. The start-up itself is the core's
//! (`jcj_shell::desktop`, core decision 0014): the window, the settings folder, the preferences
//! file, the update check, the close that starts an installed update and its helper, the file
//! dialogs. `apps/book` gives it Book's data and builds the app with [`build`]; the steps here
//! are the ones it calls back.

use std::path::PathBuf;

use jcj_shell::Shell;
use jcj_shell::desktop::{DesktopApp, Start};
use jcj_shell::prefs::Prefs;

use crate::prefs::BookPrefs;
use crate::{BookApp, languages, shell_options};

/// Build the app once its window exists: the start window shows while no book is open.
pub fn build(start: &mut Start<'_>) -> BookApp {
    start.shell_with(|s| s.with_options(shell_options()));
    BookApp::new()
}

impl DesktopApp for BookApp {
    type Prefs = Prefs<BookPrefs>;

    /// Book's preferences, then the interface language they name (ACC-3; empty: the
    /// computer's).
    fn apply_prefs(&mut self, prefs: &BookPrefs, _shell: &mut Shell) {
        self.prefs = prefs.clone();
        languages::set(&self.prefs.language);
    }

    fn prefs(&self, _last: &BookPrefs) -> BookPrefs {
        self.prefs.clone()
    }

    /// The project named on the command line, if one was; a problem goes to Book's message.
    fn open(&mut self, files: &[PathBuf], _shell: &mut Shell) {
        if let Some(path) = files.first()
            && let Err(e) = self.open_path(path)
        {
            self.message = Some(e);
        }
    }

    fn report(&mut self, problem: String, _shell: &mut Shell) {
        self.message = Some(problem);
    }
}
