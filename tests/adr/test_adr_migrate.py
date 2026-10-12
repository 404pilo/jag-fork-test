"""adr_migrate.py: --frontmatter folds bold-key metadata (filenames untouched
by default), --rename normalises names and carries the register summary,
--dry-run writes nothing, and the tool commits nothing."""
import json
import subprocess

import adr_vendored_conftest as _cf

load_script = _cf.load_script
run_script = _cf.run_script
write = _cf.write

adr_lib = load_script("adr_lib")

LEGACY = (
    "# ADR-002: Gateway auth\n"
    "\n"
    "**Status:** Accepted\n"
    "**Date:** 2026-02-26\n"
    "**Supersedes:** ADR-001 (which documented the original architecture)\n"
    "\n"
    "---\n"
    "\n"
    "## Context\n"
    "\n"
    "Auth was per service.\n"
)


def repo(tmp_path, id_scheme="sequential"):
    (tmp_path / ".git").mkdir()
    (tmp_path / "docs" / "adr").mkdir(parents=True)
    write(tmp_path / ".claude" / "adr.json",
          json.dumps({"version": 1, "dir": "docs/adr",
                      "register": "docs/adr/README.md", "id_scheme": id_scheme}))
    return tmp_path


# --- Task 35: --frontmatter folds bold-key headers ---------------------------

