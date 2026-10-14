---
type: ClosedItems
title: Closed stories
description: The stories that are done or cancelled.
status: stable
generated:
  by: prd.py regen
---

# Closed stories

## MILESTONE-01: Make it work

* [STORY-01: Store a key with setup](STORY-01.md) - As an agent builder (PERSONA-01), I want to store my TypeSafe key with one setup run, so that each later search finds the key with no more steps.
* [STORY-02: Find the key in a fixed order](STORY-02.md) - As an operator (PERSONA-02), I want jag to read the key from two variables and a config file in a fixed order, so that I can override the stored key for one shell or one job.
* [STORY-03: Replace or remove the stored key](STORY-03.md) - As an operator (PERSONA-02), I want setup to ask before it replaces a key and one command to remove the key, so that I never lose a key by accident and I can clean up a machine.
* [STORY-04: Keep the key private](STORY-04.md) - As an agent builder (PERSONA-01), I want jag to never show my key and to refuse a key file that others can read, so that my key stays secret on a shared machine.
* [STORY-05: Find where to get a key](STORY-05.md) - As an agent builder (PERSONA-01), I want the missing-key message to give the page where I get a TypeSafe key, so that I can start without a search for the sign-up page.
* [STORY-13: Rank matching lines by a question](STORY-13.md) - As a knowledge seeker (PERSONA-03), I want to run jag with a term, a question, and paths, and get the matching lines in order of how well they answer the question, so that the best answer comes first.
* [STORY-14: See the lines around a match](STORY-14.md) - As a knowledge seeker (PERSONA-03), I want each result to hold the matching line with the lines around it, so that I can read the answer in its context.
* [STORY-15: Know when no result answers well](STORY-15.md) - As an agent builder (PERSONA-01), I want jag to label the best results as weak and to exit with code 1 when no result answers the question well, so that my agent does not trust noise.
* [STORY-16: Search each subfolder of a folder](STORY-16.md) - As a shell power user (PERSONA-04), I want jag to search each file in a folder and in each subfolder, so that a folder search works as grep -r does.
* [STORY-17: Skip noise files and count them](STORY-17.md) - As a shell power user (PERSONA-04), I want jag to skip binary, hidden, and ignored files and to say how many files it skipped, so that I trust the search and see what it left out.
* [STORY-46: Fail a pull request that adds private data](STORY-46.md) - As an agent builder (PERSONA-01), I want each pull request checked for private data, so that the public repository never holds keys, case content, private evaluation data, or internal project names.

## MILESTONE-02: Make it good

* [STORY-09: Name the cause and the fix of a model failure](STORY-09.md) - As an agent builder (PERSONA-01), I want jag to stop with the cause and the fix when the model is not available, so that I know what to do next.
* [STORY-10: Tell each model failure by its exit code](STORY-10.md) - As an agent builder (PERSONA-01), I want each cause of a model failure to have its own exit code above 2, so that my agent can tell the causes apart and react to each one.
* [STORY-11: Try again one time before reporting the service as down](STORY-11.md) - As an operator (PERSONA-02), I want jag to wait a fixed time and try again one time before it reports the service as down, so that a short outage does not stop my search.
* [STORY-12: Fail the full search when the model stops during it](STORY-12.md) - As an agent builder (PERSONA-01), I want a search to show no partial result when the credit or the service stops during it, so that my agent never trusts a half-ranked list.
