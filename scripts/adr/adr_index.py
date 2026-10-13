#!/usr/bin/env python3
"""Generate the live ADR register between its markers. Standard library only."""
import argparse
import posixpath
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

sys.path.insert(0, str(Path(__file__).resolve().parent))
import adr_lib  # noqa: E402

SEP = " — "
BEGIN = "<!-- ADR-INDEX:BEGIN (generated — do not edit) -->"
END = "<!-- ADR-INDEX:END -->"
WARNING_BLOCKQUOTE = (
    "> _Generated from each ADR's frontmatter. Do not hand-edit between the "
    "markers. Run `/adr:index`._"
)
MISSING_SUMMARY = "<!-- TODO: summary -->"
MISSING_AREA_DESCRIPTION = "<!-- TODO: describe this area -->"
UNFILED = "unfiled"  # holds legacy ADRs that carry no frontmatter area yet
SEED = "# Architecture decisions\n\n"
SUPERSEDED_CELL_PREFIX = "superseded by "
# `status_cell()` writes this word for a record with no status. The row
# parser must read it back, or each `--write` adds one more cell (DEF-2).
UNKNOWN_STATUS_CELL = "unknown"

ROW_RE = re.compile(r"^- \[(?P<label>[^\]]*)\]\((?P<href>[^)]*)\)(?P<tail>.*)$")
HEADER_RE = re.compile(r"^### (?P<area>[a-z][a-z0-9-]*)(?:\s+—.*)?$")
BEGIN_RE = re.compile(r"^<!--\s*ADR-INDEX:BEGIN\b.*?-->[ \t]*$", re.MULTILINE)
END_RE = re.compile(r"^<!--\s*ADR-INDEX:END\s*-->[ \t]*$", re.MULTILINE)


# --- navigation: the generated index and edge files (spec sections 8.7, 9) ---

OKF_VERSION = "0.2"
NAV_INDEX = "index.md"
NAV_LINKS = "links.tsv"
INDEX_FRONTMATTER = f'---\nokf_version: "{OKF_VERSION}"\n---'
DEFAULT_CORPUS_TITLE = "Architecture decision records"
ARCHIVE_TITLE = "Archived decisions"
BREAK_RE = re.compile(r"[\t\r\n]")


def escape_index_text(value: str) -> str:
    """Fold each break into one space, then escape each square bracket.

    The parser accepts a `|` or `>` block scalar, so a title can hold a line
    break. The text ends on one index line, so the break cannot stay.
    The spec does not name the trim; a link label with a trailing space is
    invisible and would not match the same title written inline. [inferred]
    """
    return BREAK_RE.sub(" ", str(value)).strip().replace("[", r"\[").replace("]", r"\]")


def escape_href(value: str) -> str:
    """Encode the 3 characters that break a Markdown link href."""
    return value.replace(" ", "%20").replace("(", "%28").replace(")", "%29")


def corpus_title(corpus) -> str:
    """The H1 line of the register, or the fixed text when there is none."""
    if corpus.register.is_file():
        match = adr_lib.HEADING_RE.search(corpus.register.read_text(encoding="utf-8"))
        if match:
            return escape_index_text(match.group(1))
    return DEFAULT_CORPUS_TITLE


def index_label(record) -> str:
    """`<prefix>: <title>`, the title, or the file name (section 9.1).

    The separator is a colon and one space. It is not the register separator.
    """
    if record.prefix and record.title:
        return escape_index_text(f"{record.prefix}: {record.title}")
    return escape_index_text(record.title or record.filename)


# `derived_text` and its rule moved to `adr_lib.py` in phase 4, because
# `adr_migrate.py` uses the same rule. The name stays here for each caller.
derived_text = adr_lib.derived_text


TARGET_STATES = ("live", "archive", "doc", "url", "missing")
UNKNOWN_STATUS = "unknown"
EMPTY_FIELD = "-"
URL_RE = re.compile(r"^https?://")


def tsv_field(value: str) -> str:
    """One field of `links.tsv`: fold each break, trim, then write `-` for
    the empty value (section 8.7)."""
    return BREAK_RE.sub(" ", str(value)).strip() or EMPTY_FIELD


def corpus_prefix(corpus) -> str:
    """`<dir>/`, the path of the corpus directory from the repository root."""
    return corpus.dir.relative_to(corpus.root).as_posix() + "/"


def archive_prefix(corpus):
    """`<archive_dir>/`, or None when `archive_dir` is null."""
    archive = corpus.archive
    return None if archive is None else archive.relative_to(corpus.dir).as_posix() + "/"


