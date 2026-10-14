# stories

## MILESTONE-01: Make it work

* [STORY-25: Stay inside a default size limit](STORY-25.md) - As an agent builder (PERSONA-01), I want jag to never write more characters than a default size limit, so that the output fits the context of my agent.
* [STORY-26: Leave out the lowest-ranked results](STORY-26.md) - As an agent builder (PERSONA-01), I want jag to leave out the lowest-ranked results and tell me how many, so that I know the list is not complete and how to see more.
* [STORY-27: Cut a top result that is larger than the limit](STORY-27.md) - As an agent builder (PERSONA-01), I want jag to cut the top result and say so when that result alone is larger than the limit, so that I still get the best answer.
* [STORY-28: Reject a size limit that is not valid](STORY-28.md) - As an agent builder (PERSONA-01), I want a limit of zero, a negative limit, or a value that is not a number to give a usage error, so that a typo does not hide the output.
* [STORY-29: Read text output by default](STORY-29.md) - As a shell power user (PERSONA-04), I want jag to write readable text by default, also in a pipe, with results on standard output and notices on the error stream, so that a pipe receives results only.
* [STORY-30: Turn on structured output](STORY-30.md) - As an agent builder (PERSONA-01), I want an explicit option for structured output with the same results as the text, so that my agent can parse the results.
* [STORY-31: Get a complete structured output with notices and errors](STORY-31.md) - As an agent builder (PERSONA-01), I want structured output to hold each notice and each error inside the structure, so that my agent parses every outcome in one form.
* [STORY-32: Rely on a stable structured form](STORY-32.md) - As an agent builder (PERSONA-01), I want the structured output to show its form version and to only add fields in one major version, so that my agent keeps working when jag updates.

## MILESTONE-02: Make it good

