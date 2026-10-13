#!/usr/bin/env python3
"""Read-tolerant, write-strict ADR parser. Standard library only.

Reads two metadata styles (YAML frontmatter and legacy bold-key headers) and
writes exactly one (frontmatter). The filename is an ADR's identity; no `id`
field is ever read or written.
"""
import copy
import datetime as _dt
import json
import re
from dataclasses import dataclass
from pathlib import Path

STATUSES = ("proposed", "accepted", "deprecated", "superseded")
MODEL_STATUSES = ("modeled", "out-of-scope", "black-box")
REQUIRED_KEYS = ("type", "status", "date")
GRAPH_KEYS = ("supersedes", "superseded_by", "amends", "amended_by")
RELATIONSHIP_KEYS = GRAPH_KEYS + ("refines", "deviates_from")
LIST_KEYS = RELATIONSHIP_KEYS + ("related", "evidence")
OBJECT_KEYS = ("model",)  # frontmatter keys whose block is a mapping, not a list
KEY_ORDER = (
    "type", "title", "description", "area", "status", "date", "amended",
    "supersedes", "superseded_by", "amends", "amended_by", "refines",
    "deviates_from", "related", "parent_spec", "evidence", "model",
)

# The relation table. `schema/adr.slots.json` is the source, and
# tests/test_schema_pipeline.py keeps the two in agreement. `walk_order` is
# the order the consulting skill walks the graph in. `target_kind` says what
# a stored value names: another ADR, a document, or either one.
RELATIONS = {
    "supersedes": {"inverse": "superseded_by", "stored_inverse": True,
                   "target_kind": "adr", "walk_order": 1},
    "superseded_by": {"inverse": "supersedes", "stored_inverse": True,
                      "target_kind": "adr", "walk_order": 1},
    "amends": {"inverse": "amended_by", "stored_inverse": True,
               "target_kind": "adr", "walk_order": 2},
    "amended_by": {"inverse": "amends", "stored_inverse": True,
                   "target_kind": "adr", "walk_order": 2},
    "refines": {"inverse": None, "stored_inverse": False,
                "target_kind": "adr", "walk_order": 4},
    "deviates_from": {"inverse": None, "stored_inverse": False,
                      "target_kind": "mixed", "walk_order": 3},
    "related": {"inverse": None, "stored_inverse": False,
                "target_kind": "mixed", "walk_order": 6},
    "parent_spec": {"inverse": None, "stored_inverse": False,
                    "target_kind": "doc", "walk_order": 5},
}

FRONTMATTER_RE = re.compile(r"\A---[ \t]*\r?\n(.*?)\r?\n---[ \t]*(?:\r?\n|\Z)", re.DOTALL)


def split_frontmatter(text: str):
    """Return (frontmatter_text_or_None, body).

    Matches only when the opening '---' is line 1 and a closing '---' comes
    before any other content. A file whose only '---' is a horizontal rule
    further down falls through to the legacy reader; this never partial-parses
    a body it opened as frontmatter and then failed to close.
    """
    match = FRONTMATTER_RE.match(text)
    if not match:
        return None, text
    return match.group(1), text[match.end():]


def _scalar(value: str):
    v = value.strip()
    if len(v) >= 2 and v[0] == v[-1] and v[0] in ("'", '"'):
        inner = v[1:-1]
        # YAML's single-quoted form escapes a quote by doubling it. Dropping the
        # wrapper without undoing that renders a title as `environment''s`.
        return inner.replace("''", "'") if v[0] == "'" else inner
    if v in ("null", "~", ""):
        return None
    return v


def _inline(value: str):
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        return [_scalar(part) for part in inner.split(",")] if inner else []
    return _scalar(value)


class FrontmatterError(ValueError):
    pass


class CorpusError(ValueError):
    pass


def _without_comment(value: str) -> str:
    quote = None
    for index, character in enumerate(value):
        if character in ("'", '"'):
            quote = None if quote == character else character if quote is None else quote
        if character == "#" and quote is None and (index == 0 or value[index - 1].isspace()):
            return value[:index].rstrip()
    return value.strip()


BLOCK_SCALAR_RE = re.compile(r"\A([|>])([+-]?)[0-9]*\Z")


