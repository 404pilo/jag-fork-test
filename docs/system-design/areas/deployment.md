---
type: Area
title: Deployment
description: The deployment area of the design.
status: draft
area: deployment
area_state: decided
anchor: deployment
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
decisions:
- key: deployment-1
  binds: How users install jag
  adr: adr:/0010-ship-jag-on-github-releases-crates-io-and-npm.md
  source: adr
- key: deployment-2
  binds: The operating systems and the runtime versions that jag supports
  source: human:404pilo, 2026-10-12
- key: deployment-3
  binds: The release path and the checks that guard it
  source: human:404pilo, 2026-10-12
- key: deployment-4
  binds: The name of the public repository
  adr: adr:/0019-give-the-new-public-repository-the-name-ai-ready-future-jag.md
  source: adr
- key: deployment-5
  binds: The names of the release archives, the checksum files, and the npm platform packages
  adr: adr:/0020-name-the-release-archives-and-the-npm-platform-packages-by.md
  source: adr
- key: deployment-6
  binds: 'npm is the main channel of the program, and the `skills` tool is the channel of the skill: after `npm install -g @airf/jag`, `npx skills add ai-ready-future/jag` reads `skills/jag/SKILL.md` from the repository. cargo and the release downloads stay in the README under "Other ways to install"'
  adr: adr:/0024-install-the-jag-agent-skill-from-the-repository-with-the.md
  source: adr
---

# Deployment

<!-- LINKS:START (generated, do not edit) -->

## Links

- decisions adr: [adr:/0010-ship-jag-on-github-releases-crates-io-and-npm.md](../../adr/0010-ship-jag-on-github-releases-crates-io-and-npm.md)
- decisions adr: [adr:/0019-give-the-new-public-repository-the-name-ai-ready-future-jag.md](../../adr/0019-give-the-new-public-repository-the-name-ai-ready-future-jag.md)
- decisions adr: [adr:/0020-name-the-release-archives-and-the-npm-platform-packages-by.md](../../adr/0020-name-the-release-archives-and-the-npm-platform-packages-by.md)
- decisions adr: [adr:/0024-install-the-jag-agent-skill-from-the-repository-with-the.md](../../adr/0024-install-the-jag-agent-skill-from-the-repository-with-the.md)
- area: [deployment: Deployment](deployment.md)
- named by: [COMPONENT-08: jag-bench](../components/COMPONENT-08.md)
- named by: [COMPONENT-11: Agent skill](../components/COMPONENT-11.md)
- named by: [deployment: Deployment](deployment.md)
<!-- LINKS:END -->
