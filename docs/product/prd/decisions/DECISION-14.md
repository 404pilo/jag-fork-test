---
type: Decision
title: A folder search works like grep -r and skips noise files
description: jag searches each subfolder, skips binary, hidden, and ignored files,
  and says how many files it skipped.
status: stable
decided_at: '2026-10-06'
alternatives:
- Search every file in every subfolder and skip only binary files
- Search only the top level of the folder unless the user asks for subfolders
about:
- /capabilities/CAPABILITY-02.md
---

# DECISION-14: A folder search works like grep -r and skips noise files

jag searches each subfolder of the folder, as `grep -r` does. It skips binary files, hidden files, and files that the ignore file of the folder names. It says how many files it skipped.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- subject of: [DECISION-36: The user can give any mix of files and folders](DECISION-36.md)
<!-- LINKS:END -->
