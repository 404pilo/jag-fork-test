import ast
import json
import sys

import adr_vendored_conftest as _cf

load_script = _cf.load_script
run_script = _cf.run_script
write = _cf.write

adr_lib = load_script("adr_lib")
adr_index = load_script("adr_index")
adr_lint = load_script("adr_lint")


def repo(tmp_path, config=None):
    (tmp_path / ".git").mkdir(parents=True)
    (tmp_path / "docs" / "adr").mkdir(parents=True)
    write(tmp_path / ".claude" / "adr.json",
          json.dumps(config or {"version": 1, "dir": "docs/adr",
                                "register": "docs/adr/README.md",
                                "id_scheme": "sequential"}))
    return tmp_path


def good(root, name, **meta):
    body = {"type": "adr", "status": "accepted", "date": "2026-08-13",
            "title": "T", "area": "core"}
    body.update(meta)
    write(root / "docs" / "adr" / name,
          "---\n" + adr_lib.dump_frontmatter(body) + "---\n\n# body\n")


def index(root):
    run_script("adr_index", "--write", cwd=root)


def write_adr(root, name, *, status, area, raw):
    return write(root / "docs" / "adr" / name, (
        "---\n"
        f"type: adr\ntitle: {name}\narea: {area}\nstatus: {status}\n"
        f"date: 2026-01-01\n{raw}"
        "---\n\n# body\n"
    ))


# --- Task 15: required fields, enum, legacy warning -------------------------

def test_a_clean_corpus_exits_zero(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_frontmatter_missing_status_is_the_required_field_error(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md",
          "---\ntype: adr\ndate: 2026-08-13\ntitle: T\n---\n\n# T\n")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "required" in result.stdout.lower()
    assert "legacy" not in result.stdout.lower()
    assert ("ERROR: 0001-a.md: required-frontmatter: frontmatter is missing "
            "`status`") in result.stdout
    assert ("ERROR: 0001-a.md: schema-required: schema requires `status`"
            in result.stdout)
    assert result.stdout.rstrip().endswith("3 error(s), 0 warning(s)")


def test_a_legacy_file_is_a_warning_and_exits_zero(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md",
          "# ADR-002: Gateway auth\n\n**Status:** Accepted\n**Date:** 2026-02-26\n\n---\n")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout
    assert "WARN" in result.stdout and "legacy" in result.stdout.lower()
    assert ("WARN: ADR-002-auth.md: legacy-metadata: legacy bold-key metadata; "
            "run /adr:migrate when ready") in result.stdout
    assert result.stdout.rstrip().endswith("0 error(s), 1 warning(s)")


def test_a_stray_note_is_the_same_legacy_warning(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "stray-note.md", "just a note\n")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0
    assert "legacy" in result.stdout.lower()
    assert "WARN: stray-note.md: legacy-metadata:" in result.stdout
    assert result.stdout.rstrip().endswith("0 error(s), 1 warning(s)")


def test_a_status_outside_the_enum_and_an_unparseable_date_are_errors(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md",
          "---\ntype: adr\nstatus: superseded by adr-002\ndate: last tuesday\n"
          "title: T\n---\n\n# T\n")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "status" in result.stdout.lower() and "date" in result.stdout.lower()
    assert ("ERROR: 0001-a.md: status-enum: status `superseded by adr-002` is "
            "outside proposed, accepted, deprecated, superseded") in result.stdout
    assert ("ERROR: 0001-a.md: date-parse: date `last tuesday` is not YYYY-MM-DD"
            in result.stdout)
    assert result.stdout.rstrip().endswith("5 error(s), 0 warning(s)")


def test_malformed_frontmatter_is_reported_as_yaml_invalid(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md",
          "---\ntype: adr\nstatus: accepted\ndate: 2026-08-13\ntitle: T\n"
          "evidence: |\n  multiline\n---\n\n# T\n")
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "yaml-valid" in result.stdout
    assert ("ERROR: 0001-a.md: yaml-valid: unsupported YAML at line 5: "
            "block scalar under a list key") in result.stdout
    assert result.stdout.rstrip().endswith("1 error(s), 0 warning(s)")


def test_register_structure_rules_are_reported_together(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md", area="alpha", title="A")
    good(root, "0002-b.md", area="beta", title="B")
    write(root / "docs" / "adr" / "README.md", (
        f"{adr_index.BEGIN}\n\n{adr_index.WARNING_BLOCKQUOTE}\n\n"
        "### beta — Beta\n"
        "- [0001 — A](./0001-a.md) — accepted — first\n"
        "- [0001 — A](./0001-a.md) — accepted — duplicate\n"
        "- [0099 — Gone](./0099-gone.md) — accepted — dead\n\n"
        f"{adr_index.END}\n"
    ))
    issues = adr_lint.lint(adr_lib.load_corpus(root))
    rules = {issue.rule for issue in issues}
    assert {"orphan", "duplicate-bullet", "dead-link", "wrong-section", "ordering-drift"} <= rules
    drift = next(issue for issue in issues if issue.rule == "ordering-drift")
    assert "- actual" in drift.message and "+ expected" in drift.message
    assert run_script("adr_lint", cwd=root).returncode == 1


def test_missing_or_invalid_area_is_a_frontmatter_error(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md", area="Bad Area")
    issues = adr_lint.lint(adr_lib.load_corpus(root))
    assert any(issue.rule == "area-slug" for issue in issues)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: 0001-a.md: area-slug: area `Bad Area` must match "
            "^[a-z][a-z0-9-]*$") in result.stdout


def test_schema_rejects_unknown_keys_and_bad_model_status(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md", (
        "---\n"
        "type: adr\ntitle: T\narea: core\nstatus: accepted\ndate: 2026-08-13\n"
        "invented: true\nmodel:\n  status: maybe\n  extra: x\n"
        "---\n\n# body\n"
    ))
    index(root)
    rules = [issue.rule for issue in adr_lint.lint(adr_lib.load_corpus(root))]
    assert rules.count("schema-unknown-key") == 2
    assert "model-status-enum" in rules
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: 0001-a.md: schema-unknown-key: frontmatter key `invented` "
            "is not allowed") in result.stdout
    assert ("ERROR: 0001-a.md: model-status-enum: model.status is outside the "
            "supported enum") in result.stdout