def _indent_width(raw: str) -> int:
    return len(raw) - len(raw.lstrip(" \t"))


def _read_block_scalar(lines, index, owner_indent, indicator):
    """Read a `|` or `>` block scalar body, returning (value, next index).

    A record's `model.reason` is written as a folded `>-` block, so a parser
    that refused block scalars could not read that record at all. Nothing
    checks the prose itself, but the mapping around it has to parse.

    `-` strips the trailing newline and every other chomping form clips to
    one, which is the whole of the subset this gate accepts.
    """
    style, chomp = BLOCK_SCALAR_RE.match(indicator).groups()
    body = []
    while index < len(lines):
        raw = lines[index]
        if raw.strip() and _indent_width(raw) <= owner_indent:
            break
        body.append(raw)
        index += 1
    while body and not body[-1].strip():
        body.pop()
    content = [line for line in body if line.strip()]
    if not content:
        return "", index
    strip = min(_indent_width(line) for line in content)
    body = [line[strip:] if line.strip() else "" for line in body]
    if style == "|":
        value = "\n".join(body)
    else:
        value = ""
        for line in body:
            if not line:
                value += "\n"
            elif value and not value.endswith("\n"):
                value += " " + line
            else:
                value += line
    return (value if chomp == "-" else value + "\n"), index


def parse_simple_yaml(text: str) -> dict:
    """Parse the deterministic YAML subset accepted by the no-install gate."""
    data = {}
    parent = None
    lines = text.splitlines()
    index = 0
    while index < len(lines):
        raw = lines[index]
        number = index + 1
        index += 1
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        if raw.startswith((" ", "\t")):
            if parent is None:
                raise FrontmatterError(
                    f"unsupported YAML at line {number}: unexpected indent")
            child = raw.strip()
            if child.startswith("-"):
                if not isinstance(data[parent], list):
                    raise FrontmatterError(
                        f"unsupported YAML at line {number}: mixed mapping/list")
                item = _without_comment(child[1:].strip())
                if not item:
                    raise FrontmatterError(
                        f"unsupported YAML at line {number}: nested value")
                data[parent].append(_scalar(item))
            else:
                if not isinstance(data[parent], dict) or ":" not in child:
                    raise FrontmatterError(
                        f"unsupported YAML at line {number}: nested value")
                key, value = child.split(":", 1)
                value = _without_comment(value.strip())
                if key.strip() and BLOCK_SCALAR_RE.match(value):
                    folded, index = _read_block_scalar(
                        lines, index, _indent_width(raw), value)
                    data[parent][key.strip()] = folded
                    continue
                if not key.strip() or not value or value.startswith("{"):
                    raise FrontmatterError(
                        f"unsupported YAML at line {number}: scalar form")
                data[parent][key.strip()] = _inline(value)
            continue
        if ":" not in raw:
            raise FrontmatterError(f"unsupported YAML at line {number}: expected key")
        key, value = raw.split(":", 1)
        key, value = key.strip(), _without_comment(value.strip())
        if not key:
            raise FrontmatterError(f"unsupported YAML at line {number}: expected key")
        if BLOCK_SCALAR_RE.match(value):
            # A block scalar reads as one string, so a key that must hold a
            # list cannot take one. Recovering it as a one-item list would be
            # the partial recovery this parser refuses to do.
            if key.lower() in LIST_KEYS:
                raise FrontmatterError(
                    f"unsupported YAML at line {number}: block scalar under a list key")
            data[key], index = _read_block_scalar(
                lines, index, _indent_width(raw), value)
            parent = None
            continue
        # Only a flow mapping is out of subset. A brace INSIDE a quoted scalar is
        # ordinary prose, and rejecting it made the record unreadable — which
        # silently dropped its half of every relationship it declared.
        if value.startswith("{"):
            raise FrontmatterError(f"unsupported YAML at line {number}: scalar form")
        if value:
            data[key] = _inline(value)
            parent = None
        else:
            parent = key
            data[parent] = {} if parent.lower() in OBJECT_KEYS else []
    return _normalize(data)


