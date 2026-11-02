# npm/jag/bin

This folder holds the npm launcher of jag. The `bin` field of `../package.json` points here.

- `jag.js`: finds the binary package of this platform and starts the binary. It uses no network module. Its platform table must agree with `../package.json`.

Test: `node --test npm/jag/test/launcher.test.js`

Spec: `docs/superpowers/specs/2026-10-14-STORY-48-install-jag-from-github-releases-crates-io-or-npm-design.md`, section "The launcher".
