# criteria

## STORY-01: Store a key with setup

* [CRITERION-01: Setup stores the key for later searches](CRITERION-01.md) - After the user runs setup one time, each later search finds the key with no more steps.
* [CRITERION-08: A TypeSafe key is enough to start](CRITERION-08.md) - A user with only a TypeSafe key, and no other service account, can complete setup and run a search.
* [CRITERION-72: Setup with no terminal fails clearly](CRITERION-72.md) - When setup runs with no terminal, so it cannot ask a question, it stores nothing, says why on the error stream, and exits with a code that is not 0.

## STORY-02: Find the key in a fixed order

* [CRITERION-02: jag finds the key in a fixed order](CRITERION-02.md) - jag reads `JAG_TYPESAFE_API_KEY`, then `TYPESAFE_API_KEY`, then `~/.config/jag/config.yml`, and the first key that it finds wins.

## STORY-03: Replace or remove the stored key

* [CRITERION-03: Setup asks before it replaces a key](CRITERION-03.md) - When a key is already stored, setup asks the user before it replaces that key.
* [CRITERION-04: One command removes the stored key](CRITERION-04.md) - The user removes the stored key with one command.

## STORY-04: Keep the key private

* [CRITERION-05: jag never shows the key](CRITERION-05.md) - No output, message, or error of jag shows the key.
* [CRITERION-06: jag refuses a key file that others can read](CRITERION-06.md) - When other users on the machine can read the key file, jag refuses the file and tells the user how to fix it.

## STORY-05: Find where to get a key

* [CRITERION-07: A missing key points to where to get one](CRITERION-07.md) - When jag finds no key, the message gives the page where the user gets a TypeSafe key.

## STORY-06: Check the five items of readiness

* [CRITERION-09: The check shows each of the five items](CRITERION-09.md) - The check shows pass or fail for each item: a key is present, the service accepts the key, the account has credit, the service is reachable, and other users cannot read the key file.
* [CRITERION-10: Each failure names its fix](CRITERION-10.md) - For each item that fails, the check names the problem and the step that fixes it.

## STORY-07: Use the check result as an exit code

* [CRITERION-11: jag is ready when all items pass](CRITERION-11.md) - When all five items pass, the check says that jag is ready and exits with code 0.
* [CRITERION-67: A failed check exits with the code of the first cause](CRITERION-67.md) - When an item of the readiness check fails, the check exits with the code of the first cause that fails.

## STORY-08: Run the check for almost no cost

* [CRITERION-12: The check costs almost nothing](CRITERION-12.md) - The check makes one live call to the model service, and that call costs nothing or almost nothing.

## STORY-09: Name the cause and the fix of a model failure

* [CRITERION-13: Each model failure names its cause and fix](CRITERION-13.md) - When the key is missing, the key is not valid, the credit is gone, or the service is down, the message names that cause and the fix.
* [CRITERION-15: jag never ranks without the model](CRITERION-15.md) - When the model is not available, jag returns no result that ignores the question.

## STORY-10: Tell each model failure by its exit code

* [CRITERION-14: Each model failure has its own exit code](CRITERION-14.md) - Each cause of a model failure has its own exit code above 2, and the codes 0, 1, and 2 keep their grep meaning.

## STORY-11: Try again one time before reporting the service as down

* [CRITERION-16: jag tries again one time before it reports the service as down](CRITERION-16.md) - Before jag reports the service as down, it waits a fixed time and tries again one time, and the user can change that wait.

## STORY-12: Fail the full search when the model stops during it

* [CRITERION-17: A failure during a search fails the full search](CRITERION-17.md) - When the credit or the service stops during a search, jag shows no partial result and exits with the code of the cause.

## STORY-13: Rank matching lines by a question

