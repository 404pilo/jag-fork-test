# decisions

* [DECISION-01: jag fails clearly when the model is not available](DECISION-01.md) - When the key is missing or out of credit, or the service is down, jag fails with a clear message and does not fall back to word-match ranking.
* [DECISION-02: Release 1 does not serve users whose text must stay on the machine](DECISION-02.md) - Release 1 needs the model service; users whose text must not leave the machine wait for a later local model option.
* [DECISION-03: jag reads the key from two variables and a config file, in a fixed order](DECISION-03.md) - jag reads JAG_TYPESAFE_API_KEY, then TYPESAFE_API_KEY, then ~/.config/jag/config.yml; the first key found wins.
* [DECISION-04: The health check examines five items with one free call](DECISION-04.md) - The health check examines key present, key valid, credit left, service reachable, and key file safe, with one call that costs almost nothing.
* [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](DECISION-05.md) - jag keeps the grep exit codes 0, 1, and 2, and gives each model failure its own code above 2.
* [DECISION-06: A model failure during a search fails the full search](DECISION-06.md) - When the credit or the service stops during a search, jag returns no partial result and exits with the code of the cause.
* [DECISION-07: Errors are structured in structured output mode](DECISION-07.md) - In structured output mode, an error is structured, with the cause, the fix, and the exit code; in text mode, it is readable text.
* [DECISION-08: Setup asks before it replaces a key, and one command removes it](DECISION-08.md) - Setup asks the user before it replaces a stored key, and one command removes the stored key.
* [DECISION-09: The missing-key message tells a new user where to get a key](DECISION-09.md) - When no key is found, the message gives the page where the user gets a TypeSafe key.
* [DECISION-10: A result is a matching line with the lines around it](DECISION-10.md) - A result is the matching line with a few lines before and after it; matches near each other join into one result.
* [DECISION-11: When nothing answers well, jag labels the best results and exits 1](DECISION-11.md) - When no result answers the question well, jag shows the best results with a clear no-good-answer label and exits with code 1.
* [DECISION-12: The cost limit has a default that the user can change](DECISION-12.md) - jag has a fixed default cost limit, stated as a count of results to score, and the user can change it with an option.
* [DECISION-13: Reaching the cost limit has its own exit code](DECISION-13.md) - When jag reaches the cost limit, it returns the ranked results, says how many matches it skipped, and exits with its own code; exit 1 wins when nothing answers well.
* [DECISION-14: A folder search works like grep -r and skips noise files](DECISION-14.md) - jag searches each subfolder, skips binary, hidden, and ignored files, and says how many files it skipped.
* [DECISION-15: A result path has the same form as in grep](DECISION-15.md) - The path of each result follows the form of the folder that the user gave.
* [DECISION-16: A joined result shows its line range and marks each matching line](DECISION-16.md) - A result gives its first and last line, and it marks each line that matches the term.
* [DECISION-17: jag cuts a very long line and marks the cut](DECISION-17.md) - A result shows the exact source text, and jag cuts each line that is longer than a fixed length.
* [DECISION-18: Piped text has the label (standard input)](DECISION-18.md) - A result from piped text uses the grep label (standard input) and the line numbers of the piped text.
* [DECISION-19: The size limit counts characters](DECISION-19.md) - The limit on the text that jag returns is a count of characters.
* [DECISION-20: A default size limit always applies](DECISION-20.md) - jag always applies a default size limit for an agent; the user can raise it or turn it off.
* [DECISION-21: jag leaves out whole results with the lowest rank first](DECISION-21.md) - To stay in the limit, jag removes whole results, starting with the lowest rank.
* [DECISION-22: All text that jag writes to standard output counts toward the size limit](DECISION-22.md) - Labels, the structured wrapper, and notices on standard output count; text on the error stream does not.
* [DECISION-23: jag cuts a top result that is larger than the limit](DECISION-23.md) - When the top result alone is larger than the limit, jag cuts it, marks the cut, and says so.
* [DECISION-24: The notice gives the count and how to raise the limit](DECISION-24.md) - When jag leaves out results, the notice gives their number and how to raise the limit.
* [DECISION-25: A size limit that is not valid is a usage error](DECISION-25.md) - A limit of zero, a negative limit, or a value that is not a number gives exit code 2.
* [DECISION-26: Leaving out results has its own exit code](DECISION-26.md) - When jag leaves out results to stay in the limit, it exits with its own code above 2.
* [DECISION-27: Only an explicit option turns on structured output](DECISION-27.md) - Text is the default, also in a pipe; an option turns on structured output.
* [DECISION-28: Structured output has a field for each item that the text shows](DECISION-28.md) - Each label, count, mark, and notice of the text is a named field; each result has its rank and no score.
* [DECISION-29: In text mode, notices go to the error stream](DECISION-29.md) - Text mode writes results to standard output and notices to the error stream; structured mode writes all of it in the structure.
* [DECISION-30: Structured mode always writes a complete output](DECISION-30.md) - With no results, structured mode writes an empty result list and the summary fields.
* [DECISION-31: The command takes the term and the paths, and the question only from --ask](DECISION-31.md) - The form is jag [OPTIONS] TERM [PATH...], with the term first as in grep. The question comes only from the option --ask "QUESTION".
* [DECISION-32: A term with no question works as plain grep](DECISION-32.md) - With no question, that is with no --ask option, jag makes no model call, has no cost, and works as grep.
* [DECISION-33: The term is a regular expression, and -F makes it fixed text](DECISION-33.md) - The term is a regular expression, as in grep; the -F option makes it fixed text.
* [DECISION-34: Release 1 keeps the grep options that people use most](DECISION-34.md) - jag keeps a fixed set of grep options; any other option is a usage error that names it.
* [DECISION-35: With no path, jag searches the current folder or reads the pipe](DECISION-35.md) - With no path, jag searches the current folder, or reads standard input when text comes in through a pipe.
* [DECISION-36: The user can give any mix of files and folders](DECISION-36.md) - jag accepts several paths, and each path can be a file or a folder, as in grep.
* [DECISION-37: jag shows progress only in a terminal](DECISION-37.md) - While jag ranks, it shows a progress line on the error stream only when that stream is a terminal.
* [DECISION-38: Color works as grep --color=auto](DECISION-38.md) - jag colors the matching text only in a terminal, and --color=never or always changes that.
* [DECISION-39: When the user stops jag, it stops at once with no partial results](DECISION-39.md) - On Ctrl-C or a closed pipe, jag stops, starts no new model call, and shows no partial results.
* [DECISION-40: The cost is the billed amount in US dollars](DECISION-40.md) - jag shows the amount that the service billed, in US dollars; with no billed figure, it shows a labeled estimate.
* [DECISION-41: In text mode, the cost line goes to the error stream after each search](DECISION-41.md) - Text mode writes a cost line to the error stream after each search, also in a pipe; structured mode has a cost field.
* [DECISION-42: The report counts results scored and results skipped](DECISION-42.md) - jag shows the results it scored and the results that the cost limit skipped; together they are all results found.
* [DECISION-43: A search with no model call reports a cost of zero and the reason](DECISION-43.md) - With no question, jag makes no model call, reports a cost of zero, and says why.
* [DECISION-44: A failed or stopped search reports the cost up to that point](DECISION-44.md) - The error message gives the cost that jag paid before the failure or the stop.
* [DECISION-45: A missing cost figure never fails the search](DECISION-45.md) - When the service gives no cost figure, jag shows the count and marks the cost as not available.
* [DECISION-46: jag scores each search again and does not reuse scores](DECISION-46.md) - Release 1 keeps no scores between searches; a repeated search pays the model service again.
* [DECISION-47: The cost-limit exit code wins over the left-out exit code](DECISION-47.md) - When a search reaches the cost limit and also leaves out results, jag exits with the cost-limit code.
* [DECISION-48: The readiness check exits 0 or with the code of the first cause](DECISION-48.md) - The readiness check exits with code 0 when all five items pass, and otherwise with the code of the first cause that fails.
* [DECISION-49: Plain grep mode needs no key](DECISION-49.md) - A term with no question makes no model call, so jag works with no stored key.
* [DECISION-50: The -A, -B, and -C options set the context lines of a result](DECISION-50.md) - The grep context options set the lines around each match; with no option, jag uses its default.
* [DECISION-51: jag goes public as a new repository with a clean history](DECISION-51.md) - A new public repository gets a clean history. ai-ready-future/jag stays private, and its history is not rewritten.
* [DECISION-52: The failure limit comes from the timeout and the retry wait](DECISION-52.md) - When the model service does not answer, the time limit is 2 × the timeout + the retry wait, not a fixed number.
