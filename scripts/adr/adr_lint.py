#!/usr/bin/env python3
"""Deterministic ADR corpus lint. Standard library only, no install step.

Hand-checks the rules adr.schema.json documents -- the required keys, the
status enum, and a parseable date -- because jsonschema is a third-party
package and this tier must run in any CI with no key and no install.
"""
import argparse
import datetime as _dt
import difflib
import posixpath
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from pathlib import PurePosixPath

sys.path.insert(0, str(Path(__file__).resolve().parent))
import adr_lib  # noqa: E402
import adr_index  # noqa: E402

ERROR, WARN = "ERROR", "WARN"
AREA_RE = re.compile(r"^[a-z][a-z0-9-]*$")
REF_PATTERNS = {
    "date": re.compile(r"^(?:.*/)?\d{4}-\d{2}-\d{2}-.+\.md$"),
    "sequential": re.compile(r"^(?:.*/)?(?!\d{4}-\d{2}-\d{2}-)\d{4}-.+\.md$"),
}


@dataclass
class Issue:
    level: str
    file: str
    rule: str
    message: str


DESCRIPTION_WORD_LIMIT = 30
def check_description(record, issues):
    """`description-length` and `description-repeats-title` (slot
    `description` of adr.yaml).

    Both rules need the key. The key is optional, so a record without it is
    silent here. A missing value is the gated rule `description-missing`.
    `record.title` falls back to the H1 line when the key is absent, which is
    the title that the rest of the lint reads.
    """
    value = record.meta.get("description")
    if not isinstance(value, str):
        return
    if "\n" in value:
        issues.append(Issue(ERROR, record.relative_path, "description-length",
                            "`description` must be one line"))
    count = len(value.split())
    if count > DESCRIPTION_WORD_LIMIT:
        issues.append(Issue(
            ERROR, record.relative_path, "description-length",
            f"`description` has {count} words; the limit is {DESCRIPTION_WORD_LIMIT}"))
    title_words = adr_lib.words(str(record.title or ""))
    value_words = adr_lib.words(value)
    repeats = title_words and (
        value_words == title_words
        or (value_words[:len(title_words)] == title_words
            and 0 <= len(value_words) - len(title_words) <= 3))
    if repeats:
        issues.append(Issue(WARN, record.relative_path, "description-repeats-title",
                            "`description` repeats the title"))


def check_required(record, issues):
    if record.source == "legacy":
        issues.append(Issue(WARN, record.filename, "legacy-metadata",
                            "legacy bold-key metadata; run /adr:migrate when ready"))
        return
    if record.frontmatter_error:
        issues.append(Issue(ERROR, record.relative_path, "yaml-valid", record.frontmatter_error))
        return
    for key in ("title", "area", "status", "date"):
        if record.meta.get(key) in (None, "", []):
            issues.append(Issue(ERROR, record.relative_path, "required-frontmatter",
                                f"frontmatter is missing `{key}`"))
    if record.area and AREA_RE.fullmatch(record.area) is None:
        issues.append(Issue(ERROR, record.relative_path, "area-slug",
                            f"area `{record.area}` must match {AREA_RE.pattern}"))
    if record.status and record.status not in adr_lib.STATUSES:
        issues.append(Issue(ERROR, record.relative_path, "status-enum",
                            f"status `{record.status}` is outside "
                            f"{', '.join(adr_lib.STATUSES)}"))
    if record.date:
        try:
            _dt.date.fromisoformat(str(record.date))
        except ValueError:
            issues.append(Issue(ERROR, record.relative_path, "date-parse",
                                f"date `{record.date}` is not YYYY-MM-DD"))


def _matches_type(value, name):
    return {
        "null": value is None,
        "string": isinstance(value, str),
        "array": isinstance(value, list),
        "object": isinstance(value, dict),
    }.get(name, True)


def _matches_definition(value, definition):
    if "anyOf" in definition:
        return any(_matches_definition(value, option) for option in definition["anyOf"])
    expected_type = definition.get("type")
    if expected_type and not _matches_type(value, expected_type):
        return False
    if "const" in definition and value != definition["const"]:
        return False
    if "enum" in definition and value not in definition["enum"]:
        return False
    if isinstance(value, str):
        if ("pattern" in definition
                and re.fullmatch(definition["pattern"], value) is None):
            return False
        if len(value) < definition.get("minLength", 0):
            return False
    if isinstance(value, list) and "items" in definition:
        return all(_matches_definition(item, definition["items"]) for item in value)
    return True


