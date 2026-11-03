# Log

## 2026-11-03
* **Report**: set failed: the save code has no operation that writes the body of a page. I edited entities/pilo.md directly: I added the GitHub account, role, decisions owned, escalation and Slack lines to the body. Reason: the description field holds one line only., by knowledge/1.1.3, at 2026-11-03T11:14:39Z
* **Report**: new failed: the save code has no People type and no people/ folder. I wrote people/index.md directly: one line that links to entities/pilo.md. Reason: sofa:readiness (people-page) checks for a people/ folder or a people.md in the knowledge bundle, and the sofa setup skill asks for pages people/<name>., by knowledge/1.1.3, at 2026-11-03T11:14:39Z
* **Report**: Recommendation: the message of the general check rule K-dir-index says 'a directory index.md has no frontmatter', but the rule fires when the index HAS frontmatter (general_check.py line 57). Change the message to 'a directory index.md must not have frontmatter'., by knowledge/1.1.3, at 2026-11-03T11:14:39Z
* **Report**: Recommendation: align the knowledge brain standard with sofa. Either add a Person type that writes people/<slug>.md, or change the sofa people-page check and setup step 6 to use entities/<slug>.md. Also, links from a folder index.md do not count for G4, so a person page stays an orphan., by knowledge/1.1.3, at 2026-11-03T11:14:39Z

## 2026-11-02
* **Setup**: made the bundle docs/wiki/bundle, by claude-code/opus-5.5, at 2026-11-02T23:07:17Z