def _is_archived(corpus, relative: str) -> bool:
    prefix = archive_prefix(corpus)
    return bool(prefix) and relative.startswith(prefix)


def resolve_target(corpus, records, author_relative: str, value: str):
    """One stored target, as (target_state, target, anchor) (section 8.4).

    The 8 steps run in order and the first hit wins. `target` is the one
    normalised form of section 8.7: a path relative to `<dir>` for an ADR, a
    path from the repository root for a document, the value as written for a
    URL, and the cleaned value for a missing target.
    """
    raw = str(value).strip()
    if URL_RE.match(raw):                                        # step 1
        return "url", raw, ""
    body, _, anchor = raw.partition("#")                         # step 2
    body = body.lstrip("/")                                      # step 3
    author_dir = PurePosixPath(author_relative).parent
    prefix = corpus_prefix(corpus)
    candidates = []
    if body.startswith(prefix):                                  # step 4
        candidates.append(posixpath.normpath(body[len(prefix):]))
    # Step 5 keeps the candidate order of `_resolved_target`: the author
    # directory first, then the corpus directory.
    candidates.append(posixpath.normpath((author_dir / body).as_posix()))
    candidates.append(posixpath.normpath(body))
    for candidate in candidates:
        record = records.get(PurePosixPath(candidate).name)
        if record is not None and record.relative_path == candidate:
            return ("archive" if _is_archived(corpus, candidate) else "live"), candidate, anchor
    rooted = posixpath.normpath(body)
    if (corpus.root / rooted).is_file():                         # step 6
        return "doc", rooted, anchor
    beside = posixpath.normpath(f"{prefix}{author_dir}/{body}")  # step 7
    if (corpus.root / beside).is_file():
        return "doc", beside, anchor
    return "missing", rooted, anchor                             # step 8


CHAIN_LIMIT = 5          # the step limit of one branch (section 8.7)


@dataclass
class NavContext:
    """The record set of the generator and its resolved successor graph."""
    corpus: object
    records: dict        # base name -> Adr, from relationship_corpus()
    by_path: dict        # <dir>-relative path -> Adr
    live: list           # Adr for each file of discover()
    archived: list       # Adr for each file of discover_archived()
    graph: dict = field(default_factory=dict)
    ends: dict = field(default_factory=dict)


def nav_context(corpus, records=None) -> NavContext:
    """Read the corpus one time and resolve the successor graph.

    A caller that has already built the record set passes it as `records`, and
    the corpus is not read again.
    """
    if records is None:
        records = adr_lib.relationship_corpus(corpus)

    def record_for(path):
        """The parsed record of `path`. `relationship_corpus` leaves out a
        README or a CLAUDE.md, which `discover_archived` still returns, so
        those few files are parsed here."""
        found = records.get(path.name)
        if found is not None:
            return found
        return adr_lib.parse_adr(path, corpus.id_scheme, root=corpus.dir)

    context = NavContext(
        corpus=corpus,
        records=records,
        by_path={record.relative_path: record for record in records.values()},
        live=[record_for(path) for path in adr_lib.discover(corpus)],
        archived=[record_for(path) for path in adr_lib.discover_archived(corpus)],
    )
    context.graph = successor_map(context)
    return context


def successor_map(context) -> dict:
    """{<dir>-relative path: [<dir>-relative path]} over each `superseded_by`
    value that resolves to an ADR. A missing target is left out: it gives no
    chain end, and `target-missing` reports it."""
    graph = {}
    for record in context.records.values():
        successors = []
        for value in adr_lib.refs(record.meta, "superseded_by"):
            state, target, _ = resolve_target(
                context.corpus, context.records, record.relative_path, value)
            if state in ("live", "archive"):
                successors.append(target)
        graph[record.relative_path] = successors
    return graph


def find_cycles(graph) -> list:
    """Each `superseded_by` cycle, written from its lowest file name and
    closed by that file again. One entry for each cycle.

    A cycle that is longer than the branch limit is outside the walk, so it is
    not reported here.
    """
    cycles = {}
    for start in sorted(graph):
        stack = [(start, (start,))]
        while stack:
            node, path = stack.pop()
            for nxt in graph.get(node, ()):
                if nxt in path:
                    members = path[path.index(nxt):]
                    first = min(range(len(members)),
                                key=lambda index: PurePosixPath(members[index]).name)
                    rotated = list(members[first:] + members[:first])
                    cycles.setdefault(frozenset(members), rotated + [rotated[0]])
                elif len(path) < CHAIN_LIMIT:
                    stack.append((nxt, path + (nxt,)))
    return [cycles[key] for key in sorted(cycles, key=sorted)]


