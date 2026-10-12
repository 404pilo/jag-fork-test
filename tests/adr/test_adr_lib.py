import json

import pytest

import adr_vendored_conftest as _cf

load_script = _cf.load_script
write = _cf.write

adr_lib = load_script("adr_lib")


# --- Task 1: the frontmatter fence ---------------------------------------

def test_a_line_one_fence_is_frontmatter():
    text = "---\nstatus: accepted\n---\n\n# Title\n"
    front, body = adr_lib.split_frontmatter(text)
    assert front == "status: accepted"
    assert body.lstrip().startswith("# Title")


def test_a_horizontal_rule_below_a_bold_key_block_is_not_frontmatter():
    text = "# ADR-002: Title\n\n**Status:** Accepted\n\n---\n\n## Context\n"
    front, body = adr_lib.split_frontmatter(text)
    assert front is None
    assert body == text


def test_an_unclosed_fence_is_not_frontmatter():
    text = "---\nstatus: accepted\n\n# Title\n"
    front, _ = adr_lib.split_frontmatter(text)
    assert front is None


# --- Task 2: strict stdlib frontmatter reader ------------------------------

FLAT = (
    'type: adr\n'
    'status: accepted\n'
    'date: "2026-04-20"\n'
    'title: Order state machine: part two\n'
    'area: orders\n'
    'supersedes: null\n'
    'related: []\n'
    'superseded_by:\n'
    '  - 0088-retry-backoff.md\n'
)


def test_the_minimal_reader_handles_the_flat_subset():
    data = adr_lib.parse_simple_yaml(FLAT)
    assert data["status"] == "accepted"
    assert data["date"] == "2026-04-20"          # matched quotes stripped
    assert data["title"] == "Order state machine: part two"
    assert data["supersedes"] is None
    assert data["related"] == []
    assert data["superseded_by"] == ["0088-retry-backoff.md"]


def test_keys_are_lower_cased():
    assert adr_lib.load_frontmatter("Status: Accepted\n")["status"] == "Accepted"


def test_the_stdlib_reader_handles_model_and_inline_comments():
    data = adr_lib.load_frontmatter(
        "title: T\n"
        "superseded_by:\n"
        "  - old.md # partial — D1 only\n"
        "model:\n"
        "  status: modeled\n"
        "  element: system.gateway\n"
    )
    assert data["superseded_by"] == ["old.md"]
    assert data["model"] == {"status": "modeled", "element": "system.gateway"}


def test_unsupported_yaml_is_rejected_instead_of_partially_recovered():
    with pytest.raises(adr_lib.FrontmatterError, match="unsupported YAML"):
        adr_lib.load_frontmatter("evidence: |\n  multiline\n")


# --- Task 3: legacy bold-key reader ---------------------------------------

LEGACY = (
    "# ADR-002: Gateway auth\n"
    "\n"
    "**Status:** Accepted\n"
    "**Date:** 2026-02-26\n"
    "\n"
    "**Superseded By:** ADR-004-new.md\n"
    "**Authors:** Someone\n"
    "\n"
    "---\n"
    "\n"
    "## Context\n"
)


def test_the_closed_bold_key_set_parses():
    data = adr_lib.parse_legacy(LEGACY)
    assert data["status"] == "accepted"
    assert data["date"] == "2026-02-26"
    assert data["superseded_by"] == ["ADR-004-new.md"]


def test_a_key_outside_the_set_is_ignored_not_warned():
    assert "authors" not in adr_lib.parse_legacy(LEGACY)


def test_a_status_outside_the_enum_is_kept_verbatim():
    data = adr_lib.parse_legacy("**Status:** Superseded by ADR-002\n")
    assert data["status"] == "superseded by adr-002"


def test_only_the_first_twenty_raw_lines_are_read():
    text = "\n" * 20 + "**Status:** Accepted\n"
    assert adr_lib.parse_legacy(text) == {}


# --- Task 4: title fallback and filename identity --------------------------

@pytest.mark.parametrize("heading,expected", [
    ("# ADR-005 — Title", "Title"),
    ("# adr-001: Title", "Title"),
    ("# ADR-0017 Title", "Title"),
    ("# One orchestration substrate", "One orchestration substrate"),
])
def test_the_heading_marker_is_stripped(heading, expected):
    assert adr_lib.heading_title(heading + "\n\nbody\n") == expected


def test_a_file_with_no_heading_has_no_title_fallback():
    assert adr_lib.heading_title("just body text\n") is None


@pytest.mark.parametrize("name,scheme,expected", [
    ("0007-order-state-machine.md", "sequential", "0007"),
    ("order-state-machine.md", "sequential", None),
    ("2026-04-20-order.md", "date", "2026-04-20"),
    ("order.md", "date", None),
])
def test_the_prefix_comes_from_the_filename(name, scheme, expected):
    assert adr_lib.prefix_of(name, scheme) == expected


