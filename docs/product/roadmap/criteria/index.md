# criteria

## FEATURE-01: Store a key one time

* [CRITERION-01: Setup stores the key for later searches](CRITERION-01.md) - After the user runs setup one time, each later search finds the key with no more steps.
* [CRITERION-02: jag finds the key in a fixed order](CRITERION-02.md) - jag reads `JAG_TYPESAFE_API_KEY`, then `TYPESAFE_API_KEY`, then `~/.config/jag/config.yml`, and the first key that it finds wins.
* [CRITERION-03: Setup asks before it replaces a key](CRITERION-03.md) - When a key is already stored, setup asks the user before it replaces that key.
* [CRITERION-04: One command removes the stored key](CRITERION-04.md) - The user removes the stored key with one command.
* [CRITERION-05: jag never shows the key](CRITERION-05.md) - No output, message, or error of jag shows the key.
* [CRITERION-06: jag refuses a key file that others can read](CRITERION-06.md) - When other users on the machine can read the key file, jag refuses the file and tells the user how to fix it.
* [CRITERION-07: A missing key points to where to get one](CRITERION-07.md) - When jag finds no key, the message gives the page where the user gets a TypeSafe key.
* [CRITERION-08: A TypeSafe key is enough to start](CRITERION-08.md) - A user with only a TypeSafe key, and no other service account, can complete setup and run a search.

## FEATURE-02: Readiness check

* [CRITERION-09: The check shows each of the five items](CRITERION-09.md) - The check shows pass or fail for each item: a key is present, the service accepts the key, the account has credit, the service is reachable, and other users cannot read the key file.
* [CRITERION-10: Each failure names its fix](CRITERION-10.md) - For each item that fails, the check names the problem and the step that fixes it.
* [CRITERION-11: jag is ready when all items pass](CRITERION-11.md) - When all five items pass, the check says that jag is ready and exits with code 0.
* [CRITERION-12: The check costs almost nothing](CRITERION-12.md) - The check makes one live call to the model service, and that call costs nothing or almost nothing.
* [CRITERION-67: A failed check exits with the code of the first cause](CRITERION-67.md) - When an item of the readiness check fails, the check exits with the code of the first cause that fails.

## FEATURE-03: Clear failure when the model is not available

* [CRITERION-13: Each model failure names its cause and fix](CRITERION-13.md) - When the key is missing, the key is not valid, the credit is gone, or the service is down, the message names that cause and the fix.
* [CRITERION-14: Each model failure has its own exit code](CRITERION-14.md) - Each cause of a model failure has its own exit code above 2, and the codes 0, 1, and 2 keep their grep meaning.
* [CRITERION-15: jag never ranks without the model](CRITERION-15.md) - When the model is not available, jag returns no result that ignores the question.
* [CRITERION-16: jag tries again one time before it reports the service as down](CRITERION-16.md) - Before jag reports the service as down, it waits a fixed time and tries again one time, and the user can change that wait.
* [CRITERION-17: A failure during a search fails the full search](CRITERION-17.md) - When the credit or the service stops during a search, jag shows no partial result and exits with the code of the cause.

## FEATURE-04: Rank matching lines by a question

* [CRITERION-18: The command takes the term, the question, and the paths](CRITERION-18.md) - jag accepts the form `jag TERM "QUESTION" [PATH...]`, with the term first as in grep.
* [CRITERION-19: The best answer comes first](CRITERION-19.md) - jag returns the results that match the term in order of how well they answer the question, with the best answer first.
* [CRITERION-20: The term is a regular expression](CRITERION-20.md) - jag reads the term as a regular expression, as grep does.
* [CRITERION-21: A result holds the lines around the match](CRITERION-21.md) - A result is the matching line with a few lines before and after it, and matches near each other join into one result.
* [CRITERION-29: Weak results have a clear label](CRITERION-29.md) - When no result answers the question well, jag shows the best results with a clear label that says no result answers well.
* [CRITERION-30: No good answer exits with code 1](CRITERION-30.md) - When no result answers the question well, jag exits with code 1.

