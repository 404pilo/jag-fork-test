# features

## MILESTONE-01: Make it work

* [FEATURE-08: File, line, and snippet for each result](FEATURE-08.md) - Each result shows the file, the line range, and the exact source text, with each matching line marked, so the person or the agent can open the source at that line.
* [FEATURE-09: Size limit on the text that jag returns](FEATURE-09.md) - jag never writes more characters than the size limit, leaves out whole results with the lowest rank first, and says how many it left out and how to raise the limit.
* [FEATURE-11: Text output for a person and structured output for an agent](FEATURE-11.md) - An option turns on structured output that an agent can parse, with the same results as the text, a named field for each item, structured errors, and a form version.

## MILESTONE-02: Make it good

* [FEATURE-02: Readiness check](FEATURE-02.md) - One command examines five items, shows pass or fail for each one, and names the fix for each failure, with one call that costs almost nothing.
* [FEATURE-12: Plain grep with no question](FEATURE-12.md) - With a term and no question, jag returns the matching lines as grep does, makes no model call, and costs nothing.
* [FEATURE-13: Common grep options and exit codes](FEATURE-13.md) - jag keeps the grep options that people use most and the grep exit codes 0, 1, and 2; any other option is a usage error that names it.
* [FEATURE-14: Files, folders, and piped text](FEATURE-14.md) - With no path, jag searches the current folder or reads the pipe; the user can also give any mix of files and folders.
* [FEATURE-15: Good behavior in a terminal and in a pipe](FEATURE-15.md) - jag shows progress and color only in a terminal, and it stops at once with no partial results when the user stops it or the pipe closes.
* [FEATURE-18: Install jag from GitHub Releases, crates.io, or npm](FEATURE-18.md) - One version tag builds the jag binary for four platforms and publishes it on GitHub Releases, crates.io, and npm with the same version (ADR-0010).
* [FEATURE-19: Go public with a clean history](FEATURE-19.md) - A scan lists the private data in the history. A new public repository gets a clean history, and a second scan finds none. This repository stays private.
* [FEATURE-20: Show the cost of a search](FEATURE-20.md) - After each search, jag reports how many results it scored and skipped, and the billed cost in US dollars, as a text line or a structured field.
* [FEATURE-21: Send the model calls through an AI gateway](FEATURE-21.md) - The user can set a base URL and a model name, so that jag sends the same model calls through an AI gateway. With no setting, jag calls TypeSafe directly.

## MILESTONE-03: Make it fast

* [FEATURE-16: Fast enough to replace grep](FEATURE-16.md) - A typical search returns in under 5 seconds, so that a shell user does not go back to grep.
