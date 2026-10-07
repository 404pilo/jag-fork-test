# needs

* [NEED-01: Give the agent a search tool for a folder of text](NEED-01.md) - The agent builder gives their agent a way to find facts in a folder of text, so the agent answers correctly.
* [NEED-02: Search output fills the agent context with noise](NEED-02.md) - Keyword search output fills the agent context, so the agent loses focus and the token bill rises.
* [NEED-03: Drop-in search with no setup](NEED-03.md) - The agent builder adds one command to their agent, and it works on any folder with no index and no server.
* [NEED-04: Search logs and system files for any question](NEED-04.md) - The operator searches logs, configuration, and output for any question, as with grep.
* [NEED-05: The operator must guess the exact keyword](NEED-05.md) - The operator must know the exact error text to search for; without it, they read thousands of lines.
* [NEED-06: Ask the logs in plain words](NEED-06.md) - The operator asks a question in plain words and gets the few lines that answer it.
* [NEED-07: Find the passage that answers a question](NEED-07.md) - The knowledge seeker finds the passage that answers a question in a folder of notes or documents.
* [NEED-08: Each match must be judged by hand](NEED-08.md) - A term search returns many matches, and the knowledge seeker must read or search each one again to judge it.
* [NEED-09: Point me to the passage](NEED-09.md) - The knowledge seeker gets the passage that answers the question, with the file name.
* [NEED-10: Find specific lines, such as one kind of error](NEED-10.md) - The shell power user finds the lines of one kind, such as one type of error in the logs.
* [NEED-11: Too many matches unless the exact text is known](NEED-11.md) - A broad term returns too many matches, so the shell power user must know the exact message to search for.
* [NEED-12: grep, but it understands](NEED-12.md) - The shell power user searches a broad term and describes in plain English the lines they want.
* [NEED-13: Search prompt tuning fixes one case and breaks another](NEED-13.md) - The agent builder tunes the search instructions after a wrong answer, and the change fixes one case and breaks another.
* [NEED-14: A sub-agent or a vector index adds cost and can still miss](NEED-14.md) - A reader sub-agent or a vector index adds cost or setup for the agent builder, and the agent can still miss the answer.