* [CRITERION-18: The command takes the term, the question, and the paths](CRITERION-18.md) - jag accepts the form `jag --ask "QUESTION" TERM [PATH...]`, with the term first as in grep.
* [CRITERION-19: The best answer comes first](CRITERION-19.md) - jag returns the results that match the term in order of how well they answer the question, with the best answer first.
* [CRITERION-20: The term is a regular expression](CRITERION-20.md) - jag reads the term as a regular expression, as grep does.
* [CRITERION-71: A term with no matches makes no model call](CRITERION-71.md) - When the term matches no line, jag makes no model call, writes no results, and exits with code 1, as grep does.
* [CRITERION-85: A ranked search says how many files it skipped](CRITERION-85.md) - When the user searches a folder with a term and a question, jag tells the user how many files it skipped.

## STORY-14: See the lines around a match

* [CRITERION-21: A result holds the lines around the match](CRITERION-21.md) - A result is the matching line with a few lines before and after it, and matches near each other join into one result.

## STORY-15: Know when no result answers well

* [CRITERION-29: Weak results have a clear label](CRITERION-29.md) - When no result answers the question well, jag shows the best results with a clear label that says no result answers well.
* [CRITERION-30: No good answer exits with code 1](CRITERION-30.md) - When no result answers the question well, jag exits with code 1.

## STORY-16: Search each subfolder of a folder

* [CRITERION-22: jag searches each subfolder](CRITERION-22.md) - When the user gives a folder, jag searches each file in that folder and in each subfolder.

## STORY-17: Skip noise files and count them

* [CRITERION-23: jag skips noise files](CRITERION-23.md) - jag skips binary files, hidden files, and ignored files.
* [CRITERION-24: The folder walk counts the files it skipped](CRITERION-24.md) - The folder walk gives the count of the files it skipped and the text of the notice that tells the user the count.

## STORY-18: Score only the best matches

* [CRITERION-25: jag scores only the best matches](CRITERION-25.md) - When the matches are more than the cost limit, jag pre-ranks them by word match and scores only the best ones, up to the limit.
* [CRITERION-27: jag says how many matches it skipped](CRITERION-27.md) - When jag reaches the cost limit, it returns the ranked results and says how many matches it skipped.

## STORY-19: Change the cost limit

* [CRITERION-26: The cost limit has a default that the user can change](CRITERION-26.md) - jag has a fixed default cost limit, stated as a count of results to score, and the user can change it with an option.

## STORY-20: Tell a reached cost limit by its exit code

* [CRITERION-28: Reaching the cost limit has its own exit code](CRITERION-28.md) - When jag reaches the cost limit, it exits with its own exit code, and exit code 1 wins when nothing answers well.

## STORY-21: See the file, the line, and the snippet of a result

* [CRITERION-31: Each result shows the file, the line, and the snippet](CRITERION-31.md) - Each result shows the file, the line number, and the snippet.
* [CRITERION-32: The path has the same form as in grep](CRITERION-32.md) - The path of each result follows the form of the path that the user gave, as in grep.

## STORY-22: See the line range of a joined result

* [CRITERION-33: A joined result shows its line range](CRITERION-33.md) - A result gives its first and last line, and it marks each line that matches the term.

## STORY-23: Cut a very long line and mark the cut

* [CRITERION-34: jag cuts a very long line and marks the cut](CRITERION-34.md) - A result shows the exact source text, and jag cuts each line that is longer than a fixed length and marks the cut.

## STORY-24: Label piped text

* [CRITERION-35: Piped text has the label (standard input)](CRITERION-35.md) - A result from piped text has the label `(standard input)` and the line numbers of the piped text.

## STORY-25: Stay inside a default size limit

* [CRITERION-36: jag never goes over the size limit](CRITERION-36.md) - The size limit is a count of characters, and jag never writes more characters to standard output than the limit.
* [CRITERION-37: A default size limit always applies](CRITERION-37.md) - jag always applies a default size limit, and the user can raise it or turn it off.
* [CRITERION-38: All text on standard output counts](CRITERION-38.md) - Labels, the structured wrapper, and notices on standard output count toward the limit, and text on the error stream does not.

## STORY-26: Leave out the lowest-ranked results

