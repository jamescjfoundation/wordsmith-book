# Security

## Reporting a problem

Report security problems privately, not in a public issue, pull request or discussion. Use
GitHub's private reporting: the **Security** tab of this repository, then **Report a
vulnerability**. Only the maintainers see the report.

Include what you ran, what happened and what you expected. If you attach a file to show the
problem, remove anything personal from it first; a small made-up example is best.

We aim to reply within seven days, tell you when a fix is ready, and credit you in the advisory
and the changelog unless you ask us not to.

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

Problems in the shared parts (window, saving, format import) are fixed in open-source-core; report
them there or here and we will move the report.

Only the latest release is supported.

## How we handle a report

1. **Keep it private until the fix ships**: not in a public issue, pull request, commit message,
   branch name, code comment or file in the repository. Working notes go in `.private/`.
2. **Fix it in the open, quietly**, with a test that would have caught it.
3. **Release**, then publish the advisory and add a **Security** entry to `CHANGELOG.md`.
