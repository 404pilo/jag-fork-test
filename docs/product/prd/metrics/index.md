# metrics

* [METRIC-01: Answer quality](METRIC-01.md) - jag's ndcg@10 on a held-out evaluation set, compared with blind grep; the target is at least 0.70 at release 1.
* [METRIC-02: Context saved](METRIC-02.md) - An agent reads at least five times less text with jag than with grep for the same question.
* [METRIC-03: Search speed](METRIC-03.md) - A typical jag search returns in under five seconds, so users do not go back to grep.
* [METRIC-04: Time to first result](METRIC-04.md) - A new user goes from install to a first good result in under five minutes.
* [METRIC-05: Cost per search](METRIC-05.md) - The model cost of one search stays at or below a fixed cap, so quality does not rise by scoring more snippets.
* [METRIC-06: Answer still found](METRIC-06.md) - jag must not save context by cutting the line that answers the question.
* [METRIC-07: False confidence](METRIC-07.md) - When nothing answers the question, jag says so; measured with decoy questions.
* [METRIC-08: Setup failures](METRIC-08.md) - The number of reported problems with install or key setup stays low.