## FEATURE-05: Search a folder as grep -r does

* [CRITERION-22: jag searches each subfolder](CRITERION-22.md) - When the user gives a folder, jag searches each file in that folder and in each subfolder.
* [CRITERION-23: jag skips noise files](CRITERION-23.md) - jag skips binary files, hidden files, and ignored files.
* [CRITERION-24: jag says how many files it skipped](CRITERION-24.md) - jag tells the user how many files it skipped.

## FEATURE-06: Cost limit on scoring

* [CRITERION-25: jag scores only the best matches](CRITERION-25.md) - When the matches are more than the cost limit, jag pre-ranks them by word match and scores only the best ones, up to the limit.
* [CRITERION-26: The cost limit has a default that the user can change](CRITERION-26.md) - jag has a fixed default cost limit, stated as a count of results to score, and the user can change it with an option.
* [CRITERION-27: jag says how many matches it skipped](CRITERION-27.md) - When jag reaches the cost limit, it returns the ranked results and says how many matches it skipped.
* [CRITERION-28: Reaching the cost limit has its own exit code](CRITERION-28.md) - When jag reaches the cost limit, it exits with its own exit code, and exit code 1 wins when nothing answers well.
* [CRITERION-66: Cost-limit code wins when both limits apply](CRITERION-66.md) - When a search reaches the cost limit and also leaves out results, jag exits with the cost-limit code, and the notice says that results were left out.

## FEATURE-08: File, line, and snippet for each result

* [CRITERION-31: Each result shows the file, the line, and the snippet](CRITERION-31.md) - Each result shows the file, the line number, and the snippet.
* [CRITERION-32: The path has the same form as in grep](CRITERION-32.md) - The path of each result follows the form of the path that the user gave, as in grep.
* [CRITERION-33: A joined result shows its line range](CRITERION-33.md) - A result gives its first and last line, and it marks each line that matches the term.
* [CRITERION-34: jag cuts a very long line and marks the cut](CRITERION-34.md) - A result shows the exact source text, and jag cuts each line that is longer than a fixed length and marks the cut.
* [CRITERION-35: Piped text has the label (standard input)](CRITERION-35.md) - A result from piped text has the label `(standard input)` and the line numbers of the piped text.

## FEATURE-09: Size limit on the text that jag returns

* [CRITERION-36: jag never goes over the size limit](CRITERION-36.md) - The size limit is a count of characters, and jag never writes more characters to standard output than the limit.
* [CRITERION-37: A default size limit always applies](CRITERION-37.md) - jag always applies a default size limit, and the user can raise it or turn it off.
* [CRITERION-38: All text on standard output counts](CRITERION-38.md) - Labels, the structured wrapper, and notices on standard output count toward the limit, and text on the error stream does not.
* [CRITERION-39: jag leaves out whole results with the lowest rank first](CRITERION-39.md) - To stay inside the limit, jag leaves out whole results, starting with the lowest rank.
* [CRITERION-40: jag cuts a top result that is larger than the limit](CRITERION-40.md) - When the top result alone is larger than the limit, jag cuts it, marks the cut, and says so.
* [CRITERION-41: The notice gives the count and how to raise the limit](CRITERION-41.md) - When jag leaves out results, a notice gives their number and tells the user how to raise the limit.
* [CRITERION-42: Leaving out results has its own exit code](CRITERION-42.md) - When jag leaves out results to stay inside the limit, it exits with its own exit code above 2.
* [CRITERION-43: A size limit that is not valid is a usage error](CRITERION-43.md) - A limit of zero, a negative limit, or a value that is not a number gives a usage error and exit code 2.

## FEATURE-11: Text output for a person and structured output for an agent