def _normalize(data: dict) -> dict:
    out = {}
    for key, value in (data or {}).items():
        if isinstance(value, (_dt.datetime, _dt.date)):
            value = value.isoformat()[:10]
        elif isinstance(value, list):
            value = [str(item) for item in value if item is not None]
        elif isinstance(value, dict):
            value = _normalize(value)
        elif value is not None and not isinstance(value, str):
            value = str(value)
        name = str(key).strip().lower()
        if isinstance(value, list) and not value and name not in LIST_KEYS:
            value = None
        out[name] = value
    return out


def load_frontmatter(text: str) -> dict:
    return parse_simple_yaml(text)


def partial_targets(block: str, key: str) -> set[str]:
    """Return basenames marked with a ``# partial`` inline comment.

    The mark appears in both shapes the corpus writes: on a list item under the
    key, and on the key's own line when the value is a plain scalar. Reading
    only the list shape made a scalar partial supersession look total.
    """
    targets = set()
    active = False
    for line in block.splitlines():
        if line and not line[0].isspace():
            active = line.startswith(f"{key}:")
            if active and "#" in line:
                value, comment = line[len(key) + 1:].split("#", 1)
                name = _scalar(value.strip()) if "partial" in comment.lower() else None
                if name:
                    targets.add(Path(name).name)
            continue
        if not active or not line.lstrip().startswith("-") or "#" not in line:
            continue
        value, comment = line.lstrip()[1:].split("#", 1)
        if "partial" in comment.lower():
            targets.add(Path(_scalar(value.strip())).name)
    return targets


def _frontmatter_lines(text: str) -> tuple[list[str], str, int, int]:
    newline = "\r\n" if "\r\n" in text else "\n"
    lines = text.splitlines(keepends=True)
    if not lines or lines[0].rstrip("\r\n") != "---":
        raise FrontmatterError("text has no leading frontmatter")
    end = next((index for index, line in enumerate(lines[1:], 1)
                if line.rstrip("\r\n") == "---"), None)
    if end is None:
        raise FrontmatterError("frontmatter is not closed")
    return lines, newline, 1, end


def set_scalar(text: str, key: str, value: str, comment: str = None) -> str:
    """Replace or insert one frontmatter scalar without reserializing it.
    `comment` applies to a line that this call inserts. A line that already
    exists keeps the comment that its author wrote."""
    lines, newline, start, end = _frontmatter_lines(text)
    for index in range(start, end):
        if lines[index].startswith(f"{key}:"):
            comment = ""
            raw = lines[index].rstrip("\r\n")
            if " #" in raw:
                comment = " #" + raw.split(" #", 1)[1]
            lines[index] = f"{key}: {_emit(value)}{comment}{newline}"
            return "".join(lines)

    rank = KEY_ORDER.index(key) if key in KEY_ORDER else len(KEY_ORDER)
    insert_at = next(
        (index for index in range(start, end)
         if lines[index] and not lines[index][0].isspace()
         and lines[index].split(":", 1)[0] in KEY_ORDER
         and KEY_ORDER.index(lines[index].split(":", 1)[0]) > rank),
        end,
    )
    suffix = f" # {comment}" if comment else ""
    lines.insert(insert_at, f"{key}: {_emit(value)}{suffix}{newline}")
    return "".join(lines)


def rename_list_item(text: str, key: str, old: str, new: str) -> str:
    """Rename matching items while retaining their authored inline comments."""
    lines, newline, start, end = _frontmatter_lines(text)
    active = False
    for index in range(start, end):
        line = lines[index]
        if line and not line[0].isspace():
            active = line.startswith(f"{key}:")
            continue
        if active and line.lstrip().startswith("-"):
            prefix, value = line.split("-", 1)
            authored, separator, comment = value.rstrip("\r\n").partition(" #")
            if _scalar(authored.strip()) == old:
                suffix = f" #{comment}" if separator else ""
                lines[index] = f"{prefix}- {_emit(new)}{suffix}{newline}"
    return "".join(lines)


def append_list_item(text: str, key: str, value: str, comment: str = None) -> str:
    """Append a list item, converting empty/null forms in place when needed."""
    lines, newline, start, end = _frontmatter_lines(text)
    rendered = f"  - {_emit(value)}" + (f" # {comment}" if comment else "") + newline
    for index in range(start, end):
        if not lines[index].startswith(f"{key}:"):
            continue
        if lines[index].split(":", 1)[1].strip() in {"null", "[]", ""}:
            lines[index] = f"{key}:{newline}"
            lines.insert(index + 1, rendered)
            return "".join(lines)
    with_key = set_scalar(text, key, "[]")
    return append_list_item(with_key, key, value, comment)