def chain_ends(context, start: str) -> dict:
    """{chain end: the first hops of the branches that reach it}.

    A branch stops at the first ADR whose status is not `superseded`. A branch
    continues through a `# partial` value. A branch of more than 5 steps, and a
    branch that returns to a member of its own path, gives no chain end.
    """
    if start in context.ends:
        return context.ends[start]
    ends = {}
    for first in context.graph.get(start, ()):
        stack = [(first, (start, first))]
        while stack:
            node, path = stack.pop()
            if (context.by_path[node].status or "") != "superseded":
                ends.setdefault(node, set()).add(first)
                continue
            if len(path) > CHAIN_LIMIT:
                continue
            for nxt in context.graph.get(node, ()):
                if nxt not in path:
                    stack.append((nxt, path + (nxt,)))
    context.ends[start] = ends
    return ends


LINKS_HEADER = ("source", "relation", "target", "target_state", "target_status",
                "source_area", "target_area", "note")


def _row_sort_key(row):
    return (row[0], adr_lib.RELATIONS[row[1]]["walk_order"], row[1], row[2], row[7])


def edge_rows(context) -> list:
    """One row for each stored edge of each live and archived ADR (section 8.7).

    The file holds no derived row and no clock value, because `--check`
    compares bytes.
    """
    rows = []
    for record in context.live + context.archived:
        for relation in adr_lib.RELATIONS:
            marks = adr_lib.partial_targets(record.raw_frontmatter or "", relation)
            for value in adr_lib.refs(record.meta, relation):
                state, target, anchor = resolve_target(
                    context.corpus, context.records, record.relative_path, value)
                found = context.by_path.get(target) if state in ("live", "archive") else None
                notes = []
                # `partial_targets` reads the raw block, so the mark is read
                # from the same text that the parser reads.
                if PurePosixPath(value).name in marks:
                    notes.append("partial")
                if found is not None and found.status == "superseded":
                    ends = [end for end in sorted(chain_ends(context, target))
                            if end != record.relative_path]
                    if ends:
                        notes.append("now=" + ",".join(ends))
                if anchor:
                    notes.append("anchor=" + anchor.replace(";", "%3B"))
                rows.append([
                    tsv_field(record.relative_path),
                    relation,
                    tsv_field(target),
                    state,
                    tsv_field(found.status or UNKNOWN_STATUS) if found else EMPTY_FIELD,
                    tsv_field(record.area or ""),
                    tsv_field(found.area or "") if found else EMPTY_FIELD,
                    tsv_field(";".join(notes)),
                ])
    unique = dict.fromkeys(tuple(row) for row in rows)
    return [list(row) for row in sorted(unique, key=_row_sort_key)]


def render_links(rows) -> str:
    """The bytes of `links.tsv`: the header row and one row for each edge."""
    lines = ["\t".join(LINKS_HEADER)]
    lines.extend("\t".join(row) for row in rows)
    return "\n".join(lines) + "\n"


class NavigationError(adr_lib.CorpusError):
    """A stop of the generator. The caller prints the message and exits 2.

    It is a `CorpusError`, so the lint reports it through the same path that
    the lint already uses for an unreadable corpus.
    """


# A Grep on the area columns of links.tsv must not match a word of another
# column, so an area slug cannot be equal to one of these words (D48).
RESERVED_AREA_WORDS = frozenset(
    tuple(adr_lib.RELATIONS) + TARGET_STATES + tuple(adr_lib.STATUSES)
    + (UNKNOWN_STATUS, "partial", EMPTY_FIELD))


def guard_area_slugs(context):
    """Stop when an area slug is a reserved word of links.tsv (section 8.7).

    It reads every record, not the live and archived lists only. A record in a
    subdirectory of the archive is a target of `links.tsv`, so its area reaches
    the `target_area` column, but `discover_archived()` does not return it.
    """
    for record in context.records.values():
        if record.area in RESERVED_AREA_WORDS:
            raise NavigationError(
                f"area slug {record.area} is a reserved word of links.tsv")
    return None


def guard_file_names(context):
    """Stop when a file name holds `,` or `;` (section 8.7).

    `now=` separates with a comma and `note` separates with a semicolon, so
    neither character can be in a file name.
    """
    for name in sorted(context.records):
        if "," in name or ";" in name:
            raise NavigationError(
                f"file name {name} has a comma or a semicolon; "
                "links.tsv cannot record it")
    return None


