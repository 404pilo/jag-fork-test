# scripts/adr

This folder holds the ADR tools. Each script uses the Python standard library only. The rules for ADRs are in `docs/adr/CLAUDE.md`.

- `adr_lib.py`: reads and writes ADR files. The other scripts use it.
- `adr_new.py`: makes one new ADR.
- `adr_index.py`: makes the ADR register between its markers. `--check` finds a stale register.
- `adr_lint.py`: checks the ADR files. CI runs it on the changed files.
- `adr_migrate.py`: changes old bold-key ADRs to frontmatter.
- `requirements.txt`: `pytest`, for the tests only.

Test: `python3 -m pytest tests/adr/` (first, `pip install -r scripts/adr/requirements.txt`).