LEGACY_RE = re.compile(
    r"^\*\*(Status|Date|Supersedes|Superseded By|Type|Area|Related)\s*:\*\*\s*(.+?)\s*$",
    re.IGNORECASE,
)

# How many raw lines from the start of the file parse_legacy will scan for
# bold-key headers. Callers that need to mirror parse_legacy's own window
# (e.g. adr_migrate's fold, which must never touch a bold-key-shaped line
# outside it) import this constant rather than hardcoding 20 again.
LEGACY_HEADER_WINDOW = 20


def parse_legacy(text: str) -> dict:
    """Bold-key headers in the first 20 raw lines, blank lines counted, from
    the start of the file. Keys outside the closed set are ignored, never
    warned on. A status outside the enum is kept verbatim for the lint to
    report; it is never guessed at or split."""
    data = {}
    for raw in text.splitlines()[:LEGACY_HEADER_WINDOW]:
        match = LEGACY_RE.match(raw.strip())
        if not match:
            continue
        key = match.group(1).strip().lower().replace(" ", "_")
        value = match.group(2).strip()
        if key in LIST_KEYS:
            data[key] = [part.strip() for part in value.split(",") if part.strip()]
        elif key == "status":
            data[key] = value.lower()
        else:
            data[key] = value
    return data


HEADING_RE = re.compile(r"^#\s+(.*\S)\s*$", re.MULTILINE)
ADR_MARKER_RE = re.compile(r"^adr[-\s]?\d{3,4}\s*(?::|—|-|\s)\s*", re.IGNORECASE)
SEQ_PREFIX_RE = re.compile(r"^(\d{4})(?:[-_]|$)")
DATE_PREFIX_RE = re.compile(r"^(\d{4}-\d{2}-\d{2})(?:[-_]|$)")


WORD_BREAK_RE = re.compile(r"[^a-z0-9]+")


def words(value: str) -> list:
    """Lower case, then every character that is not a letter or a digit
    becomes a space. Punctuation must not make two equal texts differ.

    The lint and the navigation measurement compare terms with this rule, so
    the two cannot give a different answer for the same text.
    """
    return WORD_BREAK_RE.sub(" ", value.lower()).split()


def heading_title(body: str):
    match = HEADING_RE.search(body)
    if not match:
        return None
    return ADR_MARKER_RE.sub("", match.group(1).strip()).strip() or None


# --- the derived index text of an ADR with no `description` (section 11.1) ---

DECISION_HEADING = "## Decision"
H1_WINDOW = 60          # the line limit of the H1 rule (D40)
DERIVED_WORD_LIMIT = 25
DERIVED_SUFFIX = " (derived)"
SENTENCE_END_RE = re.compile(r"[.!?](?=\s)")
LIST_MARK_RE = re.compile(r"^\s*(?:[-*+]|\d+[.)])\s+")
QUOTE_MARK_RE = re.compile(r"^\s*>\s?")
LINK_RE = re.compile(r"\[([^\]]*)\]\([^)]*\)")
# A mark that sits between 2 word characters is part of the word, e.g.
# snake_case, so only a mark at a word edge is removed.
WORD_MARK_RE = re.compile(r"\*\*|__|`|(?<![A-Za-z0-9])[*_]|[*_](?![A-Za-z0-9])")


def _decision_lines(body: str) -> list:
    """The lines below the exact `## Decision` heading, or the H1 window."""
    lines = body.splitlines()
    for index, line in enumerate(lines):
        if line.strip() == DECISION_HEADING:
            return lines[index + 1:]
    for index, line in enumerate(lines):
        if line.startswith("# "):
            return lines[index + 1:index + 1 + H1_WINDOW]
    return []


def _first_block(lines) -> list:
    """The first text block: the lines up to the first empty line."""
    block = []
    for line in lines:
        if line.startswith("## "):
            break
        if not line.strip():
            if block:
                break
            continue
        block.append(line)
    return block