NO_AREA_LABEL = "(no area)"
NO_AREA_ANCHOR = "no-area"
# The index order, which is not the order of adr_lib.STATUSES (section 9.1).
STATUS_ORDER = ("accepted", "proposed", "deprecated", "superseded", UNKNOWN_STATUS)


def status_bucket(record) -> str:
    """The group of one record. A status outside the enum counts as unknown,
    because the group list of section 9.1 is closed. [inferred]"""
    return record.status if record.status in STATUS_ORDER else UNKNOWN_STATUS


def area_of(record) -> str:
    return record.area or NO_AREA_LABEL


def area_anchor(area: str) -> str:
    return NO_AREA_ANCHOR if area == NO_AREA_LABEL else area


def area_order(records) -> list:
    """The areas by Unicode code point, with `(no area)` last."""
    names = {area_of(record) for record in records}
    order = sorted(name for name in names if name != NO_AREA_LABEL)
    return order + ([NO_AREA_LABEL] if NO_AREA_LABEL in names else [])


def _line_sort_key(record):
    """Date, then file name. A record with no date sorts after the others."""
    return (not record.date, record.date or "", record.filename)


def area_summary_line(area: str, records) -> str:
    counts = {status: 0 for status in STATUS_ORDER}
    for record in records:
        counts[status_bucket(record)] += 1
    total = len(records)
    word = "decision" if total == 1 else "decisions"
    breakdown = ", ".join(f"{counts[status]} {status}"
                          for status in STATUS_ORDER if counts[status])
    return f"* [{area}](#{area_anchor(area)}) - {total} {word}: {breakdown}"


def index_href(context, relative: str, *, from_archive: bool) -> str:
    """The href of one record, relative to the index file that holds it."""
    if not from_archive:
        return escape_href(f"./{relative}")
    prefix = archive_prefix(context.corpus)
    if prefix and relative.startswith(prefix):
        return escape_href(f"./{relative[len(prefix):]}")
    return escape_href(f"../{relative}")


def index_link(context, relative: str, *, from_archive: bool) -> str:
    """`[<file name>](<href>)` (section 9.1)."""
    href = index_href(context, relative, from_archive=from_archive)
    return f"[{PurePosixPath(relative).name}]({href})"


def index_text(record) -> str:
    """The `description`, or the derived text of section 11.1."""
    description = record.meta.get("description")
    if description:
        return escape_index_text(description)
    return escape_index_text(derived_text(record.body))


def index_tail(context, record, *, from_archive: bool) -> str:
    """The `Partly replaced by:` or `Now:` text of one index line (section 9.1).

    Step 4 of the skill starts from this text, not from the status word, so a
    live ADR that holds a `superseded_by` value gets its own form (D34).
    """
    replaced_by = adr_lib.refs(record.meta, "superseded_by")
    if record.status != "superseded":
        if not replaced_by:
            return ""
        links = []
        for value in replaced_by:
            state, target, _ = resolve_target(
                context.corpus, context.records, record.relative_path, value)
            links.append(index_link(context, target, from_archive=from_archive)
                         if state in ("live", "archive")
                         # A target that no file answers gets no link. The
                         # text is the file name of the cleaned value.
                         else PurePosixPath(target).name)
        return "Partly replaced by: " + ", ".join(links) + "."
    ends = chain_ends(context, record.relative_path)
    if not ends:
        return "No successor."
    marks = record.partial_superseded_by or set()
    partial_hops = {hop for hop in context.graph.get(record.relative_path, ())
                    if PurePosixPath(hop).name in marks}
    links = []
    for end in sorted(ends):
        link = index_link(context, end, from_archive=from_archive)
        if ends[end] & partial_hops:
            link += " (partial)"
        links.append(link)
    return "Now: " + ", ".join(links) + "."


def archive_tail(context, record) -> str:
    """The tail of one archive index line (section 9.2).

    The archive index has one form only: `Now:` for a superseded ADR that has
    a chain end, and `No successor.` for each other ADR.
    """
    if record.status == "superseded" and chain_ends(context, record.relative_path):
        return index_tail(context, record, from_archive=True)
    return "No successor."


def index_line(context, record, *, from_archive: bool, with_text: bool) -> str:
    label = index_label(record)
    href = index_href(context, record.relative_path, from_archive=from_archive)
    parts = [f"* [{label}]({href}) - {record.status or UNKNOWN_STATUS}."]
    if with_text:
        text = index_text(record)
        if text:
            parts.append(text)
    tail = (archive_tail(context, record) if from_archive
            else index_tail(context, record, from_archive=False))
    if tail:
        parts.append(tail)
    return " ".join(parts)


