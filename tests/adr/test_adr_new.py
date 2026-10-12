import json

import adr_vendored_conftest as _cf

load_script = _cf.load_script
run_script = _cf.run_script
write = _cf.write

adr_lib = load_script("adr_lib")
adr_new = load_script("adr_new")


def repo(tmp_path, config=None, live=(), archived=()):
    (tmp_path / ".git").mkdir()
    (tmp_path / "docs" / "adr").mkdir(parents=True)
    write(tmp_path / ".claude" / "adr.json",
          json.dumps(config or {"version": 1, "dir": "docs/adr",
                                "register": "docs/adr/README.md",
                                "id_scheme": "sequential"}))
    for name in live:
        write(tmp_path / "docs" / "adr" / name, "# stub\n")
    for name in archived:
        write(tmp_path / "docs" / "adr" / "archive" / name, "# stub\n")
    return tmp_path


# --- Task 9: the ID allocator ------------------------------------------------

def test_the_next_id_is_zero_padded_to_four(tmp_path):
    corpus = adr_lib.load_corpus(repo(tmp_path, live=["0001-a.md", "0009-b.md"]))
    assert adr_new.next_id(corpus) == "0010"


def test_an_empty_corpus_starts_at_0001(tmp_path):
    assert adr_new.next_id(adr_lib.load_corpus(repo(tmp_path))) == "0001"


def test_a_retired_prefix_is_never_reused(tmp_path):
    root = repo(tmp_path,
                config={"version": 1, "dir": "docs/adr",
                        "register": "docs/adr/README.md",
                        "id_scheme": "sequential", "archive_dir": "archive"},
                live=["0001-a.md"], archived=["0007-old.md"])
    assert adr_new.next_id(adr_lib.load_corpus(root)) == "0008"


def test_the_date_scheme_uses_the_date(tmp_path):
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    assert adr_new.next_id(adr_lib.load_corpus(root), today="2026-08-13") == "2026-08-13"


# --- Task 10: adr_new.py writes the ADR and refuses a collision --------------

def test_it_writes_a_frontmatter_adr_with_the_five_headings(tmp_path):
    root = repo(tmp_path)
    result = run_script("adr_new", "--title", "Order state machine",
                        "--area", "orders", "--date", "2026-08-13", cwd=root)
    assert result.returncode == 0, result.stderr
    path = root / "docs" / "adr" / "0001-order-state-machine.md"
    assert path.is_file()
    assert result.stdout.strip().endswith("0001-order-state-machine.md")
    text = path.read_text(encoding="utf-8")
    assert text.startswith("---\ntype: adr\ntitle: Order state machine\n"
                          "area: orders\nstatus: proposed\ndate: 2026-08-13\n")
    assert "\nid:" not in text
    assert "area: orders" in text
    assert "# ADR-0001: Order state machine" in text
    for heading in ("## Context", "## Decision", "## Why",
                    "## Alternatives considered", "## Consequences"):
        assert heading in text


def test_a_sequential_collision_is_refused(tmp_path):
    root = repo(tmp_path, live=["0001-taken.md"])
    (root / "docs" / "adr" / "0002-next.md").unlink(missing_ok=True)
    result = run_script("adr_new", "--title", "Taken", "--area", "core",
                        "--id", "0001", cwd=root)
    assert result.returncode == 1
    assert "0001" in result.stderr


def test_same_day_files_with_different_slugs_are_not_a_collision(tmp_path):
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    first = run_script("adr_new", "--title", "One", "--area", "core",
                       "--date", "2026-08-13", cwd=root)
    second = run_script("adr_new", "--title", "Two", "--area", "core",
                        "--date", "2026-08-13", cwd=root)
    assert (first.returncode, second.returncode) == (0, 0), second.stderr
    assert (root / "docs" / "adr" / "2026-08-13-one.md").is_file()
    assert (root / "docs" / "adr" / "2026-08-13-two.md").is_file()