def derived_text(body: str) -> str:
    """The index text of an ADR that has no `description` (section 11.1).

    The generator never writes this text into the ADR. A table pipe is not in
    the mark list of step 3, so a pipe stays in the text.
    """
    block = _first_block(_decision_lines(body))
    if not block:
        return ""
    first = block[0]
    if (LIST_MARK_RE.match(first) or QUOTE_MARK_RE.match(first)
            or first.lstrip().startswith("|")):
        text = first                      # the first item, quote line or row
    else:
        text = " ".join(line.strip() for line in block)
    text = LIST_MARK_RE.sub("", text, count=1)
    text = QUOTE_MARK_RE.sub("", text, count=1)
    text = WORD_MARK_RE.sub("", LINK_RE.sub(r"\1", text))
    match = SENTENCE_END_RE.search(text)
    if match:
        text = text[:match.end()]
    words = text.split()
    if not words:
        return ""
    if len(words) > DERIVED_WORD_LIMIT:
        # Step 5 says "cut it to 25 words and add `...`", so the dots join the
        # 25th word and the cut text holds 25 words. Section 13.3 reads the
        # same way: `--descriptions` removes "the final `...`". [inferred]
        return " ".join(words[:DERIVED_WORD_LIMIT]) + "..." + DERIVED_SUFFIX
    return " ".join(words) + DERIVED_SUFFIX


def description_draft(body: str) -> str:
    """The value that `adr_migrate.py --descriptions` stores: steps 1 to 5 of
    the derived-text rule, with the two index marks removed. The index text of
    step 6 ends with ` (derived)`, and a text that the 25-word cut shortened
    ends with `...`. A stored `description` is a plain sentence, so this
    helper removes both marks. It gives an empty string when the body has no
    source block."""
    text = derived_text(body)
    if text.endswith(DERIVED_SUFFIX):
        text = text[: -len(DERIVED_SUFFIX)]
    if text.endswith("..."):
        text = text[: -len("...")]
    return text.strip()


def prefix_of(filename: str, id_scheme: str = "sequential"):
    """The filename is the identity. Under 'sequential' the four-digit numeric
    prefix identifies the ADR; under 'date' the YYYY-MM-DD prefix does."""
    stem = str(filename).rsplit("/", 1)[-1]
    pattern = DATE_PREFIX_RE if id_scheme == "date" else SEQ_PREFIX_RE
    match = pattern.match(stem)
    return match.group(1) if match else None


@dataclass
class Adr:
    path: Path
    filename: str
    relative_path: str
    source: str          # "frontmatter" | "legacy"
    meta: dict
    body: str
    title: str = None
    status: str = None
    date: str = None
    area: str = None
    prefix: str = None
    frontmatter_error: str = None
    raw_frontmatter: str = None
    partial_supersedes: set[str] = None
    partial_superseded_by: set[str] = None


def refs(meta: dict, key: str = "related"):
    value = (meta or {}).get(key)
    if value is None:
        return []
    if isinstance(value, str):
        return [value] if value.strip() else []
    return [str(item) for item in value if str(item).strip()]


def parse_adr(path, id_scheme: str = "sequential", root=None) -> Adr:
    path = Path(path)
    text = path.read_text(encoding="utf-8")
    front, body = split_frontmatter(text)
    frontmatter_error = None
    if front is not None:
        source = "frontmatter"
        try:
            meta = load_frontmatter(front)
        except FrontmatterError as error:
            meta = {}
            frontmatter_error = str(error)
    else:
        meta, source = parse_legacy(text), "legacy"
        body = text
    if root is None:
        relative_path = path.name
    else:
        try:
            relative_path = path.resolve().relative_to(Path(root).resolve()).as_posix()
        except ValueError as error:
            raise CorpusError(f"ADR path escapes corpus: {path}") from error
    return Adr(
        path=path,
        filename=path.name,
        relative_path=relative_path,
        source=source,
        meta=meta,
        body=body,
        title=meta.get("title") or heading_title(body),
        status=meta.get("status"),
        date=meta.get("date"),
        area=meta.get("area"),
        prefix=prefix_of(path.name, id_scheme),
        frontmatter_error=frontmatter_error,
        raw_frontmatter=front,
        partial_supersedes=partial_targets(front, "supersedes") if front is not None else set(),
        partial_superseded_by=(partial_targets(front, "superseded_by")
                               if front is not None else set()),
    )