def render_index(context) -> str:
    """The bytes of `<dir>/index.md` (section 9.1).

    One empty line separates 2 blocks. A block is the front matter, a heading,
    or a group of list lines.
    """
    blocks = [INDEX_FRONTMATTER, f"# {corpus_title(context.corpus)}", "## Areas"]
    by_area = {}
    for record in context.live:
        by_area.setdefault(area_of(record), []).append(record)
    order = area_order(context.live)
    if order:
        blocks.append("\n".join(area_summary_line(area, by_area[area]) for area in order))
    for area in order:
        blocks.append(f"## {area}")
        for status in STATUS_ORDER:
            group = [record for record in by_area[area] if status_bucket(record) == status]
            if not group:
                continue
            blocks.append(f"### {status}")
            blocks.append("\n".join(
                index_line(context, record, from_archive=False, with_text=True)
                for record in sorted(group, key=_line_sort_key)))
    return "\n\n".join(blocks) + "\n"


def render_archive_index(context) -> str:
    """The bytes of `<archive_dir>/index.md` (section 9.2)."""
    blocks = [INDEX_FRONTMATTER, f"# {ARCHIVE_TITLE}"]
    by_area = {}
    for record in context.archived:
        by_area.setdefault(area_of(record), []).append(record)
    for area in area_order(context.archived):
        blocks.append(f"## {area}")
        blocks.append("\n".join(
            index_line(context, record, from_archive=True, with_text=False)
            for record in sorted(by_area[area], key=_line_sort_key)))
    return "\n\n".join(blocks) + "\n"


@dataclass
class Navigation:
    """The bytes of each generated file, or the cycles that stop the run."""
    files: dict          # repository-relative posix path -> text
    cycles: list


def navigation_is_on(corpus) -> bool:
    """D16: navigation is on when `<dir>/index.md` is a generated index."""
    return adr_lib.is_generated_index(corpus.dir / NAV_INDEX)


def preflight(context) -> list:
    """Run the corpus guards, then return the successor cycles.

    Each entry point runs this one time before it reads or writes the
    generated files, so one guard set governs every caller.
    """
    guard_file_names(context)
    guard_area_slugs(context)
    return find_cycles(context.graph)


def build_navigation(corpus, context=None, cycles=None) -> Navigation:
    """Build the bytes of the generated files. It writes no file.

    The files are the index, the edge file, and the archive index when the
    archive directory exists.

    The preflight runs first, so a corpus that `links.tsv` cannot record stops
    the run before the generator writes anything. A caller that has already run
    the preflight passes its `cycles`, and the preflight does not run again.
    """
    if context is None:
        context = nav_context(corpus)
    if cycles is None:
        cycles = preflight(context)
    if cycles:
        return Navigation(files={}, cycles=cycles)
    prefix = corpus_prefix(corpus)
    files = {f"{prefix}{NAV_INDEX}": render_index(context)}
    archive = corpus.archive
    if archive is not None and archive.is_dir():
        files[f"{prefix}{archive_prefix(corpus)}{NAV_INDEX}"] = render_archive_index(context)
    files[f"{prefix}{NAV_LINKS}"] = render_links(edge_rows(context))
    return Navigation(files=files, cycles=cycles)


def stale_files(corpus, navigation):
    """Each generated file that the corpus does not already hold.

    Yields `(root-relative path, "missing" | "stale")`. The writer, `--check`
    and the lint all decide staleness here, so one rule governs the three.
    """
    for relative, text in navigation.files.items():
        path = corpus.root / relative
        if not path.is_file():
            yield relative, "missing"
        elif path.read_text(encoding="utf-8") != text:
            yield relative, "stale"


def write_navigation_files(corpus, navigation) -> int:
    """Write each generated file that changed and print its receipt line."""
    for relative, _state in stale_files(corpus, navigation):
        path = corpus.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(navigation.files[relative], encoding="utf-8")
        print(f"wrote {relative}")
    return 0


# The fixed destination of the vendored library. A vendored script cannot
# import scaffold.py, so the string is held here as well.
VENDORED_LIB = "scripts/adr/adr_lib.py"
VENDORED_MARK = "def is_generated_index"


def owner_index_stop(corpus):
    """D16: an `index.md` with no `okf_version` is a file of the owner."""
    archive = corpus.archive
    candidates = [corpus.dir / NAV_INDEX]
    if archive is not None:
        candidates.append(archive / NAV_INDEX)
    for path in candidates:
        if path.is_file() and not adr_lib.is_generated_index(path):
            relative = path.relative_to(corpus.root).as_posix()
            return f"{relative} exists and the generator did not write it; move it first"
    return None


