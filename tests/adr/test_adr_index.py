import json

import pytest

import adr_vendored_conftest as _cf

load_script = _cf.load_script
run_script = _cf.run_script
write = _cf.write

adr_lib = load_script("adr_lib")
adr_index = load_script("adr_index")


def adr(root, name, **meta):
    body = {
        "type": "adr",
        "status": "accepted",
        "date": "2026-08-13",
        "title": name.split("-", 1)[1].replace(".md", "").replace("-", " "),
    }
    body.update(meta)
    return write(
        root / "docs" / "adr" / name,
        "---\n" + adr_lib.dump_frontmatter(body) + "---\n\n# body\n",
    )


def repo(tmp_path, config=None):
    (tmp_path / ".git").mkdir()
    (tmp_path / "docs" / "adr").mkdir(parents=True)
    write(
        tmp_path / ".claude" / "adr.json",
        json.dumps(
            config
            or {
                "dir": "docs/adr",
                "register": "docs/adr/README.md",
                "id_scheme": "sequential",
            }
        ),
    )
    return tmp_path


def test_region_preserves_headers_order_and_summary_then_appends_new_areas(tmp_path):
    root = repo(tmp_path)
    adr(root, "0002-z.md", area="zeta", title="Z", date="2026-01-02")
    adr(root, "0001-a.md", area="alpha", title="A", date="2026-01-01")
    existing = (
        "### zeta — Kept description\n"
        "- [0002 — Z](./0002-z.md) — Hand summary — exact\n"
    )
    parsed = adr_index.parse_register(existing)
    region = adr_index.build_region(
        adr_lib.load_corpus(root), parsed, emit_status_cells=False
    )
    assert region.index("### zeta — Kept description") < region.index("### alpha —")
    assert "Hand summary — exact" in region


def test_live_register_never_emits_an_archive_or_unfiled_section(tmp_path):
    root = repo(tmp_path, config={"archive_dir": "archive"})
    adr(root, "0001-live.md", area="core")
    write(root / "docs" / "adr" / "archive" / "0002-old.md", "# old\n")
    region = adr_index.build_region(adr_lib.load_corpus(root), adr_index.parse_register(""))
    assert "### archived" not in region
    assert "### unfiled" not in region


def test_inject_normalizes_each_marker_seam_to_one_blank_line():
    wrapped = f"{adr_index.BEGIN}\nbody\n\n{adr_index.END}"
    assert adr_index.inject("before\n\n\n", wrapped, "\n\nafter") == (
        f"before\n\n{wrapped}\n\nafter"
    )


def test_parse_register_uses_markerless_index_section_as_a_fallback():
    sections = adr_index.parse_register(
        "# Architecture decisions\n\n## Index\n\n"
        "### core — Core decisions\n"
        "- [0001 — Foundation](./0001-foundation.md) — Kept summary\n"
    )
    assert len(sections) == 1
    assert sections[0].area == "core"
    assert sections[0].bullets[0].summary == "Kept summary"


def test_cell_less_tail_with_em_dashes_remains_the_summary():
    bullet = adr_index.parse_bullet(
        "- [2026-01-01 — T](./2026-01-01-t.md) — summary — with — dashes"
    )
    assert bullet.status_cell is None
    assert bullet.summary == "summary — with — dashes"


def test_empty_and_mixed_regions_choose_cell_mode():
    assert adr_index.detect_status_cells([]) is True
    mixed = adr_index.parse_register(
        "### core — C\n"
        "- [2026-01-01 — A](./a.md) — accepted — one\n"
        "- [2026-01-02 — B](./b.md) — two — details\n"
    )
    assert adr_index.detect_status_cells(mixed) is True
    assert mixed[0].bullets[1].summary == "two — details"


def test_superseded_cell_links_files_but_partial_keeps_frontmatter_status(tmp_path):
    root = repo(tmp_path, config={"id_scheme": "date", "archive_dir": "archive"})
    adr(
        root,
        "2026-01-01-partial.md",
        area="core",
        status="accepted",
        superseded_by=["2026-01-02-new.md"],
    )
    adr(root, "2026-01-02-new.md", area="core")
    adr(
        root,
        "2026-01-03-old.md",
        area="core",
        status="superseded",
        superseded_by=["archive/2025-12-31-survivor.md"],
    )
    write(
        root / "docs" / "adr" / "archive" / "2025-12-31-survivor.md",
        "---\ntype: adr\ntitle: Survivor\narea: core\nstatus: accepted\ndate: 2025-12-31\n---\n",
    )
    region = adr_index.build_region(adr_lib.load_corpus(root), [])
    assert "partial.md) — accepted —" in region
    assert (
        "old.md) — superseded by [2025-12-31-survivor.md]"
        "(./archive/2025-12-31-survivor.md) —"
    ) in region