DEFAULT_CONFIG = {
    "dir": "docs/adr",
    "register": {"path": "docs/adr/README.md", "exclude": []},
    "id_scheme": "sequential",
    "gate": {"deterministic": False, "model": False, "proposed_verdict": "error"},
    "archive_dir": "archive",
    "schema": None,
    "rules": None,
    "extra_corpora": [],
}
NON_ADR_BASENAMES = {"README.md", "CLAUDE.md"}


def is_generated_index(path) -> bool:
    """True when the file is a generated OKF index page (D16).

    The base name alone is not the test. A corpus can already hold an ADR
    named `index.md`. That file has no `okf_version` key, so it stays an ADR.
    """
    path = Path(path)
    if path.name != "index.md":
        return False
    try:
        front, _ = split_frontmatter(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError):
        return False
    if front is None:
        return False
    try:
        return "okf_version" in parse_simple_yaml(front)
    except FrontmatterError:
        return False


def _normalize_config(raw: dict) -> dict:
    if not isinstance(raw, dict):
        raise CorpusError("ADR config must be an object")
    config = copy.deepcopy(DEFAULT_CONFIG)
    register = raw.get("register", config["register"])
    if isinstance(register, str):
        register = {"path": register, "exclude": []}
    if not isinstance(register, dict) or not isinstance(register.get("path"), str):
        raise CorpusError("register must be a path string or an object with path")
    exclude = register.get("exclude", [])
    if not isinstance(exclude, list) or not all(isinstance(item, str) for item in exclude):
        raise CorpusError("register.exclude must be a list of basenames")
    raw_gate = raw.get("gate") or {}
    if not isinstance(raw_gate, dict):
        raise CorpusError("gate must be an object")
    config.update({key: value for key, value in raw.items()
                   if key not in {"version", "gate", "register"}})
    config["register"] = {"path": register["path"], "exclude": exclude}
    config["gate"] = {**config["gate"], **raw_gate}
    return config


@dataclass
class Corpus:
    root: Path
    config: dict

    def _path(self, value, option: str) -> Path:
        if not isinstance(value, str):
            raise CorpusError(f"{option} must be a repository-relative path")
        # `dir` and `archive` are read once per ADR by the graph walk, and
        # `resolve()` goes to the filesystem each time. The memo is keyed on
        # the configured value, so an edit to `config` gives a new answer.
        memo = self.__dict__.setdefault("_resolved_paths", {})
        key = (value, option)
        if key in memo:
            return memo[key]
        candidate = Path(value)
        if candidate.is_absolute():
            raise CorpusError(f"{option} must not be absolute")
        resolved = (self.root / candidate).resolve()
        try:
            resolved.relative_to(self.root)
        except ValueError as error:
            raise CorpusError(f"{option} escapes the repository: {value}") from error
        memo[key] = resolved
        return resolved

    @property
    def dir(self) -> Path:
        return self._path(self.config["dir"], "dir")

    @property
    def register(self) -> Path:
        return self._path(self.config["register"]["path"], "register.path")

    @property
    def id_scheme(self) -> str:
        return self.config["id_scheme"]

    @property
    def archive(self):
        name = self.config.get("archive_dir")
        if name is None:
            return None
        if not isinstance(name, str):
            raise CorpusError("archive_dir must be a directory name")
        if Path(name).is_absolute() or ".." in Path(name).parts:
            raise CorpusError(f"archive_dir escapes the corpus: {name}")
        return self.dir / name

    @property
    def schema(self) -> Path | None:
        value = self.config.get("schema")
        return None if value is None else self._path(value, "schema")

    @property
    def rules(self) -> Path | None:
        value = self.config.get("rules")
        return None if value is None else self._path(value, "rules")

    @property
    def extra_corpora(self) -> list[Path]:
        values = self.config.get("extra_corpora", [])
        if not isinstance(values, list):
            raise CorpusError("extra_corpora must be a list of repository-relative paths")
        return [self._path(value, "extra_corpora") for value in values]