def vendored_stop(corpus):
    """Section 13.1: an old vendored copy reads `index.md` as an ADR, so the
    next CI run goes red. The guard reads the file as text; it never imports
    it, and it does not fire when this script is the vendored copy."""
    path = corpus.root / VENDORED_LIB
    if Path(__file__).resolve().parent == path.parent.resolve() or not path.is_file():
        return None
    if VENDORED_MARK in path.read_text(encoding="utf-8", errors="replace"):
        return None
    return ("the vendored scripts in scripts/adr are older than this plugin; "
            "run /adr:setup --refresh first")


def print_cycles(cycles) -> None:
    for cycle in cycles:
        print("ERROR: superseded_by chain is a cycle: " + " -> ".join(cycle),
              file=sys.stderr)


def check_navigation(corpus) -> int:
    """Compare each generated file with the generator output (section 13.1)."""
    try:
        navigation = build_navigation(corpus)
    except adr_lib.CorpusError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    if navigation.cycles:
        print_cycles(navigation.cycles)
        return 1
    stale = [f"{relative} is missing; run /adr:index" if state == "missing"
             else f"{relative} is stale against the corpus"
             for relative, state in stale_files(corpus, navigation)]
    if stale:
        for line in stale:
            print(line, file=sys.stderr)
        return 1
    print("navigation files up to date")
    return 0


@dataclass
class Bullet:
    filename: str
    summary: str
    status_cell: str = None


@dataclass
class Section:
    header_line: str
    area: str
    bullets: list[Bullet]


def area_from_header(line: str) -> str | None:
    match = HEADER_RE.match(line)
    return match.group("area") if match else None


def parse_bullet(line: str) -> Bullet | None:
    match = ROW_RE.match(line)
    if not match:
        return None
    tail = match.group("tail")
    if not tail.startswith(SEP):
        return None
    filename = Path(match.group("href").strip()).name
    if not filename:
        return None
    status_cell, summary = _status_and_summary(tail[len(SEP):])
    return Bullet(filename=filename, summary=summary, status_cell=status_cell)


def _status_and_summary(tail: str) -> tuple[str | None, str]:
    first, separator, remainder = tail.partition(SEP)
    candidate = first.strip()
    if separator and (
        candidate in adr_lib.STATUSES
        or candidate == UNKNOWN_STATUS_CELL
        or candidate.startswith(SUPERSEDED_CELL_PREFIX)
    ):
        return candidate, remainder.strip()
    return None, tail.strip()


def block_of(text: str) -> str | None:
    bounds = _bounds(text)
    return None if bounds is None else text[bounds[0]:bounds[1]]


def _bounds(text: str) -> tuple[int, int] | None:
    begin, end = BEGIN_RE.search(text), END_RE.search(text)
    if not begin or not end or end.start() < begin.end():
        return None
    stop = end.end()
    if stop < len(text) and text[stop] == "\n":
        stop += 1
    return begin.start(), stop


def parse_register(text: str) -> list[Section]:
    region = block_of(text)
    source = region if region is not None else text
    sections: list[Section] = []
    current = None
    for line in source.splitlines():
        area = area_from_header(line)
        if line.startswith("### "):
            current = Section(line, area or "", [])
            sections.append(current)
            continue
        bullet = parse_bullet(line)
        if bullet is not None and current is not None:
            current.bullets.append(bullet)
    return sections


def detect_warning(text: str) -> str:
    """Return the register's own warning blockquote, or this plugin's default.

    The warning names the command a reader should run to regenerate the region,
    and that command belongs to whichever tool owns the register. Substituting
    our wording rewrites a line this plugin does not own, so an existing warning
    is carried through verbatim — including its absence. Only a register with no
    region yet gets the default, because there is nothing to preserve.
    """
    region = block_of(text)
    if region is None:
        return WARNING_BLOCKQUOTE
    quoted: list[str] = []
    for line in region.splitlines()[1:]:  # past the BEGIN marker
        if line.startswith(">"):
            quoted.append(line)
        elif line.strip() or quoted:
            break  # the blank seam under BEGIN is not yet the end of the warning
    return "\n".join(quoted)


def detect_status_cells(sections: list[Section]) -> bool:
    bullets = [bullet for section in sections for bullet in section.bullets]
    return not bullets or any(bullet.status_cell is not None for bullet in bullets)


def corpus_listing(corpus) -> dict:
    """{corpus-relative posix path: path} for every markdown file in the tree."""
    return {
        path.relative_to(corpus.dir).as_posix(): path
        for path in corpus.dir.rglob("*.md")
    }


