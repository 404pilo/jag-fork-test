# Components

* [COMPONENT-01: Command line](COMPONENT-01.md) - Reads the command and its options, picks text or structured mode, runs the pipeline, and sets the exit code.
* [COMPONENT-02: Key and settings](COMPONENT-02.md) - Finds the key for the route of each call and the settings, reads the config file at most one time in each run, checks the mode of the file at each read, and stores or removes the key with no loss of the other keys of the file. The one component that reads the key.
* [COMPONENT-03: Match search](COMPONENT-03.md) - Uses the ripgrep libraries to find the matches of the term in the files or the piped text, skips noise files, and joins near matches into results with their lines around them.
* [COMPONENT-04: Pre-rank](COMPONENT-04.md) - Orders the results by BM25 on the question words, so the scorer gets the best 200.
* [COMPONENT-05: Jev scorer](COMPONENT-05.md) - Sends the results to Jev in calls of 32, with 6 calls in parallel, under the cost limit. Owns the scoring prompt, the retry, the no-good-answer threshold, and the cost count.
* [COMPONENT-06: Output](COMPONENT-06.md) - Writes the ranked results as text or as structured output, applies the size limit, and writes the notices and the cost line.
* [COMPONENT-07: Readiness check](COMPONENT-07.md) - Runs the five checks of DECISION-04 with one free call, and gives the exit code of the first cause that fails.
* [COMPONENT-08: jag-bench](COMPONENT-08.md) - Runs a set of questions with their expected answers through jag and through plain grep, and reports which one finds the answer more often.
* [COMPONENT-09: TypeSafe API](COMPONENT-09.md) - Scores snippets with the Jev model and reports the billed usage.
* [COMPONENT-10: AI gateway](COMPONENT-10.md) - An optional route to the TypeSafe API, with the same calls.
* [COMPONENT-11: Agent skill](COMPONENT-11.md) - The file skills/jag/SKILL.md, the one source of the agent instructions. The skills tool installs it, and the README snippet copies its body.