# The complete keyword set that adr_lint.py reads. `minLength` is here for an
# adopter schema; the generated file never has it, because LinkML does not
# write it. `title`, `description` and `format` are ignored by the lint and
# stay accepted, so an older hand-written schema keeps loading.
SUPPORTED_SCHEMA_KEYWORDS = frozenset({
    "type", "properties", "required", "additionalProperties", "anyOf",
    "const", "enum", "pattern", "items", "minLength",
    "title", "description", "format",
    # A hand-written adopter schema starts with `$schema`, and often `$id`.
    # The lint ignores both, so both stay accepted. `$ref` and `$defs` are
    # not here: a raw LinkML output must still stop the lint.
    "$schema", "$id",
})


def _unsupported_keyword(node, path=""):
    """Return (keyword, path) for the first keyword outside the subset.

    Depth first, in file order. A `properties` map is walked by name, so an
    ADR key called `items` or `enum` is never read as a keyword. The value
    `false` is the only `additionalProperties` value in the subset. [inferred]
    """
    if isinstance(node, list):
        for position, item in enumerate(node):
            found = _unsupported_keyword(item, f"{path}/{position}")
            if found:
                return found
        return None
    if not isinstance(node, dict):
        return None
    for key, value in node.items():
        where = f"{path}/{key}"
        if key == "properties":
            if isinstance(value, dict):
                for name, sub in value.items():
                    found = _unsupported_keyword(sub, f"{where}/{name}")
                    if found:
                        return found
            continue
        if key not in SUPPORTED_SCHEMA_KEYWORDS:
            return key, where
        if key == "type" and isinstance(value, list):
            return "type", where
        if key == "additionalProperties" and value is not False:
            return "additionalProperties", where
        found = _unsupported_keyword(value, where)
        if found:
            return found
    return None


def load_schema(corpus: Corpus) -> dict:
    """Load the selected schema, defaulting to the plugin's shipped contract."""
    if corpus.schema:
        path = corpus.schema
    else:
        script_dir = Path(__file__).resolve().parent
        candidates = (
            script_dir.parent / "schemas" / "adr.schema.json",
            script_dir.parents[1] / "schemas" / "adr" / "adr.schema.json",
        )
        path = next((candidate for candidate in candidates if candidate.is_file()), candidates[0])
    if not path.is_file():
        raise CorpusError(f"schema does not exist: {path}")
    try:
        schema = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise CorpusError(f"invalid schema: {path}") from error
    if not isinstance(schema, dict):
        raise CorpusError(f"schema must be an object: {path}")
    # The mini-validator reads a flat subset. A raw LinkML output stops
    # _matches_type with a TypeError, so refuse it here with a message that
    # names the keyword and its path. main() prints the CorpusError on stderr
    # and returns 2, which is the exit code this case needs. [inferred]
    unsupported = _unsupported_keyword(schema)
    if unsupported:
        raise CorpusError(f"schema-unsupported: {unsupported[0]} at {unsupported[1]}")
    return schema


def schema_permits(schema: dict, key: str) -> bool:
    """True when the active schema has `key` as a property. A writer asks
    before it adds a key: a schema with `additionalProperties: false` and no
    such property gives `schema-unknown-key` on each file."""
    properties = schema.get("properties")
    return isinstance(properties, dict) and key in properties


SCHEMA_REFUSAL = ("ERROR: the active schema does not permit description; "
                  "run /adr:setup --refresh or add the property to your schema")


def descriptions_guard(corpus) -> str:
    """Return the error text when a `description` write must stop, or an
    empty string. Every writer that adds the key asks this first, so the
    refusal reads the same from each one."""
    try:
        schema = load_schema(corpus)
    except CorpusError as error:
        return f"ERROR: {error}"
    if not schema_permits(schema, "description"):
        return SCHEMA_REFUSAL
    return ""


def find_repo_root(start=None) -> Path:
    start = Path(start or Path.cwd()).resolve()
    for parent in (start, *start.parents):
        if (parent / ".git").exists():
            return parent
    return start


def load_corpus(repo_root=None, config_path=None) -> Corpus:
    root = Path(repo_root).resolve() if repo_root else find_repo_root()
    path = Path(config_path) if config_path else root / ".claude" / "adr.json"
    if not path.is_absolute():
        path = root / path
    raw = {}
    if path.is_file():
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            raise CorpusError(f"invalid ADR config: {path}") from error
    return Corpus(root=root, config=_normalize_config(raw))


