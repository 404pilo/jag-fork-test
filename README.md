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