def test_reference_patterns_follow_the_configured_id_scheme(tmp_path):
    root = repo(tmp_path, config={"id_scheme": "sequential"})
    good(root, "0001-a.md", area="core", refines=["2026-01-01-wrong.md"])
    index(root)
    assert any(issue.rule == "reference-pattern"
               for issue in adr_lint.lint(adr_lib.load_corpus(root)))
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: 0001-a.md: reference-pattern: `refines` value "
            "`2026-01-01-wrong.md` does not match sequential") in result.stdout
    # The relaxed AdrRef pattern of task 3 leaves this to the scheme rule.
    assert "schema-type" not in result.stdout


def test_custom_schema_supplies_required_properties_and_enums(tmp_path):
    root = repo(tmp_path, config={"schema": "schema.json"})
    write(root / "schema.json", json.dumps({
        "required": ["type", "status", "date", "owner"],
        "properties": {
            "type": {"const": "adr"}, "status": {"enum": ["accepted"]},
            "date": {"type": "string"}, "owner": {"enum": ["platform"]},
        },
        "additionalProperties": False,
    }))
    good(root, "0001-a.md", area="core", status="proposed")
    index(root)
    rules = {issue.rule for issue in adr_lint.lint(adr_lib.load_corpus(root))}
    assert {"schema-required", "schema-enum"} <= rules
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "ERROR: 0001-a.md: schema-required: schema requires `owner`" in result.stdout


def test_missing_configured_schema_exits_with_a_path_specific_error(tmp_path):
    root = repo(tmp_path, config={"schema": "schemas/missing.json"})
    good(root, "0001-a.md", area="core")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 2
    assert "schemas/missing.json" in result.stderr


def test_default_schema_enforces_authored_value_shapes(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md", (
        "---\n"
        "type: note\ntitle:\narea: core\nstatus: accepted\ndate: 2026-8-13\n"
        "amended: yesterday\nrelated: elsewhere.md\nevidence: [valid, \"\"]\n"
        "model:\n  status: modeled\n  element: [system.component]\n"
        "---\n\n# body\n"
    ))
    index(root)
    rules = {issue.rule for issue in adr_lint.lint(adr_lib.load_corpus(root))}
    assert {"required-frontmatter", "schema-const", "schema-pattern", "schema-type",
            "schema-item-type"} <= rules
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "ERROR: 0001-a.md: schema-const: `type` must be `adr`" in result.stdout
    assert ("ERROR: 0001-a.md: schema-item-type: `evidence` contains an invalid "
            "item") in result.stdout
    assert result.stdout.rstrip().endswith("8 error(s), 0 warning(s)")


# --- Phase 0: DEF-1, the schema holds no date-only reference pattern -------