def discover(corpus: Corpus):
    """{{DIR}}/*.md, non-recursive. The register, CLAUDE.md, and every
    subdirectory (archive/, and the vendored trees, which are not .md files at
    the top level) stay out."""
    if not corpus.dir.is_dir():
        return []
    skip = NON_ADR_BASENAMES | {corpus.register.name} | set(corpus.config["register"]["exclude"])
    return sorted((p for p in corpus.dir.glob("*.md")
                   if p.name not in skip and not is_generated_index(p)),
                  key=lambda p: p.name)


def discover_archived(corpus: Corpus):
    archive = corpus.archive
    if not archive or not archive.is_dir():
        return []
    return sorted((p for p in archive.glob("*.md") if not is_generated_index(p)),
                  key=lambda p: p.name)


def relationship_corpus(corpus: Corpus) -> dict[str, Adr]:
    paths = list(corpus.dir.glob("*.md")) if corpus.dir.is_dir() else []
    if corpus.archive and corpus.archive.is_dir():
        paths.extend(corpus.archive.rglob("*.md"))
    records = {}
    for path in sorted(paths):
        if (path.name in NON_ADR_BASENAMES or path.resolve() == corpus.register
                or is_generated_index(path)):
            continue
        if path.name in records:
            raise CorpusError(f"ADR filename collision: {path.name}")
        records[path.name] = parse_adr(path, corpus.id_scheme, root=corpus.dir)
    return records


# A `#` after a space starts a comment for `_without_comment`, so a value that
# holds one must be quoted, or the reader loses every word after it.
_NEEDS_QUOTES = re.compile(r"^[\s>|&*!%@`\[\]{}#'\"-]|:\s|\s#|\s$|^$")


def _emit(value: str) -> str:
    if _NEEDS_QUOTES.search(value):
        return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'
    return value


# Invariant 2: the file name is the identity, so `id` is never written.
# It is the one unknown key that the writer drops. [inferred: rule 7 says
# "writers keep unknown keys"; rule 12 and invariant 2 leave `id` out.]
DROPPED_KEYS = ("id",)


def _unknown_lines(key: str, value) -> list[str]:
    """Write one key that is outside `KEY_ORDER` (rule 7).

    Scalars and lists only. The parser reads no mapping other than
    `model`, so an unknown mapping never reaches this writer: the record
    fails with `yaml-valid` first. A mapping that arrives from another
    caller stops the write, because a silent drop is what rule 7 repairs.
    """
    if isinstance(value, dict):
        raise FrontmatterError(
            f"cannot write the unknown mapping key `{key}`")
    if isinstance(value, (list, tuple)):
        if not value:
            return [f"{key}: []"]
        return [f"{key}:"] + [f"  - {_emit(str(item))}" for item in value]
    if value is None:
        return [f"{key}: null"]
    return [f"{key}: {_emit(str(value))}"]


def dump_frontmatter(meta: dict) -> str:
    """Write-strict: YAML frontmatter, the known keys first in the fixed
    order, then each unknown key in source order. Never an `id` field:
    the file name is the identity."""
    lines = []
    for key in KEY_ORDER:
        if key not in meta:
            continue
        value = meta[key]
        if key in LIST_KEYS:
            items = refs(meta, key)
            if not items:
                lines.append(f"{key}: " + ("[]" if key in ("related", "evidence") else "null"))
            else:
                lines.append(f"{key}:")
                lines.extend(f"  - {_emit(str(item))}" for item in items)
        elif key == "model":
            lines.append("model:")
            lines.extend(f"  {model_key}: {_emit(str(value[model_key]))}"
                         for model_key in ("status", "element", "reason")
                         if model_key in value)
        elif value is None:
            lines.append(f"{key}: null")
        else:
            lines.append(f"{key}: {_emit(str(value))}")
    for key, value in meta.items():
        if key in KEY_ORDER or key in DROPPED_KEYS:
            continue
        lines.extend(_unknown_lines(key, value))
    return "\n".join(lines) + "\n"
