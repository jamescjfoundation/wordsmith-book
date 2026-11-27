# Command line and agents

## The command line

`wordsmith-book-cli` does the app's work without its window, with the same code. It comes in
every download beside the app ([Installing and updating](installing.md) says where it is on
each system), and it makes no network request.

| Command | What it does |
| --- | --- |
| `build <project> [--edition <id>]... [--out <folder>] [--bundle]` | Build, check and write editions (all when none is named) |
| `check <project> [--edition <id>]...` | Preflight, with each finding's page or section |
| `verify <project> <file>` | Check a PDF (letter by letter), an EPUB or a source document (word by word) against the book |
| `convert <file> <project folder> [--title <title>]` | Import a file into a new Book project |
| `report <project>` | Title, author, design, sections, words and details, as JSON |
| `run <project> <command id> [<JSON>]` | Run any Book command; changes are saved |
| `commands` | Every command and its label |
| `mcp [<file>] [--read-root <folder>] [--write-root <folder>]` | Serve Book's commands to an agent over stdio ([below](#agent-tools)) |
| `--help`, `-h`, `help` | The usage |
| `--version`, `-V` | `wordsmith-book-cli` and its version, on one line |

Results are written to standard output. What went wrong is written to standard error, as
`wordsmith-book-cli: <reason>`. Run with no argument, it prints the usage on standard error and
ends with code 2: nothing was asked, so nothing was done.

The app itself takes two things on its command line: `wordsmith-book <project>` opens the
project, and `wordsmith-book --version` prints its name and version and opens no window.

### Exit codes

| Code | Means |
| --- | --- |
| **0** | Done. For `check` and `verify`: the check passed |
| **1** | It could not be done: a project or a file could not be opened or read, a project is there already, a command refused, an edition could not be built |
| **2** | The command line was wrong: no command, an unknown command or option, a missing or extra argument, parameters that are not JSON. Nothing was done |
| **3** | It ran, and a check of the content did not pass: `verify` found that the words differ, `check` found a problem that blocks export, or `build` was stopped by one. Nothing was written |

3 is not a failure. A script reads 0 as "the same" or "nothing blocks", 3 as "they differ" or
"something blocks", and 1 or 2 as "the check did not run". What the check found (the word that
differs, each finding) is its result, so it is on standard output.

These are the codes of every app built on the same base, so a script written for one knows
the next.

## Agent tools

`wordsmith-book-cli mcp` serves every Book command as a tool over the Model Context Protocol, until its
standard input ends: `book_import`, `book_design_choose`, `book_preflight`, `book_export`,
`book_save` and the rest, each described with its parameters. An agent can import, design, build
and check a book with the tools alone. A linked Manuscript project is never changed.

- **Every tool is Book's own and is named `book_<command>`.** The general document tools of
  the base (`execute`, `list_commands`, typing, inspecting and rendering a document) are not
  served: Book has a project, not a document, behind its tools.
- **`<file>`** is a Book project to open first. Without it the agent opens or makes one
  (`book_open`, `book_new`, `book_import`).

### Keeping an agent to folders

| Flag | Means |
| --- | --- |
| `--read-root <folder>` | The only folder the agent's commands may read files in |
| `--write-root <folder>` | The only folder the agent's commands may write files in |

- Each flag takes a folder that exists. A flag given alone grants only its own kind of access:
  with `--read-root` alone the agent can write nothing.
- With neither flag nothing is confined, and the program says so in one line on standard error
  when it starts.
- The folders hold for every file an agent names (the file to import, the folder to export
  to, a house file, a manuscript to link) and for what Book writes without being told where:
  the export folder beside a project, the new book beside an imported file.
- **The project you name on the command line is yours, not the agent's.** It is opened before
  the folders hold, and it is changed, saved and backed up where it lives, wherever that is.
  Its exports still go only where the agent may write.
- **A project the agent opens or makes is held to the folders.** It is read only inside the read
  folder, with the manuscript it links, and changed or saved only inside the write folder. So an
  agent that is to make a book needs a write folder that it can also read: one inside the read
  folder, or the same folder for both.
- A book in the read folder alone can be read, checked and exported into the write folder,
  and is never changed.
- Opening the ebook in a reading app (`book_ebook_openInReadingApp`) is refused while folders
  hold: a program Book starts cannot be kept to a folder.

```sh
# The agent may read the whole `books` folder and write only into its `out` folder.
wordsmith-book-cli mcp --read-root books --write-root books/out

# Your own book, wherever it is; the agent's files go into `out`.
wordsmith-book-cli mcp "My Novel.wsbk" --read-root books --write-root books/out
```