def test_default_check_is_green_after_write_and_does_not_mutate(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-foundation.md", area="core", title="Foundation")
    before = "# Architecture decisions\n"
    write(root / "docs" / "adr" / "README.md", before)
    stale = run_script("adr_index", cwd=root)
    assert stale.returncode == 1
    assert (root / "docs" / "adr" / "README.md").read_text(encoding="utf-8") == before
    assert run_script("adr_index", "--write", cwd=root).returncode == 0
    fresh = run_script("adr_index", cwd=root)
    assert fresh.returncode == 0
    assert fresh.stdout == "register up to date\n"


def test_write_refuses_a_noncanonical_live_adr(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-foundation.md", area=None, title="Foundation")
    result = run_script("adr_index", "--write", cwd=root)
    assert result.returncode == 1
    assert "cannot produce a canonical bullet" in result.stderr


def test_content_outside_the_markers_is_preserved(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-foundation.md", area="core", title="Foundation")
    original = "# Architecture decisions\n\nHand-authored intro.\n\n"
    original += f"{adr_index.BEGIN}\n\n{adr_index.END}\n\nHand-authored footer.\n"
    write(root / "docs" / "adr" / "README.md", original)
    assert run_script("adr_index", "--write", cwd=root).returncode == 0
    text = (root / "docs" / "adr" / "README.md").read_text(encoding="utf-8")
    assert text.startswith("# Architecture decisions\n\nHand-authored intro.")
    assert text.rstrip().endswith("Hand-authored footer.")
    assert "0001-foundation.md" in text


def test_build_block_preserves_existing_section_and_summary(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-foundation.md", area="core", title="Foundation")
    register = (
        f"{adr_index.BEGIN}\n\n"
        "> notice\n\n"
        "### core — Existing description\n"
        "- [0001 — Foundation](./0001-foundation.md) — Kept summary\n\n"
        f"{adr_index.END}\n"
    )
    sections = adr_index.parse_register(register)
    block = adr_index.build_block(adr_lib.load_corpus(root), sections)
    assert "### core — Existing description" in block
    assert "Kept summary" in block


# --- Phase 0: DEF-2, a status-less row does not grow ------------------------

def test_the_row_parser_reads_unknown_as_a_status_cell():
    """`status_cell()` writes the word `unknown` for a record with no
    status. The parser did not read it back, so the word became part of
    the summary and the next write added one more cell (DEF-2)."""
    bullet = adr_index.parse_bullet(
        "- [0001 — A](./0001-a.md) — unknown — hand summary")
    assert bullet.status_cell == adr_index.UNKNOWN_STATUS_CELL
    assert bullet.summary == "hand summary"


def test_three_writes_of_a_status_less_row_give_equal_bytes(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-a.md", area="core", title="A", status=None)
    adr(root, "0002-b.md", area="core", title="B")
    register = root / "docs" / "adr" / "README.md"
    # The register must already track the status-less row: `_noncanonical`
    # refuses a fresh frontmatter record that left a canonical field empty.
    write(register, (
        f"{adr_index.BEGIN}\n\n{adr_index.WARNING_BLOCKQUOTE}\n\n"
        "### core — Core\n"
        "- [0001 — A](./0001-a.md) — first\n"
        "- [0002 — B](./0002-b.md) — accepted — second\n\n"
        f"{adr_index.END}\n"
    ))
    assert run_script("adr_index", "--write", cwd=root).returncode == 0
    first = register.read_text(encoding="utf-8")
    assert "- [0001 — A](./0001-a.md) — unknown — first" in first
    assert first.count(" — unknown — ") == 1
    for _ in range(2):
        assert run_script("adr_index", "--write", cwd=root).returncode == 0
        assert register.read_text(encoding="utf-8") == first
    check = run_script("adr_index", cwd=root)
    assert check.returncode == 0
    assert check.stdout == "register up to date\n"


# --- Phase 0: DEF-3, name each row that the next write removes -------------

REMOVED_REGISTER = (
    f"{adr_index.BEGIN}\n\n{adr_index.WARNING_BLOCKQUOTE}\n\n"
    "### core — Core\n"
    "- [0001 — A](./0001-a.md) — accepted — kept\n"
    "- [0099 — Gone](./0099-gone.md) — accepted — lost with the file\n"
    "- 0098-broken.md — no link at all\n"
    "- [0097 — Star](./0097-star.md) no separator\n\n"
    f"{adr_index.END}\n"
)


def test_unparsed_rows_names_each_bullet_the_parser_cannot_read():
    assert adr_index.unparsed_rows(REMOVED_REGISTER) == [
        "- 0098-broken.md — no link at all",
        "- [0097 — Star](./0097-star.md) no separator",
    ]


def test_dropped_rows_names_each_bullet_whose_adr_is_gone(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-a.md", area="core", title="A")
    corpus = adr_lib.load_corpus(root)
    records = [adr_lib.parse_adr(path, corpus.id_scheme, root=corpus.dir)
               for path in adr_lib.discover(corpus)]
    dropped = adr_index.dropped_rows(
        adr_index.parse_register(REMOVED_REGISTER), records)
    assert [bullet.filename for bullet in dropped] == ["0099-gone.md"]
    assert dropped[0].summary == "lost with the file"


def test_write_reports_each_removed_row_before_the_receipt(tmp_path):
    """The `indexing-adrs` skill tells the agent to read a `dropped row:`
    line and an `unparsed row:` line. No script wrote them (DEF-3)."""
    root = repo(tmp_path)
    adr(root, "0001-a.md", area="core", title="A")
    write(root / "docs" / "adr" / "README.md", REMOVED_REGISTER)
    result = run_script("adr_index", "--write", cwd=root)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout.splitlines() == [
        "dropped row: 0099-gone.md — lost with the file",
        "unparsed row: - 0098-broken.md — no link at all",
        "unparsed row: - [0097 — Star](./0097-star.md) no separator",
        "rows removed: 1 dropped, 2 unparsed",
        "wrote docs/adr/README.md (1 ADRs indexed)",
    ]


def test_write_prints_the_receipt_only_when_no_row_is_removed(tmp_path):
    root = repo(tmp_path)
    adr(root, "0001-a.md", area="core", title="A")
    assert run_script("adr_index", "--write", cwd=root).returncode == 0
    result = run_script("adr_index", "--write", cwd=root)
    assert result.stdout == "wrote docs/adr/README.md (1 ADRs indexed)\n"


def test_the_escape_rule_folds_breaks_and_escapes_brackets():
    assert adr_index.escape_index_text("a\tb\r\nc") == "a b  c"
    assert adr_index.escape_index_text("Use [x] now") == r"Use \[x\] now"
    assert adr_index.escape_index_text("  trimmed\n") == "trimmed"


def test_the_href_rule_encodes_a_space_and_each_round_bracket():
    assert adr_index.escape_href("./a b (1).md") == "./a%20b%20%281%29.md"


def test_the_corpus_title_is_the_register_h1_or_the_fixed_text(tmp_path):
    root = repo(tmp_path)
    corpus = adr_lib.load_corpus(root)
    assert adr_index.corpus_title(corpus) == "Architecture decision records"
    write(root / "docs" / "adr" / "README.md", "# Platform [decisions]\n\nintro\n")
    assert adr_index.corpus_title(corpus) == r"Platform \[decisions\]"


def test_the_index_label_follows_the_bullet_line_selection_rule(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-03-01-one-writer.md", area="orders", title="One writer per aggregate")
    corpus = adr_lib.load_corpus(root)
    record = adr_lib.parse_adr(root / "docs" / "adr" / "2026-03-01-one-writer.md",
                               corpus.id_scheme, root=corpus.dir)
    assert adr_index.index_label(record) == "2026-03-01: One writer per aggregate"
    record.prefix = None
    assert adr_index.index_label(record) == "One writer per aggregate"
    record.title = None
    assert adr_index.index_label(record) == "2026-03-01-one-writer.md"


def test_derived_text_takes_the_first_sentence_below_the_decision_heading():
    body = ("# ADR-1: T\n\n## Context\n\nTwo services wrote rows.\n\n"
            "## Decision\n\nOnly the orders service writes order rows. Each other service "
            "sends a command.\n\n## Why\n\nOne writer gives one rule set.\n")
    assert adr_index.derived_text(body) == (
        "Only the orders service writes order rows. (derived)")


def test_derived_text_strips_a_list_mark_and_the_marks_around_words():
    body = ("# T\n\n## Decision\n\n- **Only** the `orders` service writes "
            "[order rows](./x.md) to the store.\n- A second item.\n")
    assert adr_index.derived_text(body) == (
        "Only the orders service writes order rows to the store. (derived)")


def test_derived_text_cuts_at_twenty_five_words():
    words = " ".join(f"word{index}" for index in range(1, 41))
    body = f"# T\n\n## Decision\n\n{words}\n"
    result = adr_index.derived_text(body)
    assert result.endswith("... (derived)")
    # Step 5 of section 11.1 cuts the text to 25 words and then adds `...`, so
    # the cut text holds 25 words and the last one carries the 3 dots.
    assert result.split()[:25] == (
        [f"word{index}" for index in range(1, 25)] + ["word25..."])


def test_derived_text_falls_back_to_the_block_below_the_h1_line():
    body = "# ADR-9: No heading here\n\nThe cache keeps one entry for each route.\n\n## Why\n\nx\n"
    assert adr_index.derived_text(body) == (
        "The cache keeps one entry for each route. (derived)")


def test_derived_text_is_empty_when_no_block_is_found():
    assert adr_index.derived_text("# T\n\n## Decision\n\n## Why\n\nx\n") == ""
    assert adr_index.derived_text("no heading at all\n") == ""


def nav_repo(tmp_path):
    """A corpus with one live ADR, one archived ADR and one plain document."""
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date", "archive_dir": "archive"})
    adr(root, "2026-03-01-one-writer.md", area="orders", title="One writer")
    write(root / "docs" / "adr" / "archive" / "2026-01-01-use-postgres.md",
          "---\ntype: adr\ntitle: Postgres\narea: storage\nstatus: superseded\n"
          "date: 2026-01-01\nsuperseded_by: 2026-03-01-one-writer.md\n---\n\n# body\n")
    write(root / "docs" / "architecture" / "data-ownership.md", "# Data ownership\n")
    # `relationship_corpus()` keeps each `*.md` of the top level of `<dir>`, so
    # a plain note there is a record. Section 8.4 names `CLAUDE.md` as a file
    # below `<dir>` that stays out of the record set.
    write(root / "docs" / "adr" / "CLAUDE.md", "# Notes\n")
    return root


def resolved(root, value, author="2026-03-01-one-writer.md"):
    corpus = adr_lib.load_corpus(root)
    records = adr_lib.relationship_corpus(corpus)
    return adr_index.resolve_target(corpus, records, author, value)


def test_resolve_target_reports_a_url_before_any_path_step(tmp_path):
    root = nav_repo(tmp_path)
    assert resolved(root, "https://example.org/cqrs") == ("url", "https://example.org/cqrs", "")


def test_resolve_target_drops_a_leading_slash_and_keeps_the_anchor(tmp_path):
    root = nav_repo(tmp_path)
    state, target, anchor = resolved(root, "/docs/architecture/data-ownership.md#writers")
    assert (state, target, anchor) == ("doc", "docs/architecture/data-ownership.md", "writers")


def test_resolve_target_reads_a_dir_prefixed_value_as_an_adr(tmp_path):
    root = nav_repo(tmp_path)
    assert resolved(root, "docs/adr/archive/2026-01-01-use-postgres.md") == (
        "archive", "archive/2026-01-01-use-postgres.md", "")


def test_resolve_target_finds_a_live_and_an_archived_record(tmp_path):
    root = nav_repo(tmp_path)
    assert resolved(root, "archive/2026-01-01-use-postgres.md")[0] == "archive"
    assert resolved(root, "2026-03-01-one-writer.md",
                    author="archive/2026-01-01-use-postgres.md") == (
        "live", "2026-03-01-one-writer.md", "")


def test_resolve_target_finds_a_document_relative_to_the_author_directory(tmp_path):
    root = nav_repo(tmp_path)
    state, target, _ = resolved(root, "../architecture/data-ownership.md")
    assert (state, target) == ("doc", "docs/architecture/data-ownership.md")


def test_a_markdown_file_below_dir_that_is_no_adr_is_a_document(tmp_path):
    root = nav_repo(tmp_path)
    assert resolved(root, "CLAUDE.md") == ("doc", "docs/adr/CLAUDE.md", "")


def test_resolve_target_reports_missing_in_its_normalised_form(tmp_path):
    root = nav_repo(tmp_path)
    assert resolved(root, "/docs/gone.md#top") == ("missing", "docs/gone.md", "top")


def test_the_tsv_value_rule_folds_breaks_and_writes_the_empty_value(tmp_path):
    assert adr_index.tsv_field("a\tb\nc ") == "a b c"
    assert adr_index.tsv_field("") == "-"


def chain_repo(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date", "archive_dir": "archive"})
    write(root / "docs" / "adr" / "archive" / "2026-01-01-a.md",
          "---\ntype: adr\ntitle: A\narea: storage\nstatus: superseded\ndate: 2026-01-01\n"
          "superseded_by:\n  - 2026-02-01-b.md\n  - 2026-03-01-c.md # partial\n---\n\n# body\n")
    adr(root, "2026-02-01-b.md", area="storage", title="B", status="superseded",
        superseded_by=["2026-04-01-d.md"])
    adr(root, "2026-03-01-c.md", area="orders", title="C")
    adr(root, "2026-04-01-d.md", area="storage", title="D")
    return root


def context_of(root):
    return adr_index.nav_context(adr_lib.load_corpus(root))


def test_the_context_splits_the_live_and_the_archived_record_sets(tmp_path):
    context = context_of(chain_repo(tmp_path))
    assert [record.relative_path for record in context.live] == [
        "2026-02-01-b.md", "2026-03-01-c.md", "2026-04-01-d.md"]
    assert [record.relative_path for record in context.archived] == ["archive/2026-01-01-a.md"]


def test_the_successor_map_holds_only_resolved_edges(tmp_path):
    context = context_of(chain_repo(tmp_path))
    assert context.graph["archive/2026-01-01-a.md"] == ["2026-02-01-b.md", "2026-03-01-c.md"]
    assert context.graph["2026-03-01-c.md"] == []


def test_each_branch_gives_its_own_chain_end(tmp_path):
    context = context_of(chain_repo(tmp_path))
    ends = adr_index.chain_ends(context, "archive/2026-01-01-a.md")
    assert sorted(ends) == ["2026-03-01-c.md", "2026-04-01-d.md"]
    assert ends["2026-04-01-d.md"] == {"2026-02-01-b.md"}     # the first hop of that branch
    assert ends["2026-03-01-c.md"] == {"2026-03-01-c.md"}


def test_a_superseded_adr_with_no_resolved_successor_has_no_chain_end(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-01-01-a.md", area="core", title="A", status="superseded",
        superseded_by=["2026-09-09-gone.md"])
    assert adr_index.chain_ends(context_of(root), "2026-01-01-a.md") == {}


def test_a_superseded_by_loop_is_found_once_and_starts_at_the_lowest_name(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-02-01-b.md", area="core", title="B", status="superseded",
        superseded_by=["2026-01-01-a.md"])
    adr(root, "2026-01-01-a.md", area="core", title="A", status="superseded",
        superseded_by=["2026-02-01-b.md"])
    assert adr_index.find_cycles(context_of(root).graph) == [
        ["2026-01-01-a.md", "2026-02-01-b.md", "2026-01-01-a.md"]]


def test_a_corpus_with_no_loop_reports_no_cycle(tmp_path):
    assert adr_index.find_cycles(context_of(chain_repo(tmp_path)).graph) == []


def links_repo(tmp_path):
    """The corpus of the section 9.3 example rows."""
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date", "archive_dir": "archive"})
    write(root / "docs" / "adr" / "2026-03-01-one-writer.md",
          "---\ntype: adr\ntitle: One writer\narea: orders\nstatus: accepted\n"
          "date: 2026-03-01\n"
          "supersedes: archive/2026-01-01-use-postgres.md # partial\n"
          "deviates_from: https://example.org/cqrs\n"
          "related:\n  - /docs/architecture/data-ownership.md#writers\n---\n\n# body\n")
    write(root / "docs" / "adr" / "2026-02-01-use-sqlite.md",
          "---\ntype: adr\ntitle: SQLite\narea: storage\nstatus: accepted\n"
          "date: 2026-02-01\nsupersedes: archive/2026-01-01-use-postgres.md\n---\n\n# body\n")
    write(root / "docs" / "adr" / "archive" / "2026-01-01-use-postgres.md",
          "---\ntype: adr\ntitle: Postgres\narea: storage\nstatus: superseded\n"
          "date: 2026-01-01\nsuperseded_by:\n  - 2026-02-01-use-sqlite.md\n"
          "  - 2026-03-01-one-writer.md # partial\n---\n\n# body\n")
    write(root / "docs" / "architecture" / "data-ownership.md", "# Data ownership\n")
    return root


def rows_of(root):
    return adr_index.edge_rows(adr_index.nav_context(adr_lib.load_corpus(root)))


def test_each_edge_row_holds_the_eight_columns_of_the_spec_example(tmp_path):
    rows = {(row[0], row[1], row[2]): row for row in rows_of(links_repo(tmp_path))}
    supersedes = rows[("2026-03-01-one-writer.md", "supersedes",
                       "archive/2026-01-01-use-postgres.md")]
    assert supersedes == ["2026-03-01-one-writer.md", "supersedes",
                          "archive/2026-01-01-use-postgres.md", "archive", "superseded",
                          "orders", "storage", "partial;now=2026-02-01-use-sqlite.md"]
    assert rows[("2026-03-01-one-writer.md", "deviates_from", "https://example.org/cqrs")] == [
        "2026-03-01-one-writer.md", "deviates_from", "https://example.org/cqrs", "url",
        "-", "orders", "-", "-"]
    assert rows[("2026-03-01-one-writer.md", "related",
                 "docs/architecture/data-ownership.md")][7] == "anchor=writers"
    assert rows[("archive/2026-01-01-use-postgres.md", "superseded_by",
                 "2026-03-01-one-writer.md")][7] == "partial"


def test_a_now_note_names_each_chain_end_that_is_not_the_source(tmp_path):
    rows = {(row[0], row[1]): row for row in rows_of(links_repo(tmp_path))}
    assert rows[("2026-02-01-use-sqlite.md", "supersedes")][7] == (
        "now=2026-03-01-one-writer.md")


def test_the_rows_sort_by_source_then_walk_order_then_relation(tmp_path):
    rows = rows_of(links_repo(tmp_path))
    assert [(row[0], row[1]) for row in rows] == [
        ("2026-02-01-use-sqlite.md", "supersedes"),
        ("2026-03-01-one-writer.md", "supersedes"),
        ("2026-03-01-one-writer.md", "deviates_from"),
        ("2026-03-01-one-writer.md", "related"),
        ("archive/2026-01-01-use-postgres.md", "superseded_by"),
        ("archive/2026-01-01-use-postgres.md", "superseded_by"),
    ]


def test_a_value_that_is_stored_two_times_gives_one_row(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    write(root / "docs" / "adr" / "2026-01-01-a.md",
          "---\ntype: adr\ntitle: A\narea: core\nstatus: accepted\ndate: 2026-01-01\n"
          "related:\n  - 2026-02-01-b.md\n  - 2026-02-01-b.md\n---\n\n# body\n")
    adr(root, "2026-02-01-b.md", area="core", title="B")
    assert len([row for row in rows_of(root) if row[1] == "related"]) == 1


def test_the_links_file_opens_with_the_header_row_and_ends_with_one_newline(tmp_path):
    text = adr_index.render_links(rows_of(links_repo(tmp_path)))
    assert text.splitlines()[0] == (
        "source\trelation\ttarget\ttarget_state\ttarget_status\tsource_area\t"
        "target_area\tnote")
    assert text.endswith("\n") and not text.endswith("\n\n")
    assert "\r" not in text
    assert adr_index.render_links([]) == (
        "source\trelation\ttarget\ttarget_state\ttarget_status\tsource_area\t"
        "target_area\tnote\n")


def test_an_area_slug_that_is_a_reserved_word_stops_the_generator(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-01-01-a.md", area="related", title="A")
    context = adr_index.nav_context(adr_lib.load_corpus(root))
    with pytest.raises(adr_index.NavigationError) as error:
        adr_index.guard_area_slugs(context)
    assert str(error.value) == "area slug related is a reserved word of links.tsv"


@pytest.mark.parametrize("area", ["supersedes", "live", "accepted", "partial"])
def test_each_reserved_word_class_is_guarded(tmp_path, area):
    assert area in adr_index.RESERVED_AREA_WORDS


def test_an_area_slug_that_is_no_reserved_word_passes(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-01-01-a.md", area="orders", title="A")
    context = adr_index.nav_context(adr_lib.load_corpus(root))
    assert adr_index.guard_area_slugs(context) is None


def test_a_file_name_with_a_comma_or_a_semicolon_stops_the_generator(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-01-01-a,b.md", area="orders", title="A")
    context = adr_index.nav_context(adr_lib.load_corpus(root))
    with pytest.raises(adr_index.NavigationError) as error:
        adr_index.guard_file_names(context)
    assert "2026-01-01-a,b.md" in str(error.value)


def test_a_navigation_error_is_a_corpus_error(tmp_path):
    # `adr_index` imports `adr_lib` by its plain name, so the loader of this
    # suite holds two module objects for the one file. Read the class from the
    # module that `adr_index` imported, or the subclass test compares two
    # different objects of the same source file.
    assert issubclass(adr_index.NavigationError, adr_index.adr_lib.CorpusError)


def index_repo(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date", "archive_dir": "archive"})
    write(root / "docs" / "adr" / "2026-03-01-one-writer.md",
          "---\ntype: adr\ntitle: One writer per aggregate\n"
          "description: Only the orders service writes order rows.\narea: orders\n"
          "status: accepted\ndate: 2026-03-01\n---\n\n# body\n\n## Decision\n\nText.\n")
    write(root / "docs" / "adr" / "2026-04-02-order-events.md",
          "---\ntype: adr\ntitle: Order events carry the full row\narea: orders\n"
          "status: proposed\ndate: 2026-04-02\n---\n\n# T\n\n## Decision\n\n"
          "Each order event holds the full order row, not a delta.\n")
    write(root / "docs" / "adr" / "2026-05-01-loose.md",
          "---\ntype: adr\ntitle: Loose end\ndate: 2026-05-01\n---\n\n# T\n")
    return root


def index_of(root):
    corpus = adr_lib.load_corpus(root)
    return adr_index.render_index(adr_index.nav_context(corpus))


def test_the_index_opens_with_the_front_matter_and_the_corpus_title(tmp_path):
    text = index_of(index_repo(tmp_path))
    assert text.startswith('---\nokf_version: "0.2"\n---\n\n'
                           '# Architecture decision records\n\n## Areas\n\n')
    assert text.endswith("\n") and not text.endswith("\n\n")


def test_the_areas_list_counts_each_status_and_puts_no_area_last(tmp_path):
    lines = index_of(index_repo(tmp_path)).splitlines()
    start = lines.index("## Areas") + 2
    assert lines[start:start + 2] == [
        "* [orders](#orders) - 2 decisions: 1 accepted, 1 proposed",
        "* [(no area)](#no-area) - 1 decision: 1 unknown",
    ]


def test_each_area_section_holds_its_status_groups_in_the_fixed_order(tmp_path):
    text = index_of(index_repo(tmp_path))
    assert "## orders\n\n### accepted\n\n" in text
    assert text.index("### accepted") < text.index("### proposed")
    assert "## (no area)\n\n### unknown\n\n" in text
    assert "### deprecated" not in text            # an empty group is left out


def test_the_line_form_holds_the_label_the_status_and_the_text(tmp_path):
    lines = index_of(index_repo(tmp_path)).splitlines()
    assert ("* [2026-03-01: One writer per aggregate](./2026-03-01-one-writer.md) - "
            "accepted. Only the orders service writes order rows.") in lines
    assert ("* [2026-04-02: Order events carry the full row](./2026-04-02-order-events.md) - "
            "proposed. Each order event holds the full order row, not a delta. (derived)"
            ) in lines


def test_a_corpus_with_no_adr_gives_the_three_block_index(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    assert index_of(root) == ('---\nokf_version: "0.2"\n---\n\n'
                              "# Architecture decision records\n\n## Areas\n")


def test_the_status_order_covers_every_status_value():
    assert set(adr_index.STATUS_ORDER) == set(adr_lib.STATUSES) | {adr_index.UNKNOWN_STATUS}


def tail_repo(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date", "archive_dir": "archive"})
    write(root / "docs" / "adr" / "2026-02-10-cache-ttl.md",
          "---\ntype: adr\ntitle: Cache TTL\narea: cache\nstatus: accepted\n"
          "date: 2026-02-10\nsuperseded_by: 2026-05-01-cache-ttl-per-route.md\n"
          "---\n\n# T\n")
    adr(root, "2026-05-01-cache-ttl-per-route.md", area="cache", title="Per route")
    write(root / "docs" / "adr" / "2026-06-01-gone.md",
          "---\ntype: adr\ntitle: Gone\narea: cache\nstatus: accepted\n"
          "date: 2026-06-01\nsuperseded_by: 2026-12-31-absent.md\n---\n\n# T\n")
    write(root / "docs" / "adr" / "archive" / "2026-01-01-use-postgres.md",
          "---\ntype: adr\ntitle: Postgres\narea: storage\nstatus: superseded\n"
          "date: 2026-01-01\nsuperseded_by:\n  - 2026-02-01-use-sqlite.md\n"
          "  - 2026-03-01-one-writer.md # partial\n---\n\n# T\n")
    adr(root, "2026-02-01-use-sqlite.md", area="storage", title="SQLite")
    adr(root, "2026-03-01-one-writer.md", area="orders", title="One writer")
    write(root / "docs" / "adr" / "archive" / "2025-08-01-old.md",
          "---\ntype: adr\ntitle: Old\narea: build\nstatus: superseded\n"
          "date: 2025-08-01\nsuperseded_by: 2026-12-31-absent.md\n---\n\n# T\n")
    return root


def tail_for(root, relative, *, from_archive=False):
    context = adr_index.nav_context(adr_lib.load_corpus(root))
    record = context.by_path[relative]
    return adr_index.index_tail(context, record, from_archive=from_archive)


def test_a_partly_replaced_adr_names_each_superseded_by_value(tmp_path):
    root = tail_repo(tmp_path)
    assert tail_for(root, "2026-02-10-cache-ttl.md") == (
        "Partly replaced by: [2026-05-01-cache-ttl-per-route.md]"
        "(./2026-05-01-cache-ttl-per-route.md).")


def test_a_missing_superseded_by_target_is_plain_text(tmp_path):
    root = tail_repo(tmp_path)
    assert tail_for(root, "2026-06-01-gone.md") == (
        "Partly replaced by: 2026-12-31-absent.md.")


def test_a_superseded_adr_names_each_chain_end_and_marks_the_partial_branch(tmp_path):
    root = tail_repo(tmp_path)
    assert tail_for(root, "archive/2026-01-01-use-postgres.md", from_archive=True) == (
        "Now: [2026-02-01-use-sqlite.md](../2026-02-01-use-sqlite.md), "
        "[2026-03-01-one-writer.md](../2026-03-01-one-writer.md) (partial).")


def test_a_superseded_adr_with_no_chain_end_says_no_successor(tmp_path):
    root = tail_repo(tmp_path)
    assert tail_for(root, "archive/2025-08-01-old.md", from_archive=True) == "No successor."


def test_an_adr_with_no_superseded_by_value_has_no_tail(tmp_path):
    root = tail_repo(tmp_path)
    assert tail_for(root, "2026-02-01-use-sqlite.md") == ""


def test_the_tail_is_written_after_the_text_on_the_index_line(tmp_path):
    root = tail_repo(tmp_path)
    line = [line for line in index_of(root).splitlines()
            if "2026-02-10-cache-ttl.md" in line][0]
    assert line.endswith(") - accepted. Partly replaced by: "
                         "[2026-05-01-cache-ttl-per-route.md]"
                         "(./2026-05-01-cache-ttl-per-route.md).")


def archive_index_of(root):
    corpus = adr_lib.load_corpus(root)
    return adr_index.render_archive_index(adr_index.nav_context(corpus))


def test_the_archive_index_has_the_fixed_title_and_no_area_list(tmp_path):
    text = archive_index_of(tail_repo(tmp_path))
    assert text.startswith('---\nokf_version: "0.2"\n---\n\n# Archived decisions\n\n## build\n')
    assert "## Areas" not in text
    assert "### superseded" not in text
    assert text.endswith("\n") and not text.endswith("\n\n")


def test_each_archive_line_ends_with_a_successor_text(tmp_path):
    lines = archive_index_of(tail_repo(tmp_path)).splitlines()
    assert "* [2025-08-01: Old](./2025-08-01-old.md) - superseded. No successor." in lines
    assert ("* [2026-01-01: Postgres](./2026-01-01-use-postgres.md) - superseded. "
            "Now: [2026-02-01-use-sqlite.md](../2026-02-01-use-sqlite.md), "
            "[2026-03-01-one-writer.md](../2026-03-01-one-writer.md) (partial).") in lines


def test_an_archived_adr_that_is_not_superseded_also_says_no_successor(tmp_path):
    root = tail_repo(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2025-09-12-shared.md",
          "---\ntype: adr\ntitle: Shared session table\narea: storage\n"
          "status: deprecated\ndate: 2025-09-12\n---\n\n# T\n")
    assert ("* [2025-09-12: Shared session table](./2025-09-12-shared.md) - deprecated. "
            "No successor.") in archive_index_of(root).splitlines()


def test_the_archive_index_holds_no_description_and_no_derived_text(tmp_path):
    root = tail_repo(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2025-10-01-texty.md",
          "---\ntype: adr\ntitle: Texty\ndescription: Do not show this.\narea: build\n"
          "status: accepted\ndate: 2025-10-01\n---\n\n# T\n\n## Decision\n\nNo.\n")
    text = archive_index_of(root)
    assert "Do not show this." not in text
    assert "(derived)" not in text


def test_an_archived_adr_with_no_area_or_no_status_still_gets_a_line(tmp_path):
    root = tail_repo(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2025-07-01-loose.md",
          "---\ntype: adr\ntitle: Loose\ndate: 2025-07-01\n---\n\n# T\n")
    lines = archive_index_of(root).splitlines()
    # One empty line follows each `## <area>` heading, as the byte example of
    # section 9.2 shows.
    assert lines[-3:] == [
        "## (no area)", "",
        "* [2025-07-01: Loose](./2025-07-01-loose.md) - unknown. No successor."]


def write_nav_repo(tmp_path):
    """The corpus of `index_repo`, with each field that `--write` needs.

    `index_repo` holds one ADR with no area and no status. The write-strict
    rule of the register stops on that record, and the stop is before the
    generator. The `--write --nav` tests give that ADR both fields. The
    generated files are the same shape.
    """
    root = index_repo(tmp_path)
    write(root / "docs" / "adr" / "2026-05-01-loose.md",
          "---\ntype: adr\ntitle: Loose end\narea: build\nstatus: accepted\n"
          "date: 2026-05-01\n---\n\n# T\n")
    return root


def test_nav_without_write_is_a_usage_error(tmp_path):
    root = write_nav_repo(tmp_path)
    result = run_script("adr_index", "--nav", cwd=root)
    assert result.returncode == 2
    assert "--nav" in result.stderr


def test_write_nav_creates_the_three_files_and_prints_one_receipt_line(tmp_path):
    root = write_nav_repo(tmp_path)
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 0, result.stderr
    lines = result.stdout.splitlines()
    assert lines[0].startswith("wrote docs/adr/README.md (")
    assert lines[1:] == ["wrote docs/adr/index.md", "wrote docs/adr/links.tsv"]
    assert (root / "docs" / "adr" / "index.md").is_file()
    assert (root / "docs" / "adr" / "links.tsv").is_file()


def test_the_archive_index_is_written_only_when_the_directory_exists(tmp_path):
    root = write_nav_repo(tmp_path)
    assert not (root / "docs" / "adr" / "archive").exists()
    run_script("adr_index", "--write", "--nav", cwd=root)
    assert not (root / "docs" / "adr" / "archive" / "index.md").exists()
    write(root / "docs" / "adr" / "archive" / "2025-01-01-old.md",
          "---\ntype: adr\ntitle: Old\narea: build\nstatus: deprecated\n"
          "date: 2025-01-01\n---\n\n# T\n")
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert "wrote docs/adr/archive/index.md" in result.stdout


def test_a_second_write_nav_run_changes_nothing_and_prints_no_receipt(tmp_path):
    root = write_nav_repo(tmp_path)
    run_script("adr_index", "--write", "--nav", cwd=root)
    before = (root / "docs" / "adr" / "index.md").read_bytes()
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 0
    assert "wrote docs/adr/index.md" not in result.stdout
    assert (root / "docs" / "adr" / "index.md").read_bytes() == before


def test_a_plain_write_keeps_the_files_current_once_navigation_is_on(tmp_path):
    root = write_nav_repo(tmp_path)
    run_script("adr_index", "--write", "--nav", cwd=root)
    adr(root, "2026-07-01-new.md", area="orders", title="New")
    result = run_script("adr_index", "--write", cwd=root)
    assert result.returncode == 0, result.stderr
    assert "wrote docs/adr/index.md" in result.stdout
    assert "2026-07-01-new.md" in (root / "docs" / "adr" / "index.md").read_text("utf-8")


def test_a_plain_write_stays_register_only_when_navigation_is_off(tmp_path):
    root = write_nav_repo(tmp_path)
    result = run_script("adr_index", "--write", cwd=root)
    assert result.returncode == 0, result.stderr
    lines = result.stdout.splitlines()
    assert len(lines) == 1 and lines[0].startswith("wrote docs/adr/README.md (")
    assert not (root / "docs" / "adr" / "index.md").exists()
    assert not (root / "docs" / "adr" / "links.tsv").exists()


def test_navigation_is_on_only_with_a_generated_index(tmp_path):
    root = index_repo(tmp_path)
    corpus = adr_lib.load_corpus(root)
    assert adr_index.navigation_is_on(corpus) is False
    write(root / "docs" / "adr" / "index.md", "---\ntype: adr\ntitle: Index\n---\n\n# T\n")
    assert adr_index.navigation_is_on(corpus) is False
    write(root / "docs" / "adr" / "index.md", '---\nokf_version: "0.2"\n---\n\n# T\n')
    assert adr_index.navigation_is_on(corpus) is True


def test_an_owner_written_index_stops_the_run_and_writes_nothing(tmp_path):
    # `write_nav_repo` gives each ADR an area and a status, so the register
    # write is green. The first run seeds `README.md`, which gives this test
    # the bytes that the stop must keep.
    root = write_nav_repo(tmp_path)
    run_script("adr_index", "--write", cwd=root)
    write(root / "docs" / "adr" / "index.md",
          "---\ntype: adr\ntitle: Index\narea: core\nstatus: accepted\n"
          "date: 2026-01-01\n---\n\n# Index\n")
    before = (root / "docs" / "adr" / "README.md").read_bytes()
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 2
    assert result.stderr.strip() == (
        "ERROR: docs/adr/index.md exists and the generator did not write it; move it first")
    assert (root / "docs" / "adr" / "README.md").read_bytes() == before
    assert not (root / "docs" / "adr" / "links.tsv").exists()


def test_an_owner_written_archive_index_stops_the_run(tmp_path):
    root = index_repo(tmp_path)
    write(root / "docs" / "adr" / "archive" / "index.md", "# My archive notes\n")
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 2
    assert "docs/adr/archive/index.md exists" in result.stderr


def test_an_old_vendored_copy_stops_the_run(tmp_path):
    root = index_repo(tmp_path)
    write(root / "scripts" / "adr" / "adr_lib.py", "def discover(corpus):\n    return []\n")
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 2
    assert result.stderr.strip() == (
        "ERROR: the vendored scripts in scripts/adr are older than this plugin; "
        "run /adr:setup --refresh first")
    assert not (root / "docs" / "adr" / "index.md").exists()


def test_a_current_vendored_copy_does_not_stop_the_run(tmp_path):
    root = write_nav_repo(tmp_path)   # the run must reach exit 0, so each ADR is canonical
    write(root / "scripts" / "adr" / "adr_lib.py",
          "def is_generated_index(path):\n    return False\n")
    assert run_script("adr_index", "--write", "--nav", cwd=root).returncode == 0


def test_a_reserved_area_slug_stops_the_run_with_exit_two(tmp_path):
    root = index_repo(tmp_path)
    adr(root, "2026-08-01-bad.md", area="related", title="Bad")
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 2
    assert result.stderr.strip() == (
        "ERROR: area slug related is a reserved word of links.tsv")
    assert not (root / "docs" / "adr" / "index.md").exists()


def test_a_superseded_by_cycle_stops_the_run_with_exit_one(tmp_path):
    root = repo(tmp_path, config={"dir": "docs/adr", "register": "docs/adr/README.md",
                                  "id_scheme": "date"})
    adr(root, "2026-01-01-a.md", area="core", title="A", status="superseded",
        superseded_by=["2026-02-01-b.md"])
    adr(root, "2026-02-01-b.md", area="core", title="B", status="superseded",
        superseded_by=["2026-01-01-a.md"])
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 1
    assert result.stderr.strip() == (
        "ERROR: superseded_by chain is a cycle: 2026-01-01-a.md -> 2026-02-01-b.md "
        "-> 2026-01-01-a.md")
    assert not (root / "docs" / "adr" / "index.md").exists()


def test_a_basename_collision_stops_the_run_with_exit_two(tmp_path):
    root = index_repo(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2026-03-01-one-writer.md",
          "---\ntype: adr\ntitle: Clash\narea: orders\nstatus: deprecated\n"
          "date: 2026-03-01\n---\n\n# T\n")
    result = run_script("adr_index", "--write", "--nav", cwd=root)
    assert result.returncode == 2
    assert "ADR filename collision" in result.stderr


def navigated(tmp_path):
    # `write_nav_repo`, not `index_repo`: the loose ADR of `index_repo` has no
    # area and no status, so the write-strict rule stops `--write` before the
    # register is written and no generated file exists to check.
    root = write_nav_repo(tmp_path)
    assert run_script("adr_index", "--write", "--nav", cwd=root).returncode == 0
    return root


def test_check_is_unchanged_when_navigation_is_off(tmp_path):
    root = write_nav_repo(tmp_path)
    run_script("adr_index", "--write", cwd=root)
    result = run_script("adr_index", "--check", cwd=root)
    assert result.returncode == 0
    assert result.stdout == "register up to date\n"


def test_check_prints_two_green_lines_when_navigation_is_on(tmp_path):
    result = run_script("adr_index", "--check", cwd=navigated(tmp_path))
    assert result.returncode == 0, result.stderr
    assert result.stdout == "register up to date\nnavigation files up to date\n"


def test_check_reports_a_stale_generated_file(tmp_path):
    root = navigated(tmp_path)
    write(root / "docs" / "adr" / "links.tsv", "source\trelation\n")
    result = run_script("adr_index", "--check", cwd=root)
    assert result.returncode == 1
    assert result.stderr.strip() == "docs/adr/links.tsv is stale against the corpus"
    assert result.stdout == "register up to date\n"


def test_check_reports_a_missing_generated_file(tmp_path):
    root = navigated(tmp_path)
    (root / "docs" / "adr" / "links.tsv").unlink()
    result = run_script("adr_index", "--check", cwd=root)
    assert result.returncode == 1
    assert result.stderr.strip() == "docs/adr/links.tsv is missing; run /adr:index"


def test_check_reports_a_cycle_and_does_not_report_drift(tmp_path):
    """The 2 ADRs of the cycle are archived, so the register stays current and
    the check reaches the navigation step."""
    root = navigated(tmp_path)
    write(root / "docs" / "adr" / "archive" / "2026-06-01-b.md",
          "---\ntype: adr\ntitle: B\narea: storage\nstatus: superseded\n"
          "date: 2026-06-01\nsupersedes: 2026-07-01-c.md\n"
          "superseded_by: 2026-07-01-c.md\n---\n\n# T\n")
    write(root / "docs" / "adr" / "archive" / "2026-07-01-c.md",
          "---\ntype: adr\ntitle: C\narea: storage\nstatus: superseded\n"
          "date: 2026-07-01\nsupersedes: 2026-06-01-b.md\n"
          "superseded_by: 2026-06-01-b.md\n---\n\n# T\n")
    result = run_script("adr_index", "--check", cwd=root)
    assert result.returncode == 1
    assert result.stdout == "register up to date\n"
    assert ("ERROR: superseded_by chain is a cycle: archive/2026-06-01-b.md -> "
            "archive/2026-07-01-c.md -> archive/2026-06-01-b.md") in result.stderr
    assert "is stale against the corpus" not in result.stderr


def test_a_stale_register_is_still_reported_first(tmp_path):
    root = navigated(tmp_path)
    register = root / "docs" / "adr" / "README.md"
    write(register, register.read_text("utf-8").replace("2026-03-01-one-writer.md", "x.md"))
    result = run_script("adr_index", "--check", cwd=root)
    assert result.returncode == 1
    assert result.stderr.strip() == "register is stale against the corpus"
    assert "navigation" not in result.stdout