* [STORY-06: Check the five items of readiness](STORY-06.md) - As an agent builder (PERSONA-01), I want one command that shows pass or fail for five items and names the fix for each failure, so that I know what to repair before I give jag to an agent.
* [STORY-07: Use the check result as an exit code](STORY-07.md) - As an agent builder (PERSONA-01), I want the readiness check to exit with 0 when all items pass and with the code of the first cause when one fails, so that a script or an agent can act on the result.
* [STORY-08: Run the check for almost no cost](STORY-08.md) - As an agent builder (PERSONA-01), I want the readiness check to make one call that costs nothing or almost nothing, so that I can run the check as often as I want.
* [STORY-24: Label piped text](STORY-24.md) - As a shell power user (PERSONA-04), I want a result from piped text to have the label (standard input), so that I read a pipe result as a file result.
* [STORY-33: Search with a term and no question](STORY-33.md) - As a shell power user (PERSONA-04), I want jag with a term and no question to return the matching lines as grep does with no model call, so that a plain search costs nothing.
* [STORY-34: Use plain grep with no key](STORY-34.md) - As a shell power user (PERSONA-04), I want plain grep mode to work with no stored key, so that I can use jag before I set it up.
* [STORY-35: Use the common grep options](STORY-35.md) - As a shell power user (PERSONA-04), I want jag to accept the grep options that I use most, with their grep meaning, and -F to read the term as fixed text, so that I keep my habits.
* [STORY-36: Set the lines around a match](STORY-36.md) - As a shell power user (PERSONA-04), I want -A, -B, and -C to set the lines around each match in a question search, so that I choose how much context I read.
* [STORY-37: Get a usage error for an unknown option](STORY-37.md) - As a shell power user (PERSONA-04), I want any other option to give a usage error that names the option and exit code 2, so that a typo does not pass without a notice.
* [STORY-38: Keep the grep exit codes](STORY-38.md) - As a shell power user (PERSONA-04), I want the grep exit codes 0, 1, and 2, so that my scripts that test grep exit codes keep working.
* [STORY-39: Search with no path](STORY-39.md) - As a shell power user (PERSONA-04), I want jag with no path to search the current folder, or to read the pipe when text comes in through one, so that I use jag as I use grep.
* [STORY-40: Search a mix of files and folders](STORY-40.md) - As a shell power user (PERSONA-04), I want to give several paths, each a file or a folder, so that I search exactly the places that I choose.
* [STORY-41: See progress only in a terminal](STORY-41.md) - As a shell power user (PERSONA-04), I want a progress line on the error stream only when that stream is a terminal, so that a pipe or a log holds no progress noise.
* [STORY-42: Color the matches as grep does](STORY-42.md) - As a shell power user (PERSONA-04), I want jag to color the matching text only in a terminal, with --color=never and --color=always to change that, so that color works as grep --color=auto.
* [STORY-43: Stop at once when stopped](STORY-43.md) - As a shell power user (PERSONA-04), I want jag to stop at once on Ctrl-C or a closed pipe, so that a stopped search wastes no more credit.
* [STORY-45: Run the held-out evaluation before release](STORY-45.md) - As an agent builder (PERSONA-01), I want the held-out evaluation run done before release 1, so that I can trust that jag answers better than grep.
* [STORY-47: Check the full history before jag goes public](STORY-47.md) - As an agent builder (PERSONA-01), I want each commit in the history checked for private data before jag goes public, so that no old commit shows it.
* [STORY-48: Download a prebuilt binary from GitHub Releases](STORY-48.md) - As a shell power user (PERSONA-04), I want a prebuilt binary for my platform on GitHub Releases, so that I can install jag in seconds with no runtime.
* [STORY-49: Install jag with cargo from crates.io](STORY-49.md) - As a shell power user (PERSONA-04), I want to install jag with cargo install, so that I can build jag from source with the Rust tools I use.
* [STORY-50: Install or run jag with npm](STORY-50.md) - As an agent builder (PERSONA-01), I want to install jag with npm or run it with npx, so that my agent setup gets jag with the tools it already uses.
* [STORY-51: Publish each channel from one tag](STORY-51.md) - As an agent builder (PERSONA-01), I want one version tag to publish jag on each channel with the same version and no private data, so that each channel gives the same jag.
* [STORY-52: Create the new public repository with a clean history](STORY-52.md) - As an agent builder (PERSONA-01), I want a new public repository with a clean history, so that no public commit holds private data, and the private repository stays as it is.
* [STORY-53: See the cost after each search](STORY-53.md) - As a shell power user (PERSONA-04), I want a cost line after each search, so that I know what each search costs.
* [STORY-54: Read the cost in structured output](STORY-54.md) - As an agent builder (PERSONA-01), I want the cost as a field of structured output, so that my agent can track what its searches cost.
* [STORY-55: Know the cost when no figure is available](STORY-55.md) - As an agent builder (PERSONA-01), I want a clear cost report when jag makes no model call or the service gives no cost figure, so that the search still works.
* [STORY-56: Know the cost of a failed or stopped search](STORY-56.md) - As an agent builder (PERSONA-01), I want to know what a failed or stopped search cost, so that no cost is hidden from me.
* [STORY-57: Send the model calls through an AI gateway](STORY-57.md) - As an agent builder (PERSONA-01), I want to give jag the base URL and the model name of my AI gateway, so that jag uses the gateway that my team already uses.
* [STORY-58: Set the gateway as an option, a variable, or in the config file](STORY-58.md) - As an operator (PERSONA-02), I want to set the base URL and the model name as an option, a variable, or in the config file, so that I can change them for one search, one job, or each search.
* [STORY-59: Limit guesses through the check results of fork pull requests](STORY-59.md) - As an agent builder (PERSONA-01), I want a guard against guesses through the pass or fail of the private-data check on fork pull requests, so that no one can find private data by many tries.
* [STORY-60: Run the private-data check on fork pull requests in the new public repository](STORY-60.md) - As an agent builder (PERSONA-01), I want the private-data check to run on each fork pull request in the new public repository, so that no fork pull request skips the check.
* [STORY-61: Make the project board and the open issues public with the new repository](STORY-61.md) - As an agent builder (PERSONA-01), I want the board and the open issues of jag to become public with the new repository, so that outside contributors can see them.

## MILESTONE-03: Make it fast

* [STORY-44: Return a typical search in under 5 seconds](STORY-44.md) - As a shell power user (PERSONA-04), I want a typical search to return in under 5 seconds, so that I do not go back to grep.
