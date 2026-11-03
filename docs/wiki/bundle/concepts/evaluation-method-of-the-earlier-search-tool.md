---
type: Concept
title: Evaluation method of the earlier search tool
description: 'How the earlier evaluation tool scored a search tool: snippets, gain 0/1/2, nDCG@10, three arms, terms from the question text.'
status: draft
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T21:21:43Z'
---

# Evaluation method of the earlier search tool

How the earlier evaluation tool scored a search tool. The source is a researcher study of the code of that tool (2026-11-03, veracity 60: a researcher result). This page holds no repository name, private path, commit, case id, or case data.

## Unit of a relevant result

The unit is a snippet, not a whole document and not a single line.

- A snippet is a window around the grep hits: 3 context lines before and 3 after, at most 12 lines.
- Hits that are near each other share 1 snippet.

## Gain

Each snippet gets a gain of 0, 1, or 2.

- 2: the snippet covers an evidence line.
- 1: the snippet is in a gold document.
- 0: any other snippet.

## Score

The score is nDCG@10.

- IDCG is computed over the same snippet set that grep found. It is not computed over all labels.
- A case with IDCG 0 is left out of the mean.

## Three arms

The 3 arms are 3 orders of the same snippet set.

- blind: by path, then by line. The "grep" baseline is the blind order.
- lexical: by idf.
- jag: by the score of the model.

## Terms

The terms come from the text of the question.

1. Split the text at characters that are not letters or digits.
2. Drop short tokens, and drop question words and function words.
3. Stem a simple plural.
4. Keep the first 12 unique terms.
5. Join them with OR as 1 literal, case-insensitive pattern.

The fields `mustContain` and expected value are not used for terms.

## Evidence lines

Evidence lines are the lines of the gold documents that hold a needle.

- A needle is an expected value, split at ";" for each subject. A `mustContain` entry is a needle in all gold documents.
- Keep a needle only if it has a digit or at least 5 characters.
- A needle that matches more than 5 lines in a document marks nothing in that document.

## Development numbers

`prd:/metrics/METRIC-01.md` gives the public summary: jag about 0.72, blind grep about 0.06.

The researcher study (2026-11-03) gives more detail for the same development run. These are totals only:

- 83 cases, 74 defined (the other cases have IDCG 0).
- nDCG@10: blind 0.06, lexical 0.38, jag 0.73.

## Why it matters for STORY-45

The labeling rule of STORY-45 is different from this method in 4 ways:

- Term: the first `mustContain` entry, or the first expected value if there is no entry. The method uses the terms of the question text.
- Unit: a single labeled line, not a snippet.
- Gain: 1 for each labeled line, not 0, 1, or 2.
- IDCG: over all labels, not only over the snippets that grep found.

So the rule did not score as the earlier tool did, and it failed its development test (jag 0.870 and grep 0.455, against about 0.72 and 0.06). The planner found that the term is the main cause (2026-11-03). See [the STORY-45 page](/synthesis/story-45-test-the-labeling-rule-on-the-development-set-first.md).

<!-- LINKS:START (generated, do not edit) -->

## Links

- fed: [STORY-45: test the labeling rule on the development set first](../synthesis/story-45-test-the-labeling-rule-on-the-development-set-first.md)
<!-- LINKS:END -->