def _definition_issues(record, key, value, definition):
    issues = []
    if "const" in definition and value != definition["const"]:
        issues.append(Issue(ERROR, record.relative_path, "schema-const",
                            f"`{key}` must be `{definition['const']}`"))
    if "enum" in definition and value not in definition["enum"]:
        issues.append(Issue(ERROR, record.relative_path, "schema-enum",
                            f"`{key}` is outside {definition['enum']}"))
    if not _matches_definition(value, {name: definition[name]
                                       for name in ("type", "anyOf") if name in definition}):
        issues.append(Issue(ERROR, record.relative_path, "schema-type",
                            f"`{key}` has the wrong type"))
        return issues
    if isinstance(value, str):
        if ("pattern" in definition
                and re.fullmatch(definition["pattern"], value) is None):
            issues.append(Issue(ERROR, record.relative_path, "schema-pattern",
                                f"`{key}` does not match the configured pattern"))
        if len(value) < definition.get("minLength", 0):
            issues.append(Issue(ERROR, record.relative_path, "schema-min-length",
                                f"`{key}` must not be empty"))
    if isinstance(value, list) and "items" in definition:
        for item in value:
            if not _matches_definition(item, definition["items"]):
                issues.append(Issue(ERROR, record.relative_path, "schema-item-type",
                                    f"`{key}` contains an invalid item"))
    return issues


def check_schema(record, schema, id_scheme):
    issues = []
    properties = schema.get("properties", {})
    if not isinstance(properties, dict):
        properties = {}
    for key in record.meta:
        if key not in properties:
            issues.append(Issue(ERROR, record.relative_path, "schema-unknown-key",
                                f"frontmatter key `{key}` is not allowed"))
    for key in schema.get("required", []):
        if record.meta.get(key) in (None, "", []):
            issues.append(Issue(ERROR, record.relative_path, "schema-required",
                                f"schema requires `{key}`"))
    for key, definition in properties.items():
        value = record.meta.get(key)
        if value is not None and key != "model" and isinstance(definition, dict):
            issues.extend(_definition_issues(record, key, value, definition))

    # D14: LinkML does not write `minLength`, so the generated schema cannot
    # reject an empty `evidence` item. Keep the finding, and the id it gives
    # today, as a code check. An adopter schema that still has `minLength`
    # reports it through _definition_issues, so this check stands down there
    # and the item is never reported twice.
    evidence = record.meta.get("evidence")
    evidence_items = (properties.get("evidence") or {}).get("items")
    if (isinstance(evidence, list) and isinstance(evidence_items, dict)
            and "minLength" not in evidence_items):
        for item in evidence:
            if isinstance(item, str) and not item.strip():
                issues.append(Issue(ERROR, record.relative_path, "schema-item-type",
                                    "`evidence` contains an invalid item"))

    model = record.meta.get("model")
    model_definition = properties.get("model", {})
    if model is not None and not isinstance(model, dict):
        issues.append(Issue(ERROR, record.relative_path, "schema-type",
                            "`model` has the wrong type"))
    if isinstance(model, dict) and isinstance(model_definition, dict):
        model_properties = model_definition.get("properties", {})
        if not isinstance(model_properties, dict):
            model_properties = {}
        for key in model:
            if key not in model_properties:
                issues.append(Issue(ERROR, record.relative_path, "schema-unknown-key",
                                    f"model key `{key}` is not allowed"))
        for key in model_definition.get("required", []):
            if model.get(key) in (None, "", []):
                issues.append(Issue(ERROR, record.relative_path, "schema-required",
                                    f"model requires `{key}`"))
        for key, definition in model_properties.items():
            value = model.get(key)
            if value is None or not isinstance(definition, dict):
                continue
            if key == "status" and "enum" in definition and value not in definition["enum"]:
                issues.append(Issue(ERROR, record.relative_path, "model-status-enum",
                                    "model.status is outside the supported enum"))
                continue
            issues.extend(_definition_issues(record, f"model.{key}", value, definition))

    pattern = REF_PATTERNS[id_scheme]
    # The id scheme governs a relationship that names an ADR. A relationship
    # whose `target_kind` is `mixed` or `doc` names a document outside the
    # corpus, e.g. `deviates_from` names the requirement it departs from, so
    # the scheme does not govern it.
    for key in (key for key in adr_lib.RELATIONSHIP_KEYS
                if adr_lib.RELATIONS[key]["target_kind"] == "adr"):
        for ref in adr_lib.refs(record.meta, key):
            if pattern.fullmatch(ref) is None:
                issues.append(Issue(ERROR, record.relative_path, "reference-pattern",
                                    f"`{key}` value `{ref}` does not match {id_scheme}"))
    return issues


