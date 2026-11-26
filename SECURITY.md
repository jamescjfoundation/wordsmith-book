# Security

<!-- BEGIN open-source-core: security -->
<!-- Owned by open-source-core (template/SECURITY.md). Change it there, never in a copy: `cargo xtask sync` rewrites the copies. -->
## Reporting a problem

Report security problems privately, not in a public issue, pull request or discussion. Use
GitHub's private reporting: the **Security** tab of this repository, then **Report a
vulnerability**. Only the maintainers see the report.

Include what you ran, what happened and what you expected. If you attach a file to show the
problem, remove anything personal from it first; a small made-up example is best.

We aim to reply within seven days, tell you when a fix is ready, and credit you in the advisory
and the changelog unless you ask us not to.

A problem in the parts every app shares (the window, saving and backups, format import and
export, agent tools, updates) is fixed in
[open-source-core](https://github.com/jamescjfoundation/open-source-core). Report it there or in
the app where you found it; we move the report to where the fix belongs.

## How we handle a report

1. **Keep it private until the fix ships.** Not in a public issue, pull request, commit message,
   branch name, code comment or file in the repository. Discuss it in the private advisory;
   working notes go in `.private/` (ignored by Git).
2. **Fix it in the open, quietly.** The change says what the code now does, not how the old code
   could be abused, and adds a test that would have caught it.
3. **Release the fix.** A fix in the core is released there first, and every app then moves to
   that release and releases in turn.
4. **Disclose:** publish the advisory and add a **Security** entry to `CHANGELOG.md`.

If a problem is already public or being exploited, go straight to a fix and an advisory.
<!-- END open-source-core: security -->

## What counts

Anything that lets a file, an agent or a network peer do more than the user asked for. For
example:

- a Book project, a packed project (`.wsbz`), a design file, a linked Manuscript project or an
  imported file (DOCX, ODT, RTF, Markdown, text, EPUB) that crashes, hangs, or makes the app write
  somewhere unexpected when opened;
- book text that runs as typesetting markup instead of printing as text;
- an agent (MCP) command reading or writing outside the folders it was given;
- an external validator run with more than the file it was asked to check;
- auto-save, recovery or backup files readable by other users.

Only the latest release is supported.