def _resolved_target(listing: dict, author, ref: str) -> tuple[str, str]:
    author_dir = Path(author.relative_path).parent
    candidates = [
        posixpath.normpath((author_dir / ref).as_posix()),
        posixpath.normpath(Path(ref).as_posix()),
    ]
    # A dangling superseded_by is a lint error, not a crash: fall back to the
    # author-relative path so the register still generates.
    relative = next((path for path in candidates if path in listing), candidates[0])
    return Path(relative).name, f"./{relative}"


def status_cell(record, listing: dict) -> str:
    replaced_by = adr_lib.refs(record.meta, "superseded_by")
    if record.status != "superseded" or not replaced_by:
        return record.status or UNKNOWN_STATUS_CELL
    links = [f"[{label}]({href})"
             for label, href in (_resolved_target(listing, record, ref) for ref in replaced_by)]
    return SUPERSEDED_CELL_PREFIX + ", ".join(links)


def bullet_line(record, summary: str, listing: dict, *, emit_status_cells: bool) -> str:
    label = (
        f"{record.prefix}{SEP}{record.title}"
        if record.prefix
        else (record.title or record.filename)
    )
    base = f"- [{label}](./{record.filename}){SEP}"
    if emit_status_cells:
        return f"{base}{status_cell(record, listing)}{SEP}{summary}"
    return f"{base}{summary}"


def build_region(corpus, sections: list[Section], *, emit_status_cells=None, records=None,
                 warning: str = WARNING_BLOCKQUOTE) -> str:
    if emit_status_cells is None:
        emit_status_cells = detect_status_cells(sections)
    if records is None:
        records = _canonical_records(corpus)
    summaries = {
        bullet.filename: bullet.summary
        for section in sections
        for bullet in section.bullets
    }
    headers = {section.area: section.header_line for section in sections if section.area}
    existing_order = []
    for section in sections:
        if section.area and section.area not in existing_order:
            existing_order.append(section.area)

    by_area = {}
    for record in records:
        if record.title:
            by_area.setdefault(record.area or UNFILED, []).append(record)
    areas = [area for area in existing_order if area in by_area]
    areas.extend(
        sorted(
            (area for area in by_area if area not in areas),
            key=lambda name: (name == UNFILED, name),
        )
    )

    listing = corpus_listing(corpus) if emit_status_cells else {}
    parts = [warning, ""] if warning else []
    for area in areas:
        parts.append(headers.get(area, f"### {area} — {MISSING_AREA_DESCRIPTION}"))
        for record in sorted(by_area[area], key=lambda item: (item.date or "", item.filename)):
            parts.append(
                bullet_line(
                    record,
                    summaries.get(record.filename, MISSING_SUMMARY),
                    listing,
                    emit_status_cells=emit_status_cells,
                )
            )
        parts.append("")
    while parts and parts[-1] == "":
        parts.pop()
    return "\n".join(parts)


def build_block(corpus, sections: list[Section], *, records=None,
                warning: str = WARNING_BLOCKQUOTE) -> str:
    # The marker sits on its own line with a blank seam on both sides, so the
    # region reads as a block rather than running into the warning under it.
    region = build_region(corpus, sections, records=records, warning=warning)
    return f"{BEGIN}\n\n{region}\n\n{END}\n"


def inject(before: str, wrapped: str, after: str) -> str:
    return f"{before.rstrip(chr(10))}\n\n{wrapped}\n\n{after.lstrip(chr(10))}"


def unparsed_rows(text: str) -> list[str]:
    """Return each register line that looks like a row and that
    `parse_bullet` cannot read.

    `parse_register` drops such a line with no message, and the next
    `--write` then deletes the hand summary that the line holds.

    The scan reads the same source as `parse_register`, and it starts only
    below an area header. A plain Markdown list above the region is prose
    of the owner, not a register row. [inferred: the spec names the line
    kind, not the scan bound.]
    """
    region = block_of(text)
    source = region if region is not None else text
    rows: list[str] = []
    in_section = False
    for line in source.splitlines():
        if line.startswith("### "):
            in_section = True
            continue
        if in_section and line.startswith("- ") and parse_bullet(line) is None:
            rows.append(line)
    return rows


def dropped_rows(sections: list[Section], records) -> list[Bullet]:
    """Return each register bullet that the next `--write` does not write
    again.

    `build_region` keys a hand summary by the file name of the bullet, and
    it writes a bullet only for a record that has a title. A bullet whose
    file is gone, excluded or title-less thus loses its summary.
    """
    kept = {record.filename for record in records if record.title}
    return [bullet
            for section in sections
            for bullet in section.bullets
            if bullet.filename not in kept]