# --- Task 5: parse_adr returns its source and never an id -------------------

def test_a_frontmatter_file_reports_its_source(tmp_path):
    path = write(tmp_path / "0007-order.md",
                 "---\ntype: adr\nstatus: accepted\ndate: 2026-04-20\n"
                 "title: Order state machine\n---\n\n# ADR-0007: Order state machine\n")
    adr = adr_lib.parse_adr(path)
    assert adr.source == "frontmatter"
    assert adr.filename == "0007-order.md"
    assert adr.prefix == "0007"
    assert adr.status == "accepted"
    assert adr.title == "Order state machine"
    assert "id" not in adr.meta


def test_a_bold_key_file_parses_as_legacy(tmp_path):
    path = write(tmp_path / "ADR-002-auth.md", LEGACY)
    adr = adr_lib.parse_adr(path)
    assert adr.source == "legacy"
    assert adr.title == "Gateway auth"
    assert adr.status == "accepted"


def test_refs_accept_a_scalar_or_a_list():
    assert adr_lib.refs({"related": "a.md"}) == ["a.md"]
    assert adr_lib.refs({"related": ["a.md", "b.md"]}) == ["a.md", "b.md"]
    assert adr_lib.refs({}) == []


# --- Task 6: config loading and Discovery -----------------------------------

def _repo(tmp_path, config=None, files=()):
    (tmp_path / ".git").mkdir()
    (tmp_path / "docs" / "adr").mkdir(parents=True)
    if config is not None:
        write(tmp_path / ".claude" / "adr.json", json.dumps(config))
    for name in files:
        write(tmp_path / "docs" / "adr" / name, "# stub\n")
    return tmp_path


def test_defaults_apply_when_there_is_no_config(tmp_path):
    corpus = adr_lib.load_corpus(_repo(tmp_path))
    assert corpus.config["dir"] == "docs/adr"
    assert corpus.config["register"] == {
        "path": "docs/adr/README.md", "exclude": []}
    assert corpus.config["id_scheme"] == "sequential"
    assert corpus.config["gate"] == {
        "deterministic": False, "model": False, "proposed_verdict": "error"}
    assert corpus.config["archive_dir"] == "archive"


def test_a_partial_gate_block_merges_over_the_defaults(tmp_path):
    root = _repo(tmp_path, {"version": 1, "gate": {"model": True}})
    gate = adr_lib.load_corpus(root).config["gate"]
    assert gate == {"deterministic": False, "model": True, "proposed_verdict": "error"}


def test_discovery_is_the_top_level_glob_minus_the_register(tmp_path):
    root = _repo(tmp_path, None, ["0001-a.md", "README.md", "CLAUDE.md", "stray-note.md"])
    write(root / "docs" / "adr" / "archive" / "0002-b.md", "# archived\n")
    write(root / "docs" / "adr" / "sub" / "0003-c.md", "# nested\n")
    found = [p.name for p in adr_lib.discover(adr_lib.load_corpus(root))]
    assert found == ["0001-a.md", "stray-note.md"]


def test_a_renamed_register_is_still_excluded(tmp_path):
    root = _repo(tmp_path, {"version": 1, "register": "docs/adr/index.md"},
                 ["0001-a.md", "index.md"])
    found = [p.name for p in adr_lib.discover(adr_lib.load_corpus(root))]
    assert found == ["0001-a.md"]


def test_legacy_register_string_normalizes_to_the_object_contract(tmp_path):
    root = _repo(tmp_path, {"register": "docs/adr/README.md"})
    corpus = adr_lib.load_corpus(root)
    assert corpus.config["register"] == {
        "path": "docs/adr/README.md", "exclude": []
    }


def test_excluded_records_leave_the_register_but_stay_in_the_graph(tmp_path):
    root = _repo(tmp_path, {
        "dir": "docs/adr",
        "register": {"path": "docs/adr/README.md", "exclude": ["0002-diagram.md"]},
        "archive_dir": "archive",
    }, ["0001-live.md", "0002-diagram.md", "README.md", "CLAUDE.md"])
    write(root / "docs" / "adr" / "archive" / "0003-old.md", "# old\n")
    corpus = adr_lib.load_corpus(root)
    assert [path.name for path in adr_lib.discover(corpus)] == ["0001-live.md"]
    assert set(adr_lib.relationship_corpus(corpus)) == {
        "0001-live.md", "0002-diagram.md", "0003-old.md"
    }