* [CRITERION-39: jag leaves out whole results with the lowest rank first](CRITERION-39.md) - To stay inside the limit, jag leaves out whole results, starting with the lowest rank.
* [CRITERION-41: The notice gives the count and how to raise the limit](CRITERION-41.md) - When jag leaves out results, a notice gives their number and tells the user how to raise the limit.
* [CRITERION-42: Leaving out results has its own exit code](CRITERION-42.md) - When jag leaves out results to stay inside the limit, it exits with its own exit code above 2.
* [CRITERION-66: Cost-limit code wins when both limits apply](CRITERION-66.md) - When a search reaches the cost limit and also leaves out results, jag exits with the cost-limit code, and the notice says that results were left out.
* [CRITERION-104: The size-limit notice is named fields in structured output](CRITERION-104.md) - When jag leaves out results or cuts the top result, structured output gives the size-limit notice as named fields, and the error stream holds no size-limit notice.

## STORY-27: Cut a top result that is larger than the limit

* [CRITERION-40: jag cuts a top result that is larger than the limit](CRITERION-40.md) - When the top result alone is larger than the limit, jag cuts it, marks the cut, and says so.

## STORY-28: Reject a size limit that is not valid

* [CRITERION-43: A size limit that is not valid is a usage error](CRITERION-43.md) - A limit of zero, a negative limit, or a value that is not a number gives a usage error and exit code 2.

## STORY-29: Read text output by default

* [CRITERION-44: Text is the default output](CRITERION-44.md) - jag writes readable text when the user gives no output option, also when the output goes to a pipe.
* [CRITERION-45: Notices go to the error stream](CRITERION-45.md) - In text mode, jag writes results to standard output and each notice to the error stream.

## STORY-30: Turn on structured output

* [CRITERION-46: Only an explicit option turns on structured output](CRITERION-46.md) - jag writes structured output only when the user gives the structured output option.
* [CRITERION-47: Each item of the text is a named field](CRITERION-47.md) - Structured output holds the same results as the text, and each label, count, mark, and notice of the text is a named field.
* [CRITERION-48: Each result has its rank and no score](CRITERION-48.md) - Each result in structured output has its rank, and it has no score.

## STORY-31: Get a complete structured output with notices and errors

* [CRITERION-49: Structured mode always writes a complete output](CRITERION-49.md) - With no results, structured output holds an empty result list and the summary fields.
* [CRITERION-50: Notices are inside the structure](CRITERION-50.md) - In structured mode, jag writes each notice inside the structure.
* [CRITERION-51: Errors are structured](CRITERION-51.md) - In structured mode, an error is structured and gives the cause, the fix, and the exit code.

## STORY-32: Rely on a stable structured form

* [CRITERION-52: The form only adds fields in one major version](CRITERION-52.md) - The structured output shows its form version, and in one major version jag never removes or renames a field.

## STORY-33: Search with a term and no question

* [CRITERION-53: A term with no question works as grep](CRITERION-53.md) - With a term and no question, jag returns the matching lines as grep does.
* [CRITERION-54: A search with no question costs nothing](CRITERION-54.md) - With no question, jag makes no model call and the search costs nothing.

## STORY-34: Use plain grep with no key

* [CRITERION-68: Plain grep works with no key](CRITERION-68.md) - With no stored key, `jag TERM PATH`, with no `--ask`, returns the grep matches and does not ask for a key.

## STORY-35: Use the common grep options

* [CRITERION-55: jag keeps the common grep options](CRITERION-55.md) - jag accepts `-i`, `-w`, `-F`, `-e`, `-A`, `-B`, `-C`, `-l`, `-c`, `-q`, `--include`, and `--exclude`, with their grep meaning.
* [CRITERION-56: The -F option makes the term fixed text](CRITERION-56.md) - With `-F`, jag reads the term as fixed text and not as a regular expression.

## STORY-36: Set the lines around a match

* [CRITERION-69: Context options set the lines around a match](CRITERION-69.md) - `-A`, `-B`, and `-C` set the lines around each match in a question search. With none of them, a result has the default context.

## STORY-37: Get a usage error for an unknown option

* [CRITERION-57: An unknown option is a usage error](CRITERION-57.md) - Any other option gives a usage error that names the option, and jag exits with code 2.

## STORY-38: Keep the grep exit codes

