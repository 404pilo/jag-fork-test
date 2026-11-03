# Log

## 2026-11-03
* **Setup**: made the bundle docs/system-design, by system-design/0.2.3, at 2026-11-03T11:37:02Z
* **Migrate**: migrated docs/product/system-design.md (27 items, 6 not migrated); the old file stays, by claude-code/opus-5.5, at 2026-11-03T11:37:50Z
* **Settle**: system accepted -> draft: the design covers release 1 only. CAPABILITY-08, 09, 10, 13 and 14 of later or wont scopes have no component (owner choice, human:404pilo, 2026-11-03)., by claude-code/opus-5.5, at 2026-11-03T11:59:30Z
* **Report**: Recommendation: the GAP rule X4 counts each include of each release scope, also an include with priority wont (CAPABILITY-08 in SCOPE-01) and the scopes of later releases. An accepted design for release 1 cannot pass. Skip wont includes, and let system.md name the scopes it covers (destination)., by claude-code/opus-5.5, at 2026-11-03T11:59:30Z
* **Report**: Recommendation: migrate-file copies design_state accepted from the old file, but the new GAP rule then makes each gap an ERROR. Migrate as draft, or warn that the accepted state turns gaps into errors., by claude-code/opus-5.5, at 2026-11-03T11:59:30Z
* **Report**: Recommendation: resolve does not find a page by its slug field (resolve agent-skill failed for COMPONENT-11, slug agent-skill). new should print the new ID, or resolve should match slug., by claude-code/opus-5.5, at 2026-11-03T11:59:30Z
* **Report**: Recommendation: the X6 rule asks for a decision row for each ADR whose parent_spec names an area, but init does not offer to run migrate-file when docs/product/system-design.md exists. Setup ends with X6 errors until the person runs learn., by claude-code/opus-5.5, at 2026-11-03T11:59:30Z
