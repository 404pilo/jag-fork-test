---
type: system-design
title: jag system design
status: draft
version: 1
updated: 2026-10-12
session: 2d045b37-ac91-451e-b37b-928ac1645df2
areas:
  quality: decided
  context: open
  strategy: open
  building-blocks: open
  data: open
  contracts: open
  conventions: open
  deployment: open
adrs: []
---

# jag system design

## Design map

Destination: one command-line program that finds the lines that match a term,
ranks them by a question with the Jev model, and returns the best lines within
a size limit and a cost limit.

Frontier: context. Strategy, building blocks, data, contracts, conventions, and
deployment wait for it.

Fog:

- strategy-1: the default count of results to score, and the count of model
  calls in parallel. A research item is open.

Out of scope:

- A search index. Each search reads the files as they are (SCOPE-06).
- A store of scores between searches (DECISION-46).
- A server, an account, and telemetry (SCOPE-09).
- A local model. Release 1 needs the model service (DECISION-02).

## Quality scenarios

### QS-1: The top results answer the question

- Source: an agent or a person.
- Stimulus: a term and a question, on a folder that holds the answer.
- Environment: normal operation, with the default limits.
- Artifact: the ranking of jag.
- Response: jag returns the matches ranked by how well they answer the
  question.
- Response measure: ndcg@10 of 0.70 or more on the held-out set.
- Importance: H. Risk: H, because the held-out run is not done and the
  scoring prompt is not final.
- Cites: METRIC-01.

### QS-2: jag says when nothing answers

- Source: an agent.
- Stimulus: a question that no line of the folder answers.
- Environment: normal operation.
- Artifact: the ranking and the exit code.
- Response: jag shows the best results with the "no good answer" label and
  exits with code 1.
- Response measure: a decoy AUC of 0.85 or more.
- Importance: H. Risk: M, because the earlier measure is close to the target.
- Cites: METRIC-07.

### QS-3: A typical search is fast

- Source: a person at the shell.
- Stimulus: one typical search.
- Environment: a medium code repository, the default cost limit, and a model
  service that works.
- Artifact: the full search, from the start of the command to the last line.
- Response: jag returns the ranked results.
- Response measure: under 5 seconds.
- Importance: H. Risk: M, because the time of one model call with 32 snippets
  and the time of the file search are estimates, not measurements.
- Cites: METRIC-03, CONSTRAINT-06.

### QS-4: jag fails clearly and fast when the model service is down

- Source: an agent.
- Stimulus: the model service does not answer during a search.
- Environment: a search with a question.
- Artifact: the model client and the exit code.
- Response: jag fails the full search with no partial result. It gives the
  cause and the fix, and exits with the code of the cause.
- Response measure: within 10 seconds of the first call, with the one retry
  included.
- Importance: H. Risk: M, because a long timeout or many retries break the
  target.
- Cites: CONSTRAINT-07, CONSTRAINT-04.

### QS-5: The key stays secret

- Source: any run of jag.
- Stimulus: a message, an error, a notice, or structured output.
- Environment: every mode, including a failure.
- Artifact: the key store and every output path.
- Response: the key never appears in the output. jag refuses a key file that
  other users on the machine can read.
- Response measure: zero outputs that hold the key, in a test of every error
  path.
- Importance: H. Risk: L, because one module owns the key.
- Cites: CONSTRAINT-02.

METRIC-02, METRIC-04, METRIC-05, METRIC-06, and CONSTRAINT-05 have no
scenario. They stay as guards through their pages.