def test_a_live_archive_basename_collision_is_refused(tmp_path):
    root = _repo(tmp_path, {"archive_dir": "archive"}, ["0001-same.md"])
    write(root / "docs" / "adr" / "archive" / "0001-same.md", "# old\n")
    with pytest.raises(adr_lib.CorpusError, match="filename collision"):
        adr_lib.relationship_corpus(adr_lib.load_corpus(root))


# --- Task 5: raw partial parsing and textual edits --------------------------

def test_partial_targets_are_read_from_each_raw_list(tmp_path):
    front = (
        "supersedes:\n  - old.md # partial — D1 only\n"
        "superseded_by:\n  - new.md # PARTIAL\n  - full.md\n"
    )
    assert adr_lib.partial_targets(front, "supersedes") == {"old.md"}
    assert adr_lib.partial_targets(front, "superseded_by") == {"new.md"}


def test_textual_edits_preserve_unknown_keys_comments_order_and_crlf():
    before = (
        "---\r\ntype: adr\r\ntitle: T\r\ncustom: keep # local\r\n"
        "status: accepted\r\nsuperseded_by:\r\n"
        "  - old.md # partial — keep this\r\n---\r\n\r\n# T\r\n"
    )
    changed = adr_lib.set_scalar(before, "status", "deprecated")
    changed = adr_lib.rename_list_item(
        changed, "superseded_by", "old.md", "archive/old.md")
    assert "custom: keep # local\r\n" in changed
    assert "status: deprecated\r\n" in changed
    assert "archive/old.md # partial — keep this\r\n" in changed
    assert changed.endswith("# T\r\n")


def test_append_list_item_converts_null_without_reordering_keys():
    before = "---\ntitle: T\nsupersedes: null\nrelated: []\n---\n"
    after = adr_lib.append_list_item(
        before, "supersedes", "old.md", "partial — D2 only")
    assert after.index("title:") < after.index("supersedes:") < after.index("related:")
    assert "supersedes:\n  - old.md # partial — D2 only\n" in after


# --- Task 7: the frontmatter writer -----------------------------------------

def test_the_writer_emits_the_fixed_key_order_and_no_id():
    text = adr_lib.dump_frontmatter({
        "id": 7, "title": "Order state machine", "status": "proposed",
        "date": "2026-08-13", "type": "adr", "area": "orders",
        "related": ["0003-a.md"],
    })
    keys = [line.split(":")[0] for line in text.splitlines() if not line.startswith(" ")]
    assert keys == ["type", "title", "area", "status", "date", "related"]
    assert "id" not in keys


def test_a_title_holding_a_colon_round_trips_through_the_stdlib_reader():
    title = "Order state machine: part two"
    text = adr_lib.dump_frontmatter(
        {"type": "adr", "status": "proposed", "date": "2026-08-13", "title": title})
    assert adr_lib.load_frontmatter(text)["title"] == title


def test_unsupplied_keys_are_omitted_and_supplied_null_is_written():
    text = adr_lib.dump_frontmatter(
        {"type": "adr", "status": "proposed", "date": "2026-08-13", "title": "T",
         "supersedes": None})
    assert "area:" not in text
    assert "supersedes: null" in text
    assert "related:" not in text


def test_the_writer_keeps_an_unknown_scalar_and_an_unknown_list():
    """Rule 7 of the 0.3.0 design. `dump_frontmatter` dropped each key
    outside `KEY_ORDER`, so a round trip lost the own keys of an
    adopter."""
    text = adr_lib.dump_frontmatter({
        "type": "adr", "title": "T", "area": "core", "status": "accepted",
        "date": "2026-08-13", "owner": "platform",
        "reviewers": ["ana", "bo"], "empty": [],
    })
    keys = [line.split(":")[0] for line in text.splitlines()
            if not line.startswith(" ")]
    assert keys == ["type", "title", "area", "status", "date",
                    "owner", "reviewers", "empty"]
    assert "reviewers:\n  - ana\n  - bo\n" in text
    assert "empty: []" in text
    back = adr_lib.load_frontmatter(text)
    assert back["owner"] == "platform"
    assert back["reviewers"] == ["ana", "bo"]


def test_an_unknown_key_named_id_is_still_never_written():
    """Invariant 2 stands: the file name is the identity. `id` is the one
    unknown key that the writer drops. [inferred]"""
    text = adr_lib.dump_frontmatter({
        "type": "adr", "title": "T", "status": "proposed",
        "date": "2026-08-13", "id": 7,
    })
    assert not any(line.startswith("id:") for line in text.splitlines())


# --- Task 8: adr.schema.json, bound to the code -----------------------------

