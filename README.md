# jag

Grep with a model that ranks the matches by how well they answer your
question. The package on crates.io is `jag-grep`. The command is `jag`.

This release has the key setup only. The search comes later.

## Store your TypeSafe key

Get a key at <https://console.typesafe.ai/keys>. Then run:

```
$ jag --setup
```

`jag --setup` asks for the key. The terminal shows no echo. It stores the key
in `~/.config/jag/config.yml`. It needs a terminal. With no terminal, set
`JAG_TYPESAFE_API_KEY` instead.

To remove the stored key:

```
$ jag --forget-key
```

## Where jag looks for the key

jag uses the first key that it finds, in this order:

1. the variable `JAG_TYPESAFE_API_KEY`
2. the variable `TYPESAFE_API_KEY`
3. the file `~/.config/jag/config.yml`

The file has this form. The key goes under `config:`, not at the top level:

```yaml
config:
  api-key: <your key>
```

jag refuses a key file that other users can read. Run `chmod 600` on it.
jag never shows the key.

## Check that jag is ready

```
$ jag --check
```

The check shows `pass` or `fail` for five items, in this order:

1. a key is present
2. the service accepts the key
3. the account has credit
4. the service is reachable
5. other users cannot read the key file

An item that jag cannot examine, because an earlier item failed, shows
`not checked`. Each item that fails has its cause and its fix on the error
stream. The check makes one small call to the model service. The call costs
less than $0.00001. `--timeout` and `--retry-wait` apply to the call.

When all five items pass, the check shows this and exits with 0:

```
pass        a key is present (JAG_TYPESAFE_API_KEY)
pass        the service accepts the key
pass        the account has credit
pass        the service is reachable
pass        other users cannot read the key file (no key file)
jag is ready.
```

Exit codes:

| Code | Cause |
|---|---|
| 0 | All five items pass |
| 2 | The command line or a setting is not valid |
| 3 | No key, a bad key, or a key file that others can read |
| 4 | The service does not accept the key |
| 5 | The account has no credit |
| 6 | The service is not reachable |

When more than one item fails, the code is the code of the first item that
fails.

## Output for an agent

By default, jag writes text, also when the output goes to a pipe. The
results go to standard output. The notices go to the error stream.

For an agent, add `--json`:

```
$ jag --json TERM "QUESTION" PATH...
```

jag then writes one JSON document on one line, with `\n` at the end, on
standard output. The document holds the results, the counts, the notices, and
each error. jag writes nothing on the error stream, unless it cannot write the
document to standard output: then it writes that failure there, as text. Only the option turns on
this form: no variable and no setting does. `--json` does not work with
`--setup`, `--forget-key`, or `--check`.

Each document has these fields:

| Field | Meaning |
|---|---|
| `version` | The version of this form. It is `1` |
| `prompt_version` | The version of the scoring prompt, for example `jag-p1` |
| `exit_code` | The exit code of jag |
| `error` | `null`, or an object with `cause` and `fix` |
| `no_good_answer` | `true` when the text mode shows the no-good-answer label |
| `results_found`, `results_scored`, `results_skipped` | The three numbers of the cost-limit notice |
| `left_out`, `top_result_cut`, `size_limit_fix`, `warnings_left_out` | The size limit of the output. They are `0`, `0`, `""`, and `0` until jag has a size limit |
| `files_skipped` | An object with `total`, `binary`, `hidden`, and `ignored` |
| `warnings` | A list of objects with `path` (text or `null`) and `text` |
| `results` | A list in rank order |

Each result has `rank` (1 for the best result), `path` (`null` for standard
input), `first_line`, `last_line`, and `lines`. Each line has `number`,
`text`, `is_match`, and `cut` (the count of characters that jag cut from the
end of the line, or `0`). The document has no score.

When no line matches, the document has an empty `results` list and `exit_code`
1. When jag fails, `error` holds the cause and the fix, `results` is empty,
and the other fields hold what jag counted before the failure.

In one major version of jag, jag does not remove or rename a field. It can add
a field, so agent code must accept a field that it does not know. The order of
the fields can change: a new field goes before `results`. `version` changes
only when a field is removed or renamed.