def test_bold_keys_fold_into_frontmatter_and_the_filename_is_untouched(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    result = run_script("adr_migrate", cwd=root)
    assert result.returncode == 0, result.stderr
    path = root / "docs" / "adr" / "ADR-002-auth.md"
    assert path.is_file()                       # name unchanged by default
    adr = adr_lib.parse_adr(path)
    assert adr.source == "frontmatter"
    assert adr.status == "accepted"
    assert adr.date == "2026-02-26"
    assert adr.meta["title"] == "Gateway auth"
    assert "id:" not in path.read_text(encoding="utf-8")


def test_the_heading_is_kept(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    run_script("adr_migrate", cwd=root)
    text = (root / "docs" / "adr" / "ADR-002-auth.md").read_text(encoding="utf-8")
    assert "# ADR-002: Gateway auth" in text
    assert "**Status:**" not in text            # the bold keys are gone
    assert "Auth was per service." in text      # the body survives


def test_an_unresolvable_reference_is_left_for_the_lint(tmp_path):
    """Never guessed at. The converter cannot know which file
    'ADR-001 (which documented the original architecture)' means."""
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    result = run_script("adr_migrate", cwd=root)
    text = (root / "docs" / "adr" / "ADR-002-auth.md").read_text(encoding="utf-8")
    assert "ADR-001 (which documented the original architecture)" in text
    assert "unresolved" in result.stdout.lower()


def test_a_bold_key_shaped_line_in_the_body_is_not_folded_or_dropped(tmp_path):
    """A bold-key-shaped line past parse_legacy's 20-line window is never
    read as metadata, so fold() must not delete it from the body either --
    doing so would be silent content loss with no matching frontmatter
    change."""
    filler = "\n".join(f"Body line {n}." for n in range(1, 20))
    text = (
        "# ADR-002: Gateway auth\n"
        "\n"
        "**Status:** Accepted\n"
        "**Date:** 2026-02-26\n"
        "\n"
        "## Context\n"
        "\n"
        f"{filler}\n"
        "\n"
        "The old doc quoted `**Status:** Rejected` verbatim here.\n"
    )
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", text)
    result = run_script("adr_migrate", cwd=root)
    assert result.returncode == 0, result.stderr
    path = root / "docs" / "adr" / "ADR-002-auth.md"
    written = path.read_text(encoding="utf-8")
    adr = adr_lib.parse_adr(path)
    assert adr.status == "accepted"             # real status, not overwritten
    assert "**Status:** Rejected" in written     # quoted body line survives


def test_a_file_already_carrying_frontmatter_is_left_alone(tmp_path):
    root = repo(tmp_path)
    before = "---\ntype: adr\nstatus: accepted\ndate: 2026-08-13\ntitle: T\n---\n\n# T\n"
    write(root / "docs" / "adr" / "0001-a.md", before)
    run_script("adr_migrate", cwd=root)
    assert (root / "docs" / "adr" / "0001-a.md").read_text(encoding="utf-8") == before


# --- Task 36: --rename carries the register summary --------------------------

def test_rename_normalises_the_filename(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    result = run_script("adr_migrate", "--rename", cwd=root)
    assert result.returncode == 0, result.stderr
    assert (root / "docs" / "adr" / "0002-auth.md").is_file()
    assert not (root / "docs" / "adr" / "ADR-002-auth.md").exists()


def test_rename_carries_the_hand_authored_summary_to_the_new_key(tmp_path):
    """The register keys summaries by filename. Without this rewrite the
    summary is dropped and re-seeded as a TODO on the next regeneration."""
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    run_script("adr_index", "--write", cwd=root)
    register = root / "docs" / "adr" / "README.md"
    write(register, register.read_text("utf-8").replace(
        "<!-- TODO: summary -->", "Auth sits at the gateway"))

    assert run_script("adr_migrate", "--rename", cwd=root).returncode == 0
    after_migrate = register.read_text("utf-8")
    assert "(./0002-auth.md)" in after_migrate

    assert run_script("adr_index", "--write", cwd=root).returncode == 0
    final = register.read_text("utf-8")
    assert "Auth sits at the gateway" in final
    assert "<!-- TODO: summary -->" not in final


def test_rename_refuses_when_the_new_name_is_taken(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    write(root / "docs" / "adr" / "0002-auth.md",
          "---\ntype: adr\nstatus: accepted\ndate: 2026-08-13\ntitle: T\n---\n\n# T\n")
    result = run_script("adr_migrate", "--rename", cwd=root)
    assert result.returncode == 1
    assert (root / "docs" / "adr" / "ADR-002-auth.md").is_file()


def test_rename_edits_only_matching_reference_items(tmp_path):
    root = repo(tmp_path)
    write(root / "docs" / "adr" / "ADR-002-old.md", LEGACY)
    before = (
        "---\ntype: adr\ntitle: Consumer\narea: core\nstatus: accepted\n"
        "date: 2026-01-05\namended: 2026-01-06\n"
        "supersedes:\n  - ADR-002-old.md # partial — preserve\n"
        "evidence:\n  - measured\nmodel:\n  status: modeled\n"
        "---\n\n# Consumer\n"
    )
    consumer = write(root / "docs" / "adr" / "0003-consumer.md", before)
    result = run_script("adr_migrate", "--rename", cwd=root)
    assert result.returncode == 0, result.stderr
    after = consumer.read_text("utf-8")
    assert "0002-old.md # partial — preserve" in after
    assert after.replace("0002-old.md", "ADR-002-old.md") == before


def test_date_scheme_remains_a_complete_no_op_for_frontmatter_files(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    before = "---\ntype: adr\ntitle: T\narea: core\nstatus: accepted\ndate: 2026-01-05\n---\n"
    path = write(root / "docs" / "adr" / "2026-01-05-t.md", before)
    assert run_script("adr_migrate", "--rename", cwd=root).returncode == 0
    assert path.read_text("utf-8") == before


# --- Task 37: --dry-run writes nothing, the tool commits nothing -------------

def test_dry_run_reports_and_writes_nothing(tmp_path):
    root = repo(tmp_path)
    before = LEGACY
    write(root / "docs" / "adr" / "ADR-002-auth.md", before)
    result = run_script("adr_migrate", "--rename", "--dry-run", cwd=root)
    assert result.returncode == 0
    assert "ADR-002-auth.md" in result.stdout and "0002-auth.md" in result.stdout
    assert (root / "docs" / "adr" / "ADR-002-auth.md").read_text("utf-8") == before


def test_migration_commits_nothing(tmp_path):
    root = repo(tmp_path)
    subprocess.run(["git", "init", "-q"], cwd=str(root))
    write(root / "docs" / "adr" / "ADR-002-auth.md", LEGACY)
    run_script("adr_migrate", cwd=root)
    log = subprocess.run(["git", "log", "--oneline"], cwd=str(root),
                         capture_output=True, text=True)
    assert log.stdout.strip() == ""


# --- phase 4: --descriptions -------------------------------------------------

DECISION = (
    "---\ntype: adr\ntitle: Cache entries expire after one hour\narea: cache\n"
    "status: accepted\ndate: 2026-03-03\n---\n\n"
    "# ADR-2026-03-03: Cache entries expire after one hour\n\n"
    "## Decision\n\n"
    "Each cache entry expires after one hour. A longer window hid stale rows.\n"
)


def test_descriptions_dry_run_prints_the_plan_and_writes_nothing(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    path = write(root / "docs" / "adr" / "2026-03-03-cache.md", DECISION)
    before = path.read_text("utf-8")
    result = run_script("adr_migrate", "--descriptions", cwd=root)
    assert result.returncode == 0, result.stderr
    assert ("plan: 2026-03-03-cache.md -> "
            "Each cache entry expires after one hour.") in result.stdout
    assert path.read_text("utf-8") == before


def test_descriptions_skips_a_file_that_already_has_one(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    write(root / "docs" / "adr" / "2026-03-03-cache.md",
          DECISION.replace("area: cache\n", "description: Written by hand.\narea: cache\n"))
    result = run_script("adr_migrate", "--descriptions", cwd=root)
    assert result.returncode == 0, result.stderr
    assert "plan:" not in result.stdout
    assert "descriptions: 0 to write, 0 with no text" in result.stdout


def test_descriptions_reports_a_file_whose_body_gives_no_text(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    empty = (
        "---\ntype: adr\ntitle: Placeholder\narea: orders\n"
        "status: proposed\ndate: 2026-03-07\n---\n\n"
        "# ADR-2026-03-07: Placeholder\n\n## Decision\n\n## Consequences\n\nNone yet.\n"
    )
    write(root / "docs" / "adr" / "2026-03-07-placeholder.md", empty)
    result = run_script("adr_migrate", "--descriptions", cwd=root)
    assert result.returncode == 0, result.stderr
    assert "skip: 2026-03-07-placeholder.md (no text found)" in result.stdout
    assert "descriptions: 0 to write, 1 with no text" in result.stdout


def test_descriptions_apply_inserts_a_draft_line_after_title(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    path = write(root / "docs" / "adr" / "2026-03-03-cache.md", DECISION)
    result = run_script("adr_migrate", "--descriptions", "--apply", cwd=root)
    assert result.returncode == 0, result.stderr
    assert ("title: Cache entries expire after one hour\n"
            "description: Each cache entry expires after one hour. # draft\n"
            ) in path.read_text("utf-8")
    assert "described: 2026-03-03-cache.md" in result.stdout
    assert "descriptions: 1 to write, 0 with no text" in result.stdout


def test_a_second_descriptions_run_makes_no_change(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    path = write(root / "docs" / "adr" / "2026-03-03-cache.md", DECISION)
    assert run_script("adr_migrate", "--descriptions", "--apply", cwd=root).returncode == 0
    once = path.read_text("utf-8")
    second = run_script("adr_migrate", "--descriptions", "--apply", cwd=root)
    assert second.returncode == 0, second.stderr
    assert path.read_text("utf-8") == once
    assert "descriptions: 0 to write, 0 with no text" in second.stdout


def test_descriptions_apply_writes_nothing_for_a_body_with_no_text(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    empty = (
        "---\ntype: adr\ntitle: Placeholder\narea: orders\n"
        "status: proposed\ndate: 2026-03-07\n---\n\n"
        "# ADR-2026-03-07: Placeholder\n\n## Decision\n\n## Consequences\n\nNone yet.\n"
    )
    path = write(root / "docs" / "adr" / "2026-03-07-placeholder.md", empty)
    assert run_script("adr_migrate", "--descriptions", "--apply", cwd=root).returncode == 0
    assert path.read_text("utf-8") == empty


def test_descriptions_commits_nothing(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    subprocess.run(["git", "init", "-q"], cwd=str(root))
    write(root / "docs" / "adr" / "2026-03-03-cache.md", DECISION)
    run_script("adr_migrate", "--descriptions", "--apply", cwd=root)
    log = subprocess.run(["git", "log", "--oneline"], cwd=str(root),
                         capture_output=True, text=True)
    assert log.stdout.strip() == ""
    staged = subprocess.run(["git", "diff", "--cached", "--name-only"], cwd=str(root),
                            capture_output=True, text=True)
    assert staged.stdout.strip() == ""


# --- Task 5: --descriptions refuses a schema without `description` -----------

NO_DESCRIPTION_SCHEMA = json.dumps({
    "type": "object",
    "additionalProperties": False,
    "properties": {"type": {"const": "adr"}, "title": {"type": "string"},
                   "area": {"type": "string"}, "status": {"type": "string"},
                   "date": {"type": "string"}},
    "required": ["type", "status", "date"],
})


def test_descriptions_refuses_a_schema_that_does_not_permit_description(tmp_path):
    """An adopter with an old vendored schema would get `schema-unknown-key`
    on each file. The mode stops before it writes."""
    root = repo(tmp_path, id_scheme="date")
    path = write(root / "docs" / "adr" / "2026-03-03-cache.md", DECISION)
    write(root / "schemas" / "adr.schema.json", NO_DESCRIPTION_SCHEMA)
    write(root / ".claude" / "adr.json",
          json.dumps({"version": 1, "dir": "docs/adr",
                      "register": {"path": "docs/adr/README.md", "exclude": []},
                      "id_scheme": "date", "schema": "schemas/adr.schema.json"}))
    result = run_script("adr_migrate", "--descriptions", "--apply", cwd=root)
    assert result.returncode == 2
    assert ("ERROR: the active schema does not permit description; "
            "run /adr:setup --refresh or add the property to your schema") in result.stderr
    assert "description:" not in path.read_text("utf-8")


def test_descriptions_refuses_a_schema_file_that_does_not_exist(tmp_path):
    root = repo(tmp_path, id_scheme="date")
    write(root / "docs" / "adr" / "2026-03-03-cache.md", DECISION)
    write(root / ".claude" / "adr.json",
          json.dumps({"version": 1, "dir": "docs/adr",
                      "register": {"path": "docs/adr/README.md", "exclude": []},
                      "id_scheme": "date", "schema": "schemas/gone.json"}))
    result = run_script("adr_migrate", "--descriptions", cwd=root)
    assert result.returncode == 2
    assert "schema does not exist" in result.stderr
