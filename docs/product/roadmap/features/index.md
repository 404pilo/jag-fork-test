# features

## MILESTONE-02: Make it good

* [FEATURE-14: Files, folders, and piped text](FEATURE-14.md) - With no path, jag searches the current folder or reads the pipe; the user can also give any mix of files and folders.
* [FEATURE-18: Install jag from GitHub Releases, crates.io, or npm](FEATURE-18.md) - One version tag builds the jag binary for four platforms and publishes it on GitHub Releases, crates.io, and npm with the same version (ADR-0010).
* [FEATURE-19: Go public with a clean history](FEATURE-19.md) - A scan lists the private data in the history. A new public repository gets a clean history, and a second scan finds none. The private repository stays private.
* [FEATURE-20: Show the cost of a search](FEATURE-20.md) - After each search, jag reports how many results it scored and skipped, and the billed cost in US dollars, as a text line or a structured field.

## MILESTONE-03: Make it fast

* [FEATURE-16: Fast enough to replace grep](FEATURE-16.md) - A typical search returns in under 5 seconds, so that a shell user does not go back to grep.
