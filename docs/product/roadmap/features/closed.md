---
type: ClosedItems
title: Closed features
description: The features that are done or cancelled.
status: stable
generated:
  by: prd.py regen
---

# Closed features

## MILESTONE-01: Make it work

* [FEATURE-01: Store a key one time](FEATURE-01.md) - The user stores a TypeSafe key one time; jag finds it on each search, never shows it, and tells a new user where to get one.
* [FEATURE-04: Rank matching lines by a question](FEATURE-04.md) - The user runs `jag --ask "QUESTION" TERM [PATH...]`, and jag ranks the lines that match the term by how well they answer the question, and labels weak results.
* [FEATURE-05: Search a folder as grep -r does](FEATURE-05.md) - jag searches each subfolder of a folder, skips binary, hidden, and ignored files, and says how many files it skipped.
* [FEATURE-06: Cost limit on scoring](FEATURE-06.md) - When a term matches too many lines, jag scores only the best part up to a cost limit that the user can change, and says how many matches it skipped.
* [FEATURE-08: File, line, and snippet for each result](FEATURE-08.md) - Each result shows the file, the line range, and the exact source text, with each matching line marked, so the person or the agent can open the source at that line.
* [FEATURE-09: Size limit on the text that jag returns](FEATURE-09.md) - jag never writes more characters than the size limit, leaves out whole results with the lowest rank first, and says how many it left out and how to raise the limit.
* [FEATURE-11: Text output for a person and structured output for an agent](FEATURE-11.md) - An option turns on structured output that an agent can parse, with the same results as the text, a named field for each item, structured errors, and a form version.
* [FEATURE-17: Block private data in the repository](FEATURE-17.md) - A check on each pull request fails when the change adds a key, case content, private evaluation data, or an internal project name.

## MILESTONE-02: Make it good

* [FEATURE-02: Readiness check](FEATURE-02.md) - One command examines five items, shows pass or fail for each one, and names the fix for each failure, with one call that costs almost nothing.
* [FEATURE-03: Clear failure when the model is not available](FEATURE-03.md) - When the key is missing or out of credit, or the model service is down, jag stops with a message that names the cause and the fix, and exits with the code of that cause.
* [FEATURE-12: Plain grep with no question](FEATURE-12.md) - With a term and no question, jag returns the matching lines as grep does, makes no model call, and costs nothing.
* [FEATURE-15: Good behavior in a terminal and in a pipe](FEATURE-15.md) - jag shows progress and color only in a terminal, and it stops at once with no partial results when the user stops it or the pipe closes.
* [FEATURE-21: Send the model calls through an AI gateway](FEATURE-21.md) - The user can set a base URL and a model name, so that jag sends the same model calls through an AI gateway. With no setting, jag calls TypeSafe directly.
