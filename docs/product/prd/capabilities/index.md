# capabilities

* [CAPABILITY-01: Guided key setup and health check](CAPABILITY-01.md) - The user stores a TypeSafe key one time, and one command confirms that jag is ready or says what to fix.
* [CAPABILITY-02: Term and question search](CAPABILITY-02.md) - The user gives a folder, a term, and a plain-language question; jag returns the matching lines ranked by how well they answer the question.
* [CAPABILITY-03: File and line for each result](CAPABILITY-03.md) - Each result shows the file, the line number, and the snippet, so the person or the agent can open it.
* [CAPABILITY-04: Limit on the text that jag returns](CAPABILITY-04.md) - The user sets a limit on how much text jag returns to the agent; jag never goes over it and says what it left out.
* [CAPABILITY-05: Text output for people and structured output for agents](CAPABILITY-05.md) - jag gives readable text to a person and structured output that an agent can parse.
* [CAPABILITY-06: Feels like grep](CAPABILITY-06.md) - jag works in a pipe, uses grep-like options and exit codes, and is fast enough to replace grep.
* [CAPABILITY-07: Show the cost of a search](CAPABILITY-07.md) - jag reports how many snippets it scored and what the search cost.
* [CAPABILITY-08: A repeated search costs nothing](CAPABILITY-08.md) - When the user asks the same question of the same text again, jag does not pay for it again.
* [CAPABILITY-09: Say which text leaves the machine](CAPABILITY-09.md) - jag tells the user which text it sends to the model service, so the user can decide before a search.
* [CAPABILITY-10: Question without a term](CAPABILITY-10.md) - The user gives only a question, and jag chooses the candidate lines itself.
* [CAPABILITY-11: Agent instructions to copy](CAPABILITY-11.md) - A ready snippet teaches an agent when and how to call jag.
* [CAPABILITY-12: Measure jag on your own questions](CAPABILITY-12.md) - The user runs jag and grep on their own questions and sees which one finds the answer more often.
