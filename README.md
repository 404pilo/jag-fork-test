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
