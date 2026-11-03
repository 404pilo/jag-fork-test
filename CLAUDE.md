<!-- adr:router start -->
## Architecture decisions

Decisions that are expensive to undo are recorded in `docs/adr/`.
The rules are in [`docs/adr/CLAUDE.md`](docs/adr/CLAUDE.md).

Read `docs/adr/README.md` before you change an architectural boundary. Open only the ADRs that apply.
When you make a decision that is expensive to undo, write an ADR with `status: proposed`.
Do not stop to ask.
<!-- adr:router end -->

## Build, test and verify

jag is one Rust binary (ADR-0005). The toolchain is pinned in `rust-toolchain.toml`.
On a machine without `cargo`, use Docker with the image `rust:1.99`.

- Build: `cargo build --locked` (release: `cargo build --release --locked`)
- Run: `cargo run --locked -- <args>`, or `target/release/jag <args>`
- Verify before a PR. These are the same checks as `.github/workflows/ci.yml`:
  - `cargo fmt --check`
  - `cargo clippy --locked --all-features -- -D warnings`
  - `cargo test --locked --all-features`. The rg parity test needs `JAG_PARITY_RG`. Get it with `scripts/parity/install-rg.sh <folder>`.
  - `cargo test --locked --release --test speed`
  - `python3 -I -m unittest discover -s scripts/release -p 'test_*.py'`
  - `node --test npm/jag/test/launcher.test.js tests/agent_skill/install.test.js`
  - ADR changes: `python3 scripts/adr/adr_index.py --check`

Each code folder has a short `CLAUDE.md` that tells what is in it.

<!-- knowledge:router start -->
## Knowledge graph

The generic knowledge graph of this repository is `docs/wiki/bundle/`.
[AGENTS.md](./docs/wiki/AGENTS.md) holds its rules.

- Run `/knowledge:ask <question>` to ask a question. It reads each graph in
  `.claude/knowledge.json`, one step at a time.
- Run `/knowledge:learn <path or URL>` to add a source or an answer.
- Run `/knowledge:forget <graph>:/<path>` to remove a page and each reference
  to it.

Do not edit the generated files: each `index.md`, each `LINKS` block, and
`links.tsv`. Write pages only through `knowledge_graph.py`.
<!-- knowledge:router end -->

<!-- system-design:router start -->
## System design

The architecture of this repository is the graph `docs/system-design/` (graph name
`system-design` in `.claude/knowledge.json`).

- Run `/system-design:ask <question>` for questions such as "which component
  realizes capability X" and "which scenarios constrain this component".
- Run `/system-design:learn [area]` to decide the open parts of the design.
- Write pages only through `system_design.py`. Do not edit the generated
  files: each `index.md`, each `LINKS` block, the blocks of `system.md`, and
  `links.tsv`, because the next regeneration overwrites a hand edit.
<!-- system-design:router end -->
