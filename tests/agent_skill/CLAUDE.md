# tests/agent_skill

This folder holds the install test of the jag agent skill (FEATURE-24, CRITERION-116). The content tests are in `../agent_skill.rs`.

- `install.test.js`: runs the pinned `skills` tool on this checkout. It writes only in temporary folders and makes no call to the model service.
- `agents-1.7.4.txt`: the agents of `skills@1.7.4` and the project folder of each. A new pin of the tool needs a new file.

Test: `node --test tests/agent_skill/install.test.js`
