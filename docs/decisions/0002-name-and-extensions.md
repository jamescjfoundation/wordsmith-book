# 0002: The name and the file extensions

Date: 2026-11-24. Status: accepted.

## Context

A Book project is a folder, with a packed single-file form for sending, backups and recovery
(plan section 6.2). The plan proposed `.wsb` for the folder and `.wsbz` for the packed file,
beside Manuscript's `.wsm` / `.wsmz` and Script's `.wss`. An extension the operating system
already gives to a common file type would make double-clicking a project open the wrong program,
and registering ours would take that file type over.

## What was checked

For each extension, on each of the three desktop platforms the core supports: the file types the
system registers by default, the types its built-in features register, and the extensions in
common use by widely installed programs (from the platforms' published type registries and
public extension lists as known on 2026-11-24; a person may recheck before packaging).

- `.wsb`: **taken**. On one desktop platform a built-in system feature registers `.wsb` for its
  own configuration files and opens them by double-click. Registering Book there would take
  those files over, so `.wsb` is not used.
- `.wsbz`: no registration found on any platform.
- `.wsbk` and `.wsbkz`: no registration found on any platform, no common program using them, and
  they read as "Wordsmith book" beside `.wsm` and `.wss`.

## Decision

- The app's name is **Wordsmith Book**; the desktop binary is `wordsmith-book`, the command line
  `book-cli`; the app id is `org.jcjfoundation.book`.
- A Book project folder ends in **`.wsbk`**; the packed project is **`.wsbkz`** (a deterministic
  zip, `mimetype` first, as core decision 0009 describes). The packed name follows the folder's,
  as Manuscript's does, so `.wsbz` is not used either.
- Packaging (M9.4) registers `.wsbkz` as a document type and `.wsbk` as a package or folder type
  where the platform has one.
- The plan (sections 6.2, 10 and 14) uses these extensions from this step on.

## Consequences

The format (`docs/format.md`), packing and the app's open dialogs use `.wsbk` / `.wsbkz`.

## Revisit when

A platform starts registering either extension, or the foundation sets a common scheme for its
apps' extensions.