def test_a_sequential_reciprocal_pair_gives_no_schema_type(tmp_path):
    """The 0.2.0 schema held a date-only pattern on the 5 relation keys, so
    a correct sequential pair gave 2 false `schema-type` errors (DEF-1).
    D13: the test asserts that `schema-type` is absent, not that
    `schema-pattern` is present."""
    root = repo(tmp_path)
    good(root, "0001-a.md", status="superseded", superseded_by=["0002-b.md"])
    good(root, "0002-b.md", supersedes=["0001-a.md"])
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "schema-type" not in result.stdout
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_sequential_amend_and_refine_references_give_no_schema_type(tmp_path):
    """The probe covered `supersedes` and `superseded_by` only. `amends`,
    `amended_by` and `refines` held the same pattern (section 16)."""
    root = repo(tmp_path)
    good(root, "0001-a.md", amends=["0002-b.md"], refines=["0003-c.md"])
    good(root, "0002-b.md", amended_by=["0001-a.md"])
    good(root, "0003-c.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "schema-type" not in result.stdout
    assert result.stdout == "0 error(s), 0 warning(s)\n"


# --- Task 16: identity, cross-references, register staleness ----------------

def test_a_duplicate_numeric_prefix_is_an_error(tmp_path):
    root = repo(tmp_path)
    good(root, "0007-one.md")
    good(root, "0007-two.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "duplicate" in result.stdout.lower() and "0007" in result.stdout
    assert ("ERROR: 0007-two.md: duplicate-identity: `0007` is already used by "
            "0007-one.md") in result.stdout
    assert result.stdout.rstrip().endswith("1 error(s), 0 warning(s)")


def test_same_day_files_are_not_a_duplicate_under_the_date_scheme(tmp_path):
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    good(root, "2026-08-13-one.md")
    good(root, "2026-08-13-two.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_external_related_is_not_a_deterministic_reference(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md", area="core", related=["../../outside.md"])
    index(root)
    assert not [issue for issue in adr_lint.lint(adr_lib.load_corpus(root))
                if issue.rule in {"missing-reference", "dangling-reference",
                                  "escaping-reference"}]
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_partial_supersession_is_edge_specific(tmp_path):
    root = repo(tmp_path)
    write_adr(root, "0001-old.md", status="accepted", area="core",
              raw="superseded_by:\n  - 0002-new.md # partial\n")
    write_adr(root, "0002-new.md", status="accepted", area="core",
              raw="supersedes:\n  - 0001-old.md # partial\n")
    index(root)
    assert not [issue for issue in adr_lint.lint(adr_lib.load_corpus(root))
                if "supersede" in issue.rule]
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_a_partly_superseded_record_keeps_its_own_status(tmp_path):
    """One partial superseder is enough to leave the record live.

    A record replaced in part is still the decision of record for the parts
    nobody replaced, so it keeps `accepted`. Requiring `superseded` as soon as
    any one target is unmarked reports a record that is working as intended.
    """
    root = repo(tmp_path)
    write_adr(root, "0001-old.md", status="accepted", area="core",
              raw="superseded_by:\n  - 0002-a.md # partial\n  - 0003-b.md\n")
    good(root, "0002-a.md", area="core")
    good(root, "0003-b.md", area="core")
    index(root)
    rules = {issue.rule for issue in adr_lint.lint(adr_lib.load_corpus(root))}
    assert "supersede-pairing" not in rules
    # 0003-b.md is not marked partial, so the one-sided rule is the only
    # finding. The pairing rule stays silent.
    assert rules == {"one-sided-supersede"}
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "supersede-pairing" not in result.stdout
    assert ("ERROR: 0001-old.md: one-sided-supersede: 0003-b.md does not name "
            "0001-old.md in `supersedes`") in result.stdout


def test_a_wholly_superseded_record_requires_the_superseded_status(tmp_path):
    """With no target marked partial the replacement is total, and the rule
    still catches a record that claims to be live."""
    root = repo(tmp_path)
    write_adr(root, "0001-old.md", status="accepted", area="core",
              raw="superseded_by:\n  - 0002-a.md\n  - 0003-b.md\n")
    good(root, "0002-a.md", area="core")
    good(root, "0003-b.md", area="core")
    index(root)
    assert any(issue.rule == "supersede-pairing"
               for issue in adr_lint.lint(adr_lib.load_corpus(root)))
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: 0001-old.md: supersede-pairing: a non-partial `superseded_by` "
            "requires status `superseded`") in result.stdout


def test_amendment_reciprocity_resolution_and_self_reference(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md", area="core", amends=["0002-B.md", "0001-a.md"])
    good(root, "0002-b.md", area="core")
    index(root)
    rules = {issue.rule for issue in adr_lint.lint(adr_lib.load_corpus(root))}
    assert {"dangling-reference", "self-reference", "amendment-reciprocity"} <= rules
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: 0001-a.md: dangling-reference: `0002-B.md` resolves to no "
            "case-exact ADR path") in result.stdout
    assert "ERROR: 0001-a.md: self-reference: `amends` names this ADR" in result.stdout
    assert result.stdout.rstrip().endswith("3 error(s), 0 warning(s)")


def test_relationship_filename_collision_is_a_deterministic_lint_error(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md", area="core")
    write(root / "docs" / "adr" / "archive" / "0001-a.md", (
        "---\ntype: adr\ntitle: archived\narea: core\nstatus: superseded\n"
        "date: 2026-01-01\n---\n\n# body\n"
    ))
    index(root)
    issues = adr_lint.lint(adr_lib.load_corpus(root))
    assert [issue.rule for issue in issues].count("relationship-collision") == 1
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: -: relationship-collision: ADR filename collision: 0001-a.md"
            in result.stdout)


def test_a_collision_with_navigation_on_still_reports_every_issue(tmp_path):
    """The gated rules need the record set that the collision stopped. Reading
    the corpus a second time raised the same error out of `lint()` and threw
    away every issue the run had already found, including the collision."""
    root = repo(tmp_path)
    good(root, "0001-a.md", area="core")
    run_script("adr_index", "--write", "--nav", cwd=root)
    write(root / "docs" / "adr" / "archive" / "0001-a.md", (
        "---\ntype: adr\ntitle: archived\narea: core\nstatus: superseded\n"
        "date: 2026-01-01\n---\n\n# body\n"
    ))
    assert adr_index.navigation_is_on(adr_lib.load_corpus(root))
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1, result.stderr
    assert ("ERROR: -: relationship-collision: ADR filename collision: 0001-a.md"
            in result.stdout)


def test_an_archived_superseded_adr_does_not_require_a_replacement(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-live.md", area="core")
    write(root / "docs" / "adr" / "archive" / "0002-retired.md", (
        "---\ntype: adr\ntitle: archived\narea: core\nstatus: superseded\n"
        "date: 2026-01-01\n---\n\n# body\n"
    ))
    index(root)
    assert not [issue for issue in adr_lint.lint(adr_lib.load_corpus(root))
                if issue.rule == "supersede-pairing"]
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_a_one_sided_supersede_is_an_error(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-old.md", status="superseded", superseded_by=["0002-new.md"])
    good(root, "0002-new.md")            # does not name 0001 back
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1 and "one-sided" in result.stdout.lower()
    assert ("ERROR: 0001-old.md: one-sided-supersede: 0002-new.md does not name "
            "0001-old.md in `supersedes`") in result.stdout
    assert result.stdout.rstrip().endswith("1 error(s), 0 warning(s)")


def test_a_one_sided_supersede_is_an_error_from_the_supersedes_side_too(tmp_path):
    """A names B in `supersedes` and B stays silent. Checked from one side
    only, this whole shape lints clean."""
    root = repo(tmp_path)
    good(root, "0001-new.md", supersedes=["0002-old.md"])
    good(root, "0002-old.md")            # does not name 0001 back
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1 and "one-sided" in result.stdout.lower()
    assert ("ERROR: 0001-new.md: one-sided-supersede: 0002-old.md does not name "
            "0001-new.md in `superseded_by`") in result.stdout
    assert result.stdout.rstrip().endswith("1 error(s), 0 warning(s)")


def test_superseded_without_superseded_by_is_an_error_and_the_reverse(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-old.md", status="superseded")
    good(root, "0002-live.md", status="accepted", superseded_by=["0001-old.md"])
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert result.stdout.lower().count("supersede") >= 2
    assert ("ERROR: 0001-old.md: supersede-pairing: status is `superseded` with no "
            "`superseded_by`") in result.stdout
    assert ("ERROR: 0002-live.md: supersede-pairing: a non-partial `superseded_by` "
            "requires status `superseded`") in result.stdout
    assert result.stdout.rstrip().endswith("3 error(s), 0 warning(s)")


def test_a_stale_register_is_an_error(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md")
    index(root)
    good(root, "0002-b.md")              # written after the register
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1 and "register" in result.stdout.lower()
    assert "ERROR: 0002-b.md: orphan: ADR has no register bullet" in result.stdout
    assert "ERROR: README.md: ordering-drift:" in result.stdout
    assert result.stdout.rstrip().endswith("2 error(s), 0 warning(s)")


# --- Task 17: the proposed check, requirements.txt, the stdlib gate ---------

def proposed_repo(tmp_path, verdict="error"):
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md",
                                  "id_scheme": "sequential",
                                  "gate": {"proposed_verdict": verdict}})
    good(root, "0001-draft.md", status="proposed")
    good(root, "0002-settled.md", status="accepted")
    index(root)
    return root


def test_the_proposed_check_is_skipped_when_changed_is_absent(tmp_path):
    result = run_script("adr_lint", cwd=proposed_repo(tmp_path))
    assert result.returncode == 0
    assert "proposed" not in result.stdout.lower()
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_a_changed_proposed_adr_is_an_error(tmp_path):
    root = proposed_repo(tmp_path)
    result = run_script("adr_lint", "--changed", "docs/adr/0001-draft.md", cwd=root)
    assert result.returncode == 1
    assert "proposed" in result.stdout.lower()
    assert ("ERROR: 0001-draft.md: still-proposed: this ADR is still "
            "`proposed` in a changed file") in result.stdout


def test_a_changed_list_holding_no_adr_finds_nothing(tmp_path):
    root = proposed_repo(tmp_path)
    result = run_script("adr_lint", "--changed", "src/main.py", cwd=root)
    assert result.returncode == 0
    assert result.stdout == "0 error(s), 0 warning(s)\n"


def test_a_bare_base_name_in_changed_finds_the_adr(tmp_path):
    """Form 2 of D22: the gate can pass a base name only."""
    result = run_script("adr_lint", "--changed", "0001-draft.md",
                        cwd=proposed_repo(tmp_path))
    assert result.returncode == 1
    assert ("ERROR: 0001-draft.md: still-proposed: this ADR is still "
            "`proposed` in a changed file") in result.stdout


def test_a_changed_file_outside_the_corpus_with_an_adr_base_name_fires(tmp_path):
    """Form 3 of D22: `--changed` compares base names, so a file anywhere
    in the repository that carries the base name of a proposed ADR reports
    that ADR. This is a recorded limit (section 16), not a repair."""
    result = run_script("adr_lint", "--changed", "src/0001-draft.md",
                        cwd=proposed_repo(tmp_path))
    assert result.returncode == 1
    assert ("ERROR: 0001-draft.md: still-proposed: this ADR is still "
            "`proposed` in a changed file") in result.stdout


def test_the_verdict_downgrades_to_a_warning_and_switches_off(tmp_path):
    warn = run_script("adr_lint", "--changed", "docs/adr/0001-draft.md",
                      cwd=proposed_repo(tmp_path / "w", "warning"))
    assert warn.returncode == 0 and "WARN" in warn.stdout
    assert ("WARN: 0001-draft.md: still-proposed: this ADR is still "
            "`proposed` in a changed file") in warn.stdout
    off = run_script("adr_lint", "--changed", "docs/adr/0001-draft.md",
                     cwd=proposed_repo(tmp_path / "o", "off"))
    assert off.returncode == 0 and "proposed" not in off.stdout.lower()
    assert off.stdout == "0 error(s), 0 warning(s)\n"


def test_the_vendored_scripts_import_only_the_standard_library(scripts_dir):
    """ADRs land in repos that are not Python projects; `python3 adr_lint.py`
    with no install step is the difference between a gate that is enabled and
    one that is not."""
    local = {"adr_lib", "adr_index", "adr_new", "adr_migrate"}
    for path in sorted(scripts_dir.glob("*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        guarded = {node for guard in ast.walk(tree) if isinstance(guard, ast.Try)
                   for node in ast.walk(guard) if isinstance(node, (ast.Import, ast.ImportFrom))}
        for node in ast.walk(tree):
            if not isinstance(node, (ast.Import, ast.ImportFrom)):
                continue
            names = ([alias.name.split(".")[0] for alias in node.names]
                     if isinstance(node, ast.Import) else [(node.module or "").split(".")[0]])
            for name in names:
                if name in sys.stdlib_module_names or name in local:
                    continue
                assert False, f"{path.name} imports third-party `{name}`"


# --- Phase 1: the schema subset ---------------------------------------------

def test_a_raw_linkml_schema_stops_the_lint_with_schema_unsupported(tmp_path):
    """The shipped file is flattened before it ships. A corpus that points the
    `schema` config key at a raw LinkML output must get a message, not the
    TypeError that _matches_type raises on a list key."""
    root = repo(tmp_path, config={"schema": "raw.json"})
    write(root / "raw.json", json.dumps({
        "type": "object",
        "properties": {"related": {"type": "array",
                                   "items": {"$ref": "#/$defs/DocRef"}}},
        "$defs": {"DocRef": {"type": "string"}},
    }))
    good(root, "0001-a.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 2
    assert "schema-unsupported: $ref at /properties/related/items/$ref" in result.stderr


def test_a_type_list_in_the_active_schema_is_unsupported(tmp_path):
    root = repo(tmp_path, config={"schema": "raw.json"})
    write(root / "raw.json", json.dumps({
        "type": "object",
        "properties": {"title": {"type": ["string", "null"]}},
    }))
    good(root, "0001-a.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 2
    assert "schema-unsupported: type at /properties/title/type" in result.stderr


def test_a_hand_written_adopter_schema_with_title_and_format_still_loads(tmp_path):
    """`title`, `description` and `format` were always ignored, not refused.
    A refusal would turn every adopter schema red on upgrade."""
    root = repo(tmp_path, config={"schema": "adopter.json"})
    write(root / "adopter.json", json.dumps({
        "title": "Adr", "description": "an adopter schema", "type": "object",
        "required": ["type", "status", "date"],
        "properties": {
            "type": {"const": "adr"},
            "status": {"enum": list(adr_lib.STATUSES)},
            "date": {"type": "string"},
            "title": {"type": "string", "minLength": 1},
            "area": {"type": "string"},
            "parent_spec": {"type": "string", "format": "uri-reference"},
        },
        "additionalProperties": False,
    }))
    good(root, "0001-a.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0, result.stdout + result.stderr


# --- Phase 1: the evidence check in code ------------------------------------

def _evidence_schema(items):
    return json.dumps({
        "type": "object",
        "required": ["type", "status", "date"],
        "properties": {
            "type": {"const": "adr"},
            "status": {"enum": list(adr_lib.STATUSES)},
            "date": {"type": "string"},
            "title": {"type": "string"},
            "area": {"type": "string"},
            "evidence": {"type": "array", "items": items},
        },
        "additionalProperties": False,
    })


def _corpus_with_an_empty_evidence_item(tmp_path, items):
    root = repo(tmp_path, config={"schema": "evidence.json"})
    write(root / "evidence.json", _evidence_schema(items))
    write(root / "docs" / "adr" / "0001-a.md", (
        "---\ntype: adr\ntitle: T\narea: core\nstatus: accepted\ndate: 2026-08-13\n"
        "evidence: [valid, \"\"]\n---\n\n# body\n"))
    index(root)
    return [issue.rule for issue in adr_lint.lint(adr_lib.load_corpus(root))]


def test_an_empty_evidence_item_without_min_length(tmp_path):
    """LinkML never writes `minLength`, so the generated schema cannot reject
    an empty item. The finding and its id stay, in code (D14)."""
    rules = _corpus_with_an_empty_evidence_item(tmp_path, {"type": "string"})
    assert rules.count("schema-item-type") == 1
    assert "schema-min-length" not in rules


def test_an_empty_evidence_item_with_min_length_is_reported_once(tmp_path):
    """The schema check and the code check must not both fire."""
    rules = _corpus_with_an_empty_evidence_item(
        tmp_path, {"type": "string", "minLength": 1})
    assert rules.count("schema-item-type") == 1


# --- Phase 1: the description rules -----------------------------------------

def _with_description(tmp_path, value, title="Order state machine"):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md", (
        "---\ntype: adr\n"
        f"title: {title}\n"
        f"description: {value}\n"
        "area: core\nstatus: accepted\ndate: 2026-08-13\n"
        "---\n\n# body\n"))
    index(root)
    return adr_lint.lint(adr_lib.load_corpus(root))


def _description_findings(issues):
    return [issue for issue in issues if issue.rule.startswith("description")]


def test_a_description_over_thirty_words_is_an_error(tmp_path):
    issues = _with_description(tmp_path, " ".join(["word"] * 31))
    finding = next(issue for issue in issues if issue.rule == "description-length")
    assert finding.level == "ERROR"
    assert finding.file == "0001-a.md"
    assert finding.message == "`description` has 31 words; the limit is 30"


def test_a_description_of_thirty_words_is_clean(tmp_path):
    assert _description_findings(
        _with_description(tmp_path, " ".join(["word"] * 30))) == []


def test_a_description_with_a_line_break_is_an_error(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md", (
        "---\ntype: adr\ntitle: Order state machine\n"
        "description: |-\n  first line\n  second line\n"
        "area: core\nstatus: accepted\ndate: 2026-08-13\n"
        "---\n\n# body\n"))
    index(root)
    issues = adr_lint.lint(adr_lib.load_corpus(root))
    assert [issue.message for issue in issues if issue.rule == "description-length"] \
        == ["`description` must be one line"]


def test_a_description_equal_to_the_title_is_a_warning(tmp_path):
    issues = _with_description(tmp_path, "Order state machine")
    finding = next(issue for issue in issues
                   if issue.rule == "description-repeats-title")
    assert finding.level == "WARN"
    assert finding.file == "0001-a.md"
    assert finding.message == "`description` repeats the title"


def test_the_title_with_three_more_words_still_repeats_it(tmp_path):
    issues = _with_description(tmp_path, "Order state machine, in one place")
    assert any(issue.rule == "description-repeats-title" for issue in issues)


def test_the_title_with_four_more_words_does_not_repeat_it(tmp_path):
    issues = _with_description(tmp_path, "Order state machine, in exactly one place")
    assert _description_findings(issues) == []


def test_a_description_that_says_something_new_is_clean(tmp_path):
    issues = _with_description(
        tmp_path, "Read this before you add a second writer to the orders table.")
    assert _description_findings(issues) == []


def test_an_adr_with_no_description_gives_no_finding(tmp_path):
    """`description` is optional (D3). `description-missing` is the gated
    rule, and it arrives with navigation in phase 2."""
    root = repo(tmp_path)
    good(root, "0001-a.md")
    index(root)
    assert _description_findings(adr_lint.lint(adr_lib.load_corpus(root))) == []


# --- Task 14: the lint gate, index-drift and links-drift --------------------

def nav_corpus(tmp_path):
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md", "id_scheme": "date",
                                  "archive_dir": "archive"})
    write(root / "docs" / "adr" / "2026-01-01-a.md",
          "---\ntype: adr\ntitle: A\ndescription: One writer per aggregate.\narea: core\n"
          "status: accepted\ndate: 2026-01-01\n---\n\n# T\n\n## Decision\n\nText.\n")
    write(root / "docs" / "adr" / "archive" / "2025-01-01-old.md",
          "---\ntype: adr\ntitle: Old\ndescription: The old cache.\narea: core\n"
          "status: deprecated\ndate: 2025-01-01\n---\n\n# T\n\n## Decision\n\nText.\n")
    run_script("adr_index", "--write", "--nav", cwd=root)
    return root


def rules_of(result):
    return [line.split(": ")[2] for line in result.stdout.splitlines()
            if line.startswith(("ERROR: ", "WARN: "))]


def test_a_navigated_corpus_lints_clean(tmp_path):
    result = run_script("adr_lint", cwd=nav_corpus(tmp_path))
    assert result.returncode == 0, result.stdout
    assert result.stdout.splitlines()[-1] == "0 error(s), 0 warning(s)"


def test_a_stale_index_is_an_index_drift_error(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "index.md",
          '---\nokf_version: "0.2"\n---\n\n# Architecture decision records\n\n## Areas\n')
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert "ERROR: index.md: index-drift: generated index is stale; run /adr:index" in result.stdout


def test_a_stale_archive_index_is_reported_on_its_own_path(tmp_path):
    root = nav_corpus(tmp_path)
    (root / "docs" / "adr" / "archive" / "index.md").unlink()
    result = run_script("adr_lint", cwd=root)
    assert ("ERROR: archive/index.md: index-drift: generated index is stale; "
            "run /adr:index") in result.stdout


def test_a_stale_and_a_missing_links_file_give_their_own_message(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "links.tsv", "source\n")
    assert ("ERROR: links.tsv: links-drift: links.tsv is stale; run /adr:index"
            in run_script("adr_lint", cwd=root).stdout)
    (root / "docs" / "adr" / "links.tsv").unlink()
    assert ("ERROR: links.tsv: links-drift: links.tsv is missing; run /adr:index"
            in run_script("adr_lint", cwd=root).stdout)


def test_with_no_index_no_gated_rule_fires(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md")
    index(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0
    assert set(adr_lint.GATED_RULES) == {"index-drift", "links-drift", "graph-cycle",
                                         "target-missing", "yaml-shape",
                                         "description-missing"}
    assert not set(adr_lint.GATED_RULES) & set(rules_of(result))
    assert not (root / "docs" / "adr" / "index.md").exists()


# --- Task 15: graph-cycle and target-missing --------------------------------

def navigate(root):
    """Turn navigation on with the bytes the corpus has now."""
    return run_script("adr_index", "--write", "--nav", cwd=root)


def test_a_superseded_by_cycle_is_a_graph_cycle_error(tmp_path):
    """The 2 ADRs of the cycle are archived, so the register stays current and
    the only new finding is the cycle."""
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: superseded\ndate: 2026-02-01\n"
          "superseded_by: 2026-03-01-c.md\nsupersedes: 2026-03-01-c.md\n---\n\n# T\n")
    write(root / "docs" / "adr" / "archive" / "2026-03-01-c.md",
          "---\ntype: adr\ntitle: C\narea: core\nstatus: superseded\ndate: 2026-03-01\n"
          "superseded_by: 2026-02-01-b.md\nsupersedes: 2026-02-01-b.md\n---\n\n# T\n")
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: archive/2026-02-01-b.md: graph-cycle: `superseded_by` chain is a cycle: "
            "archive/2026-02-01-b.md -> archive/2026-03-01-c.md -> archive/2026-02-01-b.md"
            ) in result.stdout
    assert "index-drift" not in result.stdout


def test_a_missing_target_is_a_warning_on_the_source_adr(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          "refines: 2026-12-31-absent.md\n---\n\n# T\n")
    navigate(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 0                      # a WARN never blocks
    assert ("WARN: 2026-02-01-b.md: target-missing: `refines` target does not exist: "
            "2026-12-31-absent.md") in result.stdout


def test_a_url_target_and_a_document_target_give_no_warning(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "runbook.md", "# Runbook\n")
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          "deviates_from: https://example.org/rule\n"
          "related:\n  - ../runbook.md\n---\n\n# T\n")
    navigate(root)
    result = run_script("adr_lint", cwd=root)
    assert "target-missing" not in result.stdout


def test_a_missing_target_on_an_archived_adr_names_its_archive_path(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2025-01-01-old.md",
          "---\ntype: adr\ntitle: Old\ndescription: The old cache.\narea: core\n"
          "status: deprecated\ndate: 2025-01-01\n"
          "parent_spec: docs/specs/gone.md\n---\n\n# T\n\n## Decision\n\nText.\n")
    navigate(root)
    result = run_script("adr_lint", cwd=root)
    assert ("WARN: archive/2025-01-01-old.md: target-missing: `parent_spec` target does not "
            "exist: docs/specs/gone.md") in result.stdout


# --- Task 16: yaml-shape ----------------------------------------------------

def test_the_good_yaml_forms_give_no_finding(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          "related:\n  - ../runbook.md\n"
          "evidence: [src/a.ts, src/b.ts]\n---\n\n# T\n")
    write(root / "docs" / "runbook.md", "# Runbook\n")
    navigate(root)
    assert "yaml-shape" not in run_script("adr_lint", cwd=root).stdout


def test_a_mapping_shaped_list_item_and_a_quoted_flow_list_are_errors(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          "related:\n  - rel: docs/a.md\n"
          'evidence: ["a, b", c]\n---\n\n# T\n')
    navigate(root)
    result = run_script("adr_lint", cwd=root)
    assert result.returncode == 1
    assert ("ERROR: 2026-02-01-b.md: yaml-shape: `related` has a YAML form that the parser "
            "reads wrongly: rel: docs/a.md") in result.stdout
    assert ("ERROR: 2026-02-01-b.md: yaml-shape: `evidence` has a YAML form that the parser "
            'reads wrongly: ["a, b", c]') in result.stdout


def test_a_quoted_list_item_is_not_a_mapping(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          'evidence:\n  - "rel: docs/a.md"\n---\n\n# T\n')
    navigate(root)
    assert "yaml-shape" not in run_script("adr_lint", cwd=root).stdout


def test_yaml_shape_is_gated(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "0001-a.md",
          "---\ntype: adr\ntitle: A\narea: core\nstatus: accepted\ndate: 2026-01-01\n"
          'evidence: ["a, b", c]\n---\n\n# T\n')
    index(root)
    assert "yaml-shape" not in run_script("adr_lint", cwd=root).stdout


# --- Task 17: description-missing -------------------------------------------

def test_a_changed_adr_with_no_description_is_a_warning(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          "---\n\n# T\n")
    navigate(root)
    result = run_script("adr_lint", "--changed", "docs/adr/2026-02-01-b.md", cwd=root)
    assert result.returncode == 0
    assert ("WARN: 2026-02-01-b.md: description-missing: `description` is missing"
            in result.stdout)


def test_a_draft_description_asks_for_a_review(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\ndescription: The cache keeps one entry. # draft\n"
          "area: core\nstatus: accepted\ndate: 2026-02-01\n---\n\n# T\n")
    navigate(root)
    result = run_script("adr_lint", "--changed", "2026-02-01-b.md", cwd=root)
    assert ("WARN: 2026-02-01-b.md: description-missing: `description` is a draft; review it "
            "and remove the comment") in result.stdout


def test_an_adr_that_changed_is_not_named_gives_no_warning(tmp_path):
    root = nav_corpus(tmp_path)
    write(root / "docs" / "adr" / "2026-02-01-b.md",
          "---\ntype: adr\ntitle: B\narea: core\nstatus: accepted\ndate: 2026-02-01\n"
          "---\n\n# T\n")
    navigate(root)
    assert "description-missing" not in run_script("adr_lint", cwd=root).stdout
    assert "description-missing" not in run_script(
        "adr_lint", "--changed", "docs/other.md", cwd=root).stdout


def test_description_missing_is_gated(tmp_path):
    root = repo(tmp_path)
    good(root, "0001-a.md")
    index(root)
    result = run_script("adr_lint", "--changed", "0001-a.md", cwd=root)
    assert "description-missing" not in result.stdout