def _canonical_records(corpus):
    return [
        adr_lib.parse_adr(path, corpus.id_scheme, root=corpus.dir)
        for path in adr_lib.discover(corpus)
    ]


def _noncanonical(records, tracked) -> list[str]:
    # Read-tolerant: a legacy ADR still carrying bold-key headers, and any ADR
    # the register already tracks (migration in progress), render under the
    # unfiled section with placeholders. Write-strict only bites a fresh ADR
    # that has adopted frontmatter but left a canonical field empty.
    return [
        record.filename
        for record in records
        if record.source == "frontmatter"
        and record.filename not in tracked
        and not all((record.prefix, record.title, record.area, record.date, record.status))
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description="Regenerate the ADR register")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--config")
    parser.add_argument("--repo-root")
    parser.add_argument("--nav", action="store_true",
                        help="turn navigation on: also write the generated index and "
                             "links files")
    args = parser.parse_args()
    if args.nav and not args.write:
        parser.error("--nav is a modifier of --write")

    corpus = adr_lib.load_corpus(args.repo_root, args.config)
    # Read the corpus and run the guards before the register is written, so a
    # stop of section 13.1 writes no file.
    navigation_context = None
    if args.write and (args.nav or navigation_is_on(corpus)):
        # The stops run on each run that writes a generated file, not on
        # `--write --nav` only: a plain `--write` would otherwise write over an
        # archive index that the owner wrote. [inferred]
        stop = owner_index_stop(corpus) or vendored_stop(corpus)
        if stop:
            print(f"ERROR: {stop}", file=sys.stderr)
            return 2
        try:
            navigation_context = nav_context(corpus)
            cycles = preflight(navigation_context)
        except adr_lib.CorpusError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 2
        if cycles:
            # A cycle is found here, before the register is written, because
            # a stop of section 13.1 writes no file.
            print_cycles(cycles)
            return 1
    current = corpus.register.read_text(encoding="utf-8") if corpus.register.is_file() else ""
    # `nav_context` parsed the same live files with the same arguments, so a
    # second read of the corpus would give the same records.
    records = (navigation_context.live if navigation_context is not None
               else _canonical_records(corpus))
    sections = parse_register(current)
    tracked = {bullet.filename for section in sections for bullet in section.bullets}
    invalid = _noncanonical(records, tracked)
    if invalid:
        print(
            "ERROR: cannot produce a canonical bullet for " + ", ".join(invalid),
            file=sys.stderr,
        )
        return 1

    block = build_block(corpus, sections, records=records, warning=detect_warning(current))
    if not args.write:
        if block_of(current) != block:
            print("register is stale against the corpus", file=sys.stderr)
            return 1
        print("register up to date")
        return check_navigation(corpus) if navigation_is_on(corpus) else 0

    corpus.register.parent.mkdir(parents=True, exist_ok=True)
    bounds = _bounds(current)
    body = block.rstrip("\n")  # inject re-adds the seams; SEED keeps the file-final newline
    fresh = (
        inject(current[:bounds[0]], body, current[bounds[1]:])
        if bounds is not None
        else inject(current, body, "") if current else SEED + block
    )
    if fresh != current:
        corpus.register.write_text(fresh, encoding="utf-8")
    removed = dropped_rows(sections, records)
    unreadable = unparsed_rows(current)
    # The receipt is the only report that the skill reads, so a lost hand
    # summary must be named here. The exit code stays 0: a removed row is
    # a report, not a failure.
    for bullet in removed:
        print(f"dropped row: {bullet.filename}{SEP}{bullet.summary}")
    for row in unreadable:
        print(f"unparsed row: {row}")
    if removed or unreadable:
        print(f"rows removed: {len(removed)} dropped, "
              f"{len(unreadable)} unparsed")
    print(f"wrote {corpus.register.relative_to(corpus.root).as_posix()} "
          f"({len(records)} ADRs indexed)")
    if navigation_context is None:
        return 0
    # Render after the register is written. The index title is the H1 of the
    # register, and this run can seed that file. A render before the write
    # would put the fixed title in the index, and the next run would change
    # it. The spec does not name the order; this is the order that makes
    # `--write --nav` give the same bytes each run. [inferred]
    return write_navigation_files(
        corpus, build_navigation(corpus, navigation_context, cycles=cycles))


if __name__ == "__main__":
    sys.exit(main())