def test_an_identical_full_filename_is_refused_under_the_date_scheme(tmp_path):
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    run_script("adr_new", "--title", "One", "--area", "core",
               "--date", "2026-08-13", cwd=root)
    again = run_script("adr_new", "--title", "One", "--area", "core",
                       "--date", "2026-08-13", cwd=root)
    assert again.returncode == 1
    assert "already exists" in again.stderr


# --- Task 11: schema-complete ADR creation ----------------------------------

def test_area_is_required(tmp_path):
    root = repo(tmp_path)
    result = run_script("adr_new", "--title", "No area", cwd=root)
    assert result.returncode == 2
    assert "--area" in result.stderr


def test_optional_reference_metadata_is_written_in_contract_order(tmp_path):
    root = repo(tmp_path)
    result = run_script(
        "adr_new", "--title", "Gateway", "--area", "auth",
        "--date", "2026-01-05", "--parent-spec", "../spec.md",
        "--evidence", "Measured 2026-01-05", "--model-status", "modeled",
        "--model-element", "system.gateway", cwd=root,
    )
    assert result.returncode == 0, result.stderr
    front, _ = adr_lib.split_frontmatter(
        (root / "docs" / "adr" / "0001-gateway.md").read_text("utf-8")
    )
    keys = [line.split(":", 1)[0] for line in front.splitlines()
            if line and not line[0].isspace()]
    assert keys == [key for key in adr_lib.KEY_ORDER if key in keys]
    data = adr_lib.load_frontmatter(front)
    assert data["evidence"] == ["Measured 2026-01-05"]
    assert data["model"] == {"status": "modeled", "element": "system.gateway"}


# --- Phase 1: the description flag ------------------------------------------

def test_the_description_flag_writes_the_line_below_the_title(tmp_path):
    root = repo(tmp_path)
    result = run_script("adr_new", "--title", "Order state machine",
                        "--area", "orders", "--date", "2026-08-13",
                        "--description", "Read this before you add a second writer.",
                        cwd=root)
    assert result.returncode == 0, result.stderr
    text = (root / "docs" / "adr" / "0001-order-state-machine.md").read_text("utf-8")
    assert text.startswith(
        "---\ntype: adr\ntitle: Order state machine\n"
        "description: Read this before you add a second writer.\n"
        "area: orders\nstatus: proposed\ndate: 2026-08-13\n")


def test_without_the_flag_the_written_bytes_do_not_change(tmp_path):
    """`dump_frontmatter` writes no key that is absent. An adopter who never
    passes the flag gets the file that 0.2.0 wrote."""
    root = repo(tmp_path)
    run_script("adr_new", "--title", "Order state machine", "--area", "orders",
               "--date", "2026-08-13", cwd=root)
    text = (root / "docs" / "adr" / "0001-order-state-machine.md").read_text("utf-8")
    assert "description:" not in text


def test_the_description_flag_is_refused_by_a_schema_without_the_property(tmp_path):
    """An adopter with an old vendored schema would get `schema-unknown-key`
    on every new ADR. Stop before the write."""
    root = repo(tmp_path, config={"version": 1, "dir": "docs/adr",
                                  "register": "docs/adr/README.md",
                                  "id_scheme": "sequential",
                                  "schema": "old.json"})
    write(root / "old.json", json.dumps({
        "type": "object",
        "required": ["type", "status", "date"],
        "properties": {"type": {"const": "adr"}, "status": {"type": "string"},
                       "date": {"type": "string"}, "title": {"type": "string"},
                       "area": {"type": "string"}},
        "additionalProperties": False,
    }))
    result = run_script("adr_new", "--title", "T", "--area", "core",
                        "--description", "one line", cwd=root)
    assert result.returncode == 2
    assert "the active schema does not permit description" in result.stderr
    assert list((root / "docs" / "adr").glob("*.md")) == []