* [CRITERION-44: Text is the default output](CRITERION-44.md) - jag writes readable text when the user gives no output option, also when the output goes to a pipe.
* [CRITERION-45: Notices go to the error stream](CRITERION-45.md) - In text mode, jag writes results to standard output and each notice to the error stream.
* [CRITERION-46: Only an explicit option turns on structured output](CRITERION-46.md) - jag writes structured output only when the user gives the structured output option.
* [CRITERION-47: Each item of the text is a named field](CRITERION-47.md) - Structured output holds the same results as the text, and each label, count, mark, and notice of the text is a named field.
* [CRITERION-48: Each result has its rank and no score](CRITERION-48.md) - Each result in structured output has its rank, and it has no score.
* [CRITERION-49: Structured mode always writes a complete output](CRITERION-49.md) - With no results, structured output holds an empty result list and the summary fields.
* [CRITERION-50: Notices are inside the structure](CRITERION-50.md) - In structured mode, jag writes each notice inside the structure.
* [CRITERION-51: Errors are structured](CRITERION-51.md) - In structured mode, an error is structured and gives the cause, the fix, and the exit code.
* [CRITERION-52: The form only adds fields in one major version](CRITERION-52.md) - The structured output shows its form version, and in one major version jag never removes or renames a field.

## FEATURE-12: Plain grep with no question

* [CRITERION-53: A term with no question works as grep](CRITERION-53.md) - With a term and no question, jag returns the matching lines as grep does.
* [CRITERION-54: A search with no question costs nothing](CRITERION-54.md) - With no question, jag makes no model call and the search costs nothing.
* [CRITERION-68: Plain grep works with no key](CRITERION-68.md) - With no stored key, `jag TERM PATH` returns the grep matches and does not ask for a key.

## FEATURE-13: Common grep options and exit codes

* [CRITERION-55: jag keeps the common grep options](CRITERION-55.md) - jag accepts `-i`, `-w`, `-F`, `-e`, `-A`, `-B`, `-C`, `-l`, `-c`, `-q`, `--include`, and `--exclude`, with their grep meaning.
* [CRITERION-56: The -F option makes the term fixed text](CRITERION-56.md) - With `-F`, jag reads the term as fixed text and not as a regular expression.
* [CRITERION-57: An unknown option is a usage error](CRITERION-57.md) - Any other option gives a usage error that names the option, and jag exits with code 2.
* [CRITERION-58: jag keeps the grep exit codes](CRITERION-58.md) - jag exits with code 0 when it returns a result, code 1 when it finds no match, and code 2 for a usage error, as grep does.
* [CRITERION-69: Context options set the lines around a match](CRITERION-69.md) - `-A`, `-B`, and `-C` set the lines around each match in a question search. With none of them, a result has the default context.

## FEATURE-14: Files, folders, and piped text

* [CRITERION-59: With no path, jag searches the current folder](CRITERION-59.md) - When the user gives no path and no text comes in through a pipe, jag searches the current folder.
* [CRITERION-60: With no path, jag reads the pipe](CRITERION-60.md) - When the user gives no path and text comes in through a pipe, jag searches the piped text.
* [CRITERION-61: The user can give any mix of files and folders](CRITERION-61.md) - jag accepts several paths, and each path can be a file or a folder.

## FEATURE-15: Good behavior in a terminal and in a pipe

* [CRITERION-62: jag shows progress only in a terminal](CRITERION-62.md) - While jag ranks, it shows a progress line on the error stream only when that stream is a terminal.
* [CRITERION-63: Color works as grep --color=auto](CRITERION-63.md) - jag colors the matching text only in a terminal, and `--color=never` and `--color=always` change that.
* [CRITERION-64: jag stops at once when the user stops it](CRITERION-64.md) - On Ctrl-C or a closed pipe, jag stops at once, starts no new model call, and shows no partial results.

## FEATURE-16: Fast enough to replace grep

* [CRITERION-65: A typical search returns in under 5 seconds](CRITERION-65.md) - One search at the default cost limit, on a medium code repository, with a working service, returns in under 5 seconds.
