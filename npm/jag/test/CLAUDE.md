# npm/jag/test

This folder holds the tests of the npm launcher `../bin/jag.js`.

- `launcher.test.js`: uses the `node:test` API of Node 18. A fake binary package in a temporary `node_modules` stands in for the real binary.

Test: `node --test npm/jag/test/launcher.test.js`