* [CRITERION-58: jag keeps the grep exit codes](CRITERION-58.md) - jag exits with code 0 when it returns a result, code 1 when it finds no match, and code 2 for a usage error, as grep does.

## STORY-39: Search with no path

* [CRITERION-59: With no path, jag searches the current folder](CRITERION-59.md) - When the user gives no path and no text comes in through a pipe, jag searches the current folder.
* [CRITERION-60: With no path, jag reads the pipe](CRITERION-60.md) - When the user gives no path and text comes in through a pipe, jag searches the piped text.

## STORY-40: Search a mix of files and folders

* [CRITERION-61: The user can give any mix of files and folders](CRITERION-61.md) - jag accepts several paths, and each path can be a file or a folder.
* [CRITERION-70: A path that does not exist is an error](CRITERION-70.md) - When a path does not exist, jag names the path on the error stream and exits with code 2, as grep does.

## STORY-41: See progress only in a terminal

* [CRITERION-62: jag shows progress only in a terminal](CRITERION-62.md) - While jag ranks, it shows a progress line on the error stream only when that stream is a terminal.

## STORY-42: Color the matches as grep does

* [CRITERION-63: Color works as grep --color=auto](CRITERION-63.md) - jag colors the matching text only in a terminal, and `--color=never` and `--color=always` change that.

## STORY-43: Stop at once when stopped

* [CRITERION-64: jag stops at once when the user stops it](CRITERION-64.md) - On Ctrl-C or a closed pipe, jag stops at once, starts no new model call, and shows no partial results.

## STORY-44: Return a typical search in under 5 seconds

* [CRITERION-65: A typical search returns in under 5 seconds](CRITERION-65.md) - One search at the default cost limit, on a medium code repository, with a working service, returns in under 5 seconds.

## STORY-45: Run the held-out evaluation before release

* [CRITERION-73: The held-out score meets the target and beats grep](CRITERION-73.md) - The held-out run gives jag an ndcg@10 of 0.70 or more, which is higher than blind grep. The public summary shows only the numbers and no case content.

## STORY-46: Fail a pull request that adds private data

* [CRITERION-74: A pull request that adds private data fails the check](CRITERION-74.md) - A pull request that adds a key, case content, private evaluation data, or an internal project name fails the check, and the pull request cannot merge.
* [CRITERION-75: The check names the place and never shows the value](CRITERION-75.md) - For each finding, the check names the file and the line, and it never shows the found value in its output.

## STORY-47: Check the full history before jag goes public

* [CRITERION-76: The history scan lists each finding and never shows the value](CRITERION-76.md) - The scan finds each key, case content, private evaluation data, and internal project name in each commit, names its commit, file, and line, and never shows the value.

## STORY-48: Download a prebuilt binary from GitHub Releases

* [CRITERION-77: Each release has one archive for each platform with a checksum](CRITERION-77.md) - Each release on GitHub Releases has one archive for Linux x86_64, Linux arm64, macOS arm64, and macOS x86_64, and each archive has a checksum.
* [CRITERION-78: The binary runs with no runtime](CRITERION-78.md) - The binary from the archive runs on its platform with no other runtime or program installed.

## STORY-49: Install jag with cargo from crates.io

* [CRITERION-79: cargo install gives the release version](CRITERION-79.md) - cargo install jag-grep installs the jag command, with the same version as the GitHub Release.

## STORY-50: Install or run jag with npm

* [CRITERION-80: npx runs jag on each platform](CRITERION-80.md) - npx @airf/jag runs jag on each of the four platforms, and the install downloads nothing from GitHub.
* [CRITERION-81: npm installs only the binary of the machine](CRITERION-81.md) - npm installs only the binary package that matches the platform of the machine.

## STORY-51: Publish each channel from one tag

* [CRITERION-82: One tag publishes each channel with the same version](CRITERION-82.md) - One version tag publishes the GitHub Release, the crate, and the npm packages, and each one has the same version.
* [CRITERION-83: The repository stores no publish token](CRITERION-83.md) - The release publishes to crates.io and npm with trusted publishing, and the repository stores no publish token.
* [CRITERION-84: The published packages hold no private data](CRITERION-84.md) - The archives, the crate, and the npm packages hold no key, no case content, no private evaluation data, and no internal project name.

