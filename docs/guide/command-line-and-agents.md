# Command line and agents

**The command line (AUT-1).** `book-cli` runs the same code as the window:

| Command | What it does |
| --- | --- |
| `build <project> [--edition <id>]... [--out <folder>] [--bundle]` | Build, check and write editions (all when none is named) |
| `check <project> [--edition <id>]...` | Preflight, with each finding's page or section |
| `verify <project> <file>` | Check a PDF (letter by letter), an EPUB or a source document (word by word) against the book |
| `convert <file> <project folder> [--title <title>]` | Import a file into a new Book project |
| `report <project>` | Title, author, design, sections, words and details, as JSON |
| `run <project> <command id> [<JSON>]` | Run any Book command; changes are saved |
| `commands` | Every command and its label |
| `mcp [<project>]` | Serve Book's commands to an agent over stdio |

Exit codes: **0** done, **1** a check failed or a command refused (nothing was written), **2** the
command line was wrong or a file could not be opened.

**Agent tools (AUT-2).** `book-cli mcp` serves every Book command as a tool over the Model
Context Protocol: `book_import`, `book_design_choose`, `book_preflight`, `book_export`,
`book_save` and the rest, each described with its parameters. An agent can import, design, build
and check a book with the tools alone. A linked Manuscript project is never changed.
