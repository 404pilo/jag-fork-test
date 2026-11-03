# jag-bench

`jag-bench` scores jag and a blind grep on a set of cases that you label.
It is for the maintainer of jag. Releases do not include it.

A case has a folder, a search term, a question, and the lines in the folder
that answer the question. For each case, `jag-bench` runs the search of jag
and a blind grep on the same files. It gives each of them a score from 0 to
1, and it prints four summary lines.

## Build

Build the tool with the Cargo feature `bench`:

```
$ cargo build --release --features bench --bin jag-bench
```

The program is `target/release/jag-bench`. Without the feature, Cargo does
not build it.

## Run

```
$ jag-bench [--detail FILE] CASES
```

`CASES` is the case file. `--detail FILE` writes one result line for each
case to `FILE`. Without `--detail`, the tool writes no result for a case.

`jag-bench` reads the key and the model settings as jag does: the key
variables, the key file, `JAG_BASE_URL`, `JAG_MODEL`, `JAG_TIMEOUT`, and
`JAG_RETRY_WAIT`. It has no option for the prompt, the threshold, the
model, the cost limit, the context lines, or the bar. These values are
fixed, so that each run is the same test.

Keep the case file and the `--detail` file outside the source folder of
jag. `jag-bench` stops with exit 2 when one of them is inside it.

## The case file

The case file is one YAML file. This example uses invented data:

```yaml
cases:
  - folder: shop            # relative to the case file, or absolute
    term: retry
    question: Where does the client wait before it tries again?
    answers:
      - file: src/client.rs # relative to the folder
        line: 42
        grade: 2
      - file: src/client.rs
        line: 57            # grade 1 when not given
  - folder: shop
    term: refund
    ignore_case: true       # as -i; false when not given
    fixed_strings: false    # as -F; false when not given
    question: Who approves a refund above the limit?
    answers: []             # a case with no answer
```

| Field | Meaning |
|---|---|
| `folder` | The folder to search. A relative path starts at the folder of the case file. |
| `term` | The search term. It follows the rules of jag. |
| `ignore_case` | The same as `-i`. The default is `false`. |
| `fixed_strings` | The same as `-F`. The default is `false`. |
| `question` | The question that jag gives to the model. |
| `answers` | The labeled lines. `[]` means that the case has no answer. |
| `answers[].file` | The path of the file, relative to `folder`. |
| `answers[].line` | The line number. The first line is 1. |
| `answers[].grade` | 1, 2, or 3. A higher grade is a better answer. The default is 1. |

`jag-bench` stops with exit 2, before the first model call, when a rule is
broken. The message names the case by its number and the field. It does not
show the text of the case. The rules are:

- `cases` holds one case or more. An unknown field is an error.
- `folder` is a folder that exists.
- `term` is not empty, and it is a valid term.
- `question` is not empty.
- Each answer has `file` and `line`. The file exists in the folder. `line` is
  1 or more. `grade` is 1, 2, or 3.
- Two answers of one case do not name the same file and line.
- At least one case has an answer.

## How the tool scores a case

For each case with an answer, in the order of the file:

1. The tool scans `folder` as jag scans the current folder when you give it
   no path.
2. The blind grep sorts the matching lines by the bytes of the path, and then
   by the line number. Each line is one result. The tool keeps the first 10.
3. jag runs its search: the result windows, the pre-rank when there are more
   than 200 results, the model scores, and the rank. The tool keeps the first
   10. These 10 results are the same as the first 10 `results` of
   `jag --json --ask`, run in the folder with no path, for the same term and
   question.

A case with no answer makes no model call. It is not in the mean.

A result gets credit for the labeled line of the highest grade that is a
matching line of the result, and that no earlier result got credit for.
A context line gives no credit. A labeled line gets credit once. A labeled
line and a result match when the file (relative to `folder`) and the line
number are the same.

The score of a list of results is ndcg@10:

```
DCG@10  = sum over rank i of grade_i / log2(i + 1)
IDCG@10 = the DCG@10 of the grades of all labeled lines, highest first, cut at 10
ndcg@10 = DCG@10 / IDCG@10
```

A case with no matching line scores 0 and stays in the mean. The score of a
tool is the mean of its ndcg@10 over the cases with an answer.

## Output

Standard output holds four lines, and only when each case ended:

```
jag-bench: 4 cases, 1 with no answer
jag-bench: ndcg@10 jag 0.903, grep 0.703
jag-bench: bar pass (jag 0.70 or more, and higher than grep)
jag-bench: prompt jag-p1, threshold 0.4, model <model>, jag <version>
```

The numbers are rounded to 3 places. The bar passes when the score of jag is
0.70 or more and is higher than the score of grep. The tool compares the
values before it rounds them. When the bar fails, the third line says
`bar fail`.

Standard error shows one progress line for each case: `jag-bench: case 1 of
4`. It never shows the text of a case. A failure names the case by number
and field, and gives a cause and a fix.

`--detail FILE` writes one JSON line for each case. It holds no text and no
path:

```json
{"case":1,"jag":0.832,"grep":0.117,"answers":[{"jag_rank":1,"grep_rank":null},{"jag_rank":4,"grep_rank":9}]}
```

`jag_rank` and `grep_rank` give the rank (1 is the first) of the result that
got credit for the labeled line. They are `null` when no result in the first
10 got credit. The tool writes the file when each case has ended.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Each case ended, and the bar passes. |
| 1 | Each case ended, and the bar fails. |
| 2 | Usage error, a bad case file, or a file inside the source folder. |
| 3 to 6 | A failure of the key or of the model service, as for jag. |
| 130 | A Ctrl-C stopped the run. |

When a failure stops the run, the tool prints no score.

This line is a test of the fork pull request approval (STORY-77, step L3). Do not merge.