## STORY-52: Create the new public repository with a clean history

* [CRITERION-86: The new public repository gets one clean history, after the scan](CRITERION-86.md) - After the history scan lists each finding, one new history that has none of the findings is made for the new public repository. The new history is not made before the scan.
* [CRITERION-87: A second scan is clean before the new repository becomes public](CRITERION-87.md) - Before the new repository becomes public, a second scan of each of its refs finds no key, no case content, no private evaluation data, and no internal project name.
* [CRITERION-102: This repository stays private](CRITERION-102.md) - After the new public repository is created, ai-ready-future/jag stays private, and its history is not rewritten.

## STORY-53: See the cost after each search

* [CRITERION-88: Text mode writes a cost line to the error stream](CRITERION-88.md) - In text mode, jag writes one cost line to the error stream after each search, also when the output goes to a pipe.
* [CRITERION-89: The cost line counts the results scored and skipped](CRITERION-89.md) - The cost line shows the count of results that jag scored, the count that the cost limit skipped, and the billed amount in US dollars.

## STORY-54: Read the cost in structured output

* [CRITERION-90: Structured output always has a cost field](CRITERION-90.md) - In structured mode, the output always has a cost field with the same counts and amount as the text cost line.

## STORY-55: Know the cost when no figure is available

* [CRITERION-91: A search with no model call reports a cost of zero](CRITERION-91.md) - When jag makes no model call, it reports a cost of zero and the reason.
* [CRITERION-92: A missing cost figure never fails the search](CRITERION-92.md) - When the service gives no cost figure and jag cannot estimate one, jag shows the count, marks the cost as not available, and the search does not fail.
* [CRITERION-97: An estimated cost has a label](CRITERION-97.md) - When the service gives no billed amount, jag shows an estimate in US dollars and labels it as an estimate.

## STORY-56: Know the cost of a failed or stopped search

* [CRITERION-93: A failed or stopped search reports the cost up to that point](CRITERION-93.md) - When a search fails or the user stops it, the error gives the cost that jag paid up to that point. In structured mode, the cost is a field of the error.

## STORY-57: Send the model calls through an AI gateway

* [CRITERION-94: jag sends the calls to the gateway that the user names](CRITERION-94.md) - When the user sets a base URL and a model name, jag sends the same model calls to that address with that model name.
* [CRITERION-95: A TypeSafe key alone is enough](CRITERION-95.md) - When the user sets no base URL and no model name, jag calls TypeSafe directly and needs only the TypeSafe key.

## STORY-58: Set the gateway as an option, a variable, or in the config file

* [CRITERION-96: An option wins over a variable, and a variable wins over the config file](CRITERION-96.md) - For the base URL and the model name, the command option wins over the variable, the variable wins over the config file, and the config file wins over the default.

## STORY-59: Limit guesses through the check results of fork pull requests

* [CRITERION-98: A guard against guesses is in place before the new repository becomes public](CRITERION-98.md) - Before the new repository becomes public, a guard such as an approval step or a rate limit stops fast guesses through the check results of fork pull requests.

## STORY-60: Run the private-data check on fork pull requests in the new public repository

* [CRITERION-99: The Actions event policy lets the check run on fork pull requests](CRITERION-99.md) - Before the new repository becomes public, its Actions event policy allows pull_request_target for the private-data check, and a test fork pull request shows that the check runs.

## STORY-61: Make the project board and the open issues public with the new repository

* [CRITERION-100: The project board becomes public after the new repository](CRITERION-100.md) - After the new repository becomes public, the Project is linked to it and becomes public. Until then, it is private.
* [CRITERION-101: The project board holds no private data when it becomes public](CRITERION-101.md) - Before the new repository and the Project become public, the items, the issues, and their comments hold no private data.
* [CRITERION-103: The open issues move to the new repository while it is private](CRITERION-103.md) - While the new repository is private, each open issue moves to it with gh issue transfer. The same labels and milestones exist in it first.