def line_diff(expected, actual):
    return "\n".join(difflib.unified_diff(
        actual.splitlines(), expected.splitlines(),
        fromfile="actual", tofile="expected", lineterm="",
    ))


def check_register(corpus, records, sections):
    issues = []
    if not corpus.register.is_file():
        return issues

    omitted = set(corpus.config["register"]["exclude"])
    register_records = [record for record in records if record.filename not in omitted]
    linked = {}
    # build_region files an area-less record under UNFILED, so mirror that
    # placement here or a legacy ADR is falsely flagged out of its section.
    area_by_name = {record.filename: record.area or adr_index.UNFILED
                    for record in register_records}
    for section in sections:
        for bullet in section.bullets:
            linked[bullet.filename] = linked.get(bullet.filename, 0) + 1
            if (bullet.filename in area_by_name
                    and area_by_name[bullet.filename] != section.area):
                issues.append(Issue(
                    ERROR,
                    corpus.register.name,
                    "wrong-section",
                    f"{bullet.filename} belongs under `{area_by_name[bullet.filename]}`",
                ))

    for record in register_records:
        # A titleless legacy note cannot yield a canonical bullet, so
        # build_region omits it; its legacy-metadata warning already stands.
        if not record.title:
            continue
        count = linked.get(record.filename, 0)
        if count == 0:
            issues.append(Issue(ERROR, record.filename, "orphan", "ADR has no register bullet"))
        if count > 1:
            issues.append(Issue(ERROR, record.filename, "duplicate-bullet",
                                f"ADR is linked {count} times"))

    known = {record.filename for record in register_records}
    for filename, count in linked.items():
        if filename not in known:
            issues.append(Issue(ERROR, corpus.register.name, "dead-link",
                                f"./{filename} has no live ADR ({count} references)"))

    text = corpus.register.read_text(encoding="utf-8")
    current = adr_index.block_of(text)
    # Build the comparison against the register's own warning. Expecting our
    # default here reports drift that `--write` would not actually remove.
    expected = adr_index.build_block(corpus, sections,
                                     warning=adr_index.detect_warning(text))
    if current != expected:
        issues.append(Issue(ERROR, corpus.register.name, "ordering-drift",
                            line_diff(expected, current or "")))
    return issues


def check_reference_resolution(corpus, records):
    real = set(adr_index.corpus_listing(corpus))
    issues = []
    for record in records.values():
        author_dir = PurePosixPath(record.relative_path).parent
        for key in adr_lib.GRAPH_KEYS:
            for ref in adr_lib.refs(record.meta, key):
                cleaned = ref.strip()
                if cleaned.startswith("/"):
                    issues.append(Issue(ERROR, record.relative_path, "escaping-reference",
                                        f"`{ref}` is absolute"))
                    continue
                resolved = posixpath.normpath(str(author_dir / cleaned))
                rooted = posixpath.normpath(cleaned)
                if resolved in real or rooted in real:
                    continue
                rule = ("escaping-reference" if resolved == ".." or resolved.startswith("../")
                        else "dangling-reference")
                issues.append(Issue(ERROR, record.relative_path, rule,
                                    f"`{ref}` resolves to no case-exact ADR path"))
    return issues


def reciprocal_issues(records, left, right, rule, partial_attr=None):
    issues = []
    for record in records.values():
        skipped = getattr(record, partial_attr) if partial_attr else set()
        for target in adr_lib.refs(record.meta, left):
            name = Path(target).name
            if name in skipped or name not in records:
                continue
            reverse = {Path(item).name for item in adr_lib.refs(records[name].meta, right)}
            if record.filename not in reverse:
                issues.append(Issue(ERROR, record.relative_path, rule,
                                    f"{name} does not name {record.filename} in `{right}`"))
    return issues


def check_self_references(records):
    issues = []
    for record in records.values():
        for key in adr_lib.GRAPH_KEYS:
            for ref in adr_lib.refs(record.meta, key):
                if Path(ref).name == record.filename:
                    issues.append(Issue(ERROR, record.relative_path, "self-reference",
                                        f"`{key}` names this ADR"))
    return issues


def check_supersede_pairing(records):
    issues = []
    for record in records:
        replaced_by = adr_lib.refs(record.meta, "superseded_by")
        partial = record.partial_superseded_by or set()
        if record.status == "superseded" and not replaced_by:
            issues.append(Issue(ERROR, record.relative_path, "supersede-pairing",
                                "status is `superseded` with no `superseded_by`"))
        # A record that is partly superseded stays live for the parts that were
        # not replaced, so it keeps its own status. The requirement applies only
        # when no superseder is marked partial, which is a total replacement.
        if (replaced_by and record.status != "superseded"
                and not any(Path(target).name in partial for target in replaced_by)):
            issues.append(Issue(ERROR, record.relative_path, "supersede-pairing",
                                "a non-partial `superseded_by` requires status `superseded`"))
    return issues


