# scripts/speed

This folder holds a time check with the real model service (STORY-44, CRITERION-65).

- `measure.py`: measures the time of a typical search. It exits with 0 when it meets the target, 1 when it does not, and 2 when jag fails.

The same check with the fake server is `cargo test --release --test speed`.
