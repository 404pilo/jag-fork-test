# tests/adr

This folder holds the pytest tests of the ADR tools in `scripts/adr/`. One test file for each script.

- `conftest.py`: shared helpers. It finds the scripts in this layout and in the layout of a target repository. It ships byte for byte, so keep both layouts.
- `test_adr_<script>.py`: the tests of `scripts/adr/adr_<script>.py`.

Test: `python3 -m pytest tests/adr/` (first, `pip install -r scripts/adr/requirements.txt`).