def test_the_schema_matches_the_parser_constants(schemas_dir):
    schema = json.loads(
        (schemas_dir / "adr.schema.json").read_text(encoding="utf-8"))
    assert sorted(schema["required"]) == sorted(adr_lib.REQUIRED_KEYS)
    assert schema["properties"]["status"]["enum"] == list(adr_lib.STATUSES)
    assert "id" not in schema["properties"]
    assert schema["additionalProperties"] is False


def test_the_schema_matches_the_reference_property_surface(schemas_dir):
    schema = json.loads((schemas_dir / "adr.schema.json").read_text("utf-8"))
    assert set(schema["properties"]) == {
        "type", "status", "date", "title", "description", "area", "amended",
        "supersedes", "superseded_by", "amends", "amended_by", "refines",
        "deviates_from", "related", "parent_spec", "evidence", "model",
    }
    assert schema["properties"]["area"]["pattern"] == "^[a-z][a-z0-9-]*$"
    assert schema["properties"]["model"]["additionalProperties"] is False


# --- Phase 2 Task 1: a generated index is not an ADR ------------------------

def test_a_generated_index_is_detected_by_its_okf_version_key(tmp_path):
    generated = _cf.write(tmp_path / "index.md", '---\nokf_version: "0.2"\n---\n\n# Index\n')
    owner = _cf.write(tmp_path / "other" / "index.md",
                      "---\ntype: adr\ntitle: Index\n---\n\n# Index\n")
    plain = _cf.write(tmp_path / "0001-a.md", '---\nokf_version: "0.2"\n---\n')
    assert adr_lib.is_generated_index(generated) is True
    assert adr_lib.is_generated_index(owner) is False       # an ADR named index.md (D16)
    assert adr_lib.is_generated_index(plain) is False       # the base name decides first
    assert adr_lib.is_generated_index(tmp_path / "gone.md") is False


def test_the_three_discovery_functions_skip_a_generated_index(tmp_path):
    root = _repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                   "archive_dir": "archive"})
    _cf.write(root / "docs" / "adr" / "0001-a.md",
              "---\ntype: adr\ntitle: A\narea: core\nstatus: accepted\ndate: 2026-01-01\n---\n")
    _cf.write(root / "docs" / "adr" / "index.md", '---\nokf_version: "0.2"\n---\n\n# Index\n')
    _cf.write(root / "docs" / "adr" / "archive" / "0002-b.md",
              "---\ntype: adr\ntitle: B\narea: core\nstatus: deprecated\ndate: 2026-01-02\n---\n")
    _cf.write(root / "docs" / "adr" / "archive" / "index.md",
              '---\nokf_version: "0.2"\n---\n\n# Archived decisions\n')
    corpus = adr_lib.load_corpus(root)
    assert [path.name for path in adr_lib.discover(corpus)] == ["0001-a.md"]
    assert [path.name for path in adr_lib.discover_archived(corpus)] == ["0002-b.md"]
    # Two index.md files would otherwise raise `ADR filename collision: index.md`.
    assert sorted(adr_lib.relationship_corpus(corpus)) == ["0001-a.md", "0002-b.md"]


# --- phase 4: the draft value of adr_migrate.py --descriptions --------------

def test_description_draft_removes_the_cut_mark_and_the_derived_mark():
    """The index text of section 11.1 ends with ` (derived)`, and a cut text
    ends with `...`. A stored `description` is a plain sentence, so the draft
    carries neither mark."""
    body = "\n## Decision\n\n" + " ".join(f"word{n}" for n in range(1, 41)) + ".\n"
    draft = adr_lib.description_draft(body)
    assert draft.startswith("word1 word2 word3")
    assert len(draft.split()) == 25
    assert not draft.endswith("...")
    assert not draft.endswith("(derived)")


def test_description_draft_is_empty_when_the_body_gives_no_text():
    assert adr_lib.description_draft("\n## Decision\n\n## Consequences\n") == ""


def test_set_scalar_writes_an_inline_comment_on_a_key_that_it_inserts():
    text = "---\ntype: adr\ntitle: T\nstatus: accepted\ndate: 2026-03-03\n---\n\n# T\n"
    out = adr_lib.set_scalar(text, "description", "Short text.", comment="draft")
    assert "description: Short text. # draft\n" in out
    assert out.index("description:") > out.index("title:")


def test_set_scalar_keeps_the_authored_comment_on_a_key_that_exists():
    """The comment argument covers an inserted line only. A line that a
    person wrote keeps the comment that the person wrote."""
    text = ("---\ntype: adr\ntitle: T\ndescription: Old. # mine\n"
            "status: accepted\ndate: 2026-03-03\n---\n\n# T\n")
    out = adr_lib.set_scalar(text, "description", "New.", comment="draft")
    assert "description: New. # mine\n" in out
    assert "# draft" not in out