GATED_RULES = ("index-drift", "links-drift", "graph-cycle", "target-missing",
               "yaml-shape", "description-missing")


def cycle_issues(cycles):
    """`graph-cycle`, on the member with the lowest file name (section 10.1)."""
    return [Issue(ERROR, cycle[0], "graph-cycle",
                  "`superseded_by` chain is a cycle: " + " -> ".join(cycle))
            for cycle in cycles]


def target_missing_issues(context):
    """`target-missing`: one WARN for each stored target that no file answers.

    This is the refinement of parity D1 that the gate turns on. A `url` target
    gives no finding.
    """
    issues = []
    for record in context.live + context.archived:
        for key in adr_lib.RELATIONS:
            for value in adr_lib.refs(record.meta, key):
                state, _, _ = adr_index.resolve_target(
                    context.corpus, context.records, record.relative_path, value)
                if state == "missing":
                    issues.append(Issue(WARN, record.relative_path, "target-missing",
                                        f"`{key}` target does not exist: {value}"))
    return issues


YAML_SHAPE_KEYS = frozenset(adr_lib.LIST_KEYS) | {"parent_spec"}
# `- rel: x` is a mapping in YAML. The parser stores the whole text as one
# string, so the record keeps a value that no reader can resolve.
MAPPING_ITEM_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*:( |$)")
# `["a, b", c]` has 2 items in YAML. The parser splits on each comma and
# gives 3.
QUOTE_RE = re.compile(r"[\"']")
YAML_SHAPE_MESSAGE = "`{key}` has a YAML form that the parser reads wrongly: {value}"


def yaml_shape_issues(context):
    """The 2 YAML forms that the parser reads with no message (section 10.1).

    The rule reads the raw front matter text, not the parsed value, because
    the parsed value has lost the form. It uses the comment stripper of the
    parser, so it sees the same text that the parser sees.
    """
    issues = []
    for record in context.live + context.archived:
        active = None
        for line in (record.raw_frontmatter or "").splitlines():
            if line and not line[0].isspace():
                key, separator, value = line.partition(":")
                active = key.strip() if separator and key.strip() in YAML_SHAPE_KEYS else None
                value = adr_lib._without_comment(value.strip())
                if (active and value.startswith("[") and value.endswith("]")
                        and QUOTE_RE.search(value)):
                    issues.append(Issue(ERROR, record.relative_path, "yaml-shape",
                                        YAML_SHAPE_MESSAGE.format(key=active, value=value)))
                continue
            item = line.lstrip()
            if not active or not item.startswith("-"):
                continue
            item = adr_lib._without_comment(item[1:].strip())
            if not item.startswith(("'", '"')) and MAPPING_ITEM_RE.match(item):
                issues.append(Issue(ERROR, record.relative_path, "yaml-shape",
                                    YAML_SHAPE_MESSAGE.format(key=active, value=item)))
    return issues


def drift_issues(corpus, context, cycles):
    """`index-drift` and `links-drift` (section 10.1).

    A cycle stops the generator, so there is no output to compare. The
    `graph-cycle` ERROR already fails the run.
    """
    if cycles:
        return []
    # The caller already ran the cycle search; pass it, so the preflight and
    # the graph walk do not run a second time.
    navigation = adr_index.build_navigation(corpus, context, cycles=cycles)
    prefix = adr_index.corpus_prefix(corpus)
    issues = []
    for relative, state in adr_index.stale_files(corpus, navigation):
        field = relative[len(prefix):]
        if field == adr_index.NAV_LINKS:
            issues.append(Issue(ERROR, field, "links-drift",
                                f"links.tsv is {state}; run /adr:index"))
        else:
            issues.append(Issue(ERROR, field, "index-drift",
                                "generated index is stale; run /adr:index"))
    return issues


# The mode `adr_migrate.py --descriptions` writes this comment on a value that
# a person has not yet reviewed (section 13.3).
DRAFT_RE = re.compile(r"^description:.*#\s*draft\s*$")


def description_missing_issues(adrs, changed):
    """`description-missing`: gated, and only for the files `--changed` names.

    It uses the files of `discover()`, as the other description rules do.
    """
    if changed is None:
        return []
    names = {Path(name).name for name in changed}
    issues = []
    for record in adrs:
        if record.filename not in names:
            continue
        if record.meta.get("description") in (None, "", []):
            issues.append(Issue(WARN, record.relative_path, "description-missing",
                                "`description` is missing"))
        elif any(DRAFT_RE.match(line)
                 for line in (record.raw_frontmatter or "").splitlines()):
            issues.append(Issue(WARN, record.relative_path, "description-missing",
                                "`description` is a draft; review it and remove the comment"))
    return issues


def gated_issues(corpus, adrs, changed, records=None):
    """The rules of section 10.1 that run only when navigation is on (D16).

    `records` is None when `relationship_corpus` could not read the corpus.
    Each rule here needs that record set, so the whole group is left out. A
    second read would raise the same error out of `lint()` and would throw
    away every issue the run has already found.
    """
    if records is None or not adr_index.navigation_is_on(corpus):
        return []
    context = adr_index.nav_context(corpus, records=records)
    # The preflight runs the corpus guards and returns the cycles, so the
    # guards still stop a corpus that `links.tsv` cannot record, and
    # `drift_issues` does not have to search the graph again.
    cycles = adr_index.preflight(context)
    issues = cycle_issues(cycles)
    issues.extend(drift_issues(corpus, context, cycles))
    issues.extend(target_missing_issues(context))
    issues.extend(yaml_shape_issues(context))
    issues.extend(description_missing_issues(adrs, changed))
    return issues


def lint(corpus, changed=None):
    issues = []
    schema = adr_lib.load_schema(corpus)
    adrs = [adr_lib.parse_adr(path, corpus.id_scheme) for path in adr_lib.discover(corpus)]
    for adr in adrs:
        check_required(adr, issues)
        if adr.source != "legacy" and not adr.frontmatter_error:
            check_description(adr, issues)
            issues.extend(check_schema(adr, schema, corpus.id_scheme))
    seen = {}
    for adr in adrs:
        key = adr.prefix if corpus.id_scheme == "sequential" else adr.filename
        if key is None:
            continue
        if key in seen:
            issues.append(Issue(ERROR, adr.filename, "duplicate-identity",
                                f"`{key}` is already used by {seen[key]}"))
        else:
            seen[key] = adr.filename

    current = corpus.register.read_text(encoding="utf-8") if corpus.register.is_file() else ""
    sections = adr_index.parse_register(current)
    issues.extend(check_register(corpus, adrs, sections))

    records = None
    try:
        records = adr_lib.relationship_corpus(corpus)
    except adr_lib.CorpusError as error:
        issues.append(Issue(ERROR, "-", "relationship-collision", str(error)))
    else:
        issues.extend(check_reference_resolution(corpus, records))
        issues.extend(reciprocal_issues(
            records, "superseded_by", "supersedes", "one-sided-supersede",
            "partial_superseded_by",
        ))
        issues.extend(reciprocal_issues(
            records, "supersedes", "superseded_by", "one-sided-supersede",
            "partial_supersedes",
        ))
        issues.extend(reciprocal_issues(
            records, "amends", "amended_by", "amendment-reciprocity",
        ))
        issues.extend(reciprocal_issues(
            records, "amended_by", "amends", "amendment-reciprocity",
        ))
        issues.extend(check_self_references(records))
        issues.extend(check_supersede_pairing(adrs))

    issues.extend(gated_issues(corpus, adrs, changed, records))

    if changed is not None:
        verdict = (corpus.config["gate"] or {}).get("proposed_verdict", "error")
        if verdict != "off":
            names = {Path(name).name for name in changed}
            for adr in adrs:
                if adr.filename in names and adr.status == "proposed":
                    issues.append(Issue(ERROR if verdict == "error" else WARN,
                                        adr.filename, "still-proposed",
                                        "this ADR is still `proposed` in a changed file"))

    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description="Lint the ADR corpus")
    parser.add_argument("--changed", nargs="*", default=None)
    parser.add_argument("--config")
    parser.add_argument("--repo-root")
    args = parser.parse_args()
    try:
        corpus = adr_lib.load_corpus(args.repo_root, args.config)
        if not corpus.dir.is_dir():
            print(f"ERROR: corpus directory {corpus.dir} does not exist", file=sys.stderr)
            return 2
        issues = lint(corpus, args.changed)
    except adr_lib.CorpusError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    for issue in issues:
        print(f"{issue.level}: {issue.file}: {issue.rule}: {issue.message}")
    errors = sum(1 for issue in issues if issue.level == ERROR)
    print(f"{errors} error(s), {len(issues) - errors} warning(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
