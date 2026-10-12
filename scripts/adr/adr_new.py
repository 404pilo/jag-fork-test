#!/usr/bin/env python3
"""Create one ADR. Standard library only.

The filename is the identity: this writes no `id` field. Under
'sequential' the next four-digit prefix is one above the highest in use,
counting archived files, so a retired prefix is never reused.
"""
import argparse
import datetime as _dt
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import adr_lib  # noqa: E402

BODY_HEADINGS = ("## Context", "## Decision", "## Why",
                 "## Alternatives considered", "## Consequences")


def next_id(corpus, today=None, *, paths=None) -> str:
    if corpus.id_scheme == "date":
        return today or _dt.date.today().isoformat()
    if paths is None:
        paths = list(adr_lib.discover(corpus)) + list(adr_lib.discover_archived(corpus))
    used = []
    for path in paths:
        prefix = adr_lib.prefix_of(path.name, "sequential")
        if prefix is not None:
            used.append(int(prefix))
    return f"{(max(used) + 1) if used else 1:04d}"


def slugify(title: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    if len(slug) > 60:
        slug = slug[:60].rsplit("-", 1)[0]
    return slug or "untitled"


def body_for(prefix: str, title: str) -> str:
    lines = [f"# ADR-{prefix}: {title}", ""]
    for heading in BODY_HEADINGS:
        lines += [heading, "", ""]
    return "\n".join(lines).rstrip("\n") + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Create one ADR")
    parser.add_argument("--title", required=True)
    parser.add_argument("--description",
                        help="one line of 30 words or fewer, in the words of the task")
    parser.add_argument("--area", required=True)
    parser.add_argument("--status", default="proposed", choices=list(adr_lib.STATUSES))
    parser.add_argument("--date")
    parser.add_argument("--id", help="override the allocated prefix")
    parser.add_argument("--supersedes", action="append", default=[])
    parser.add_argument("--related", action="append", default=[])
    parser.add_argument("--amends", action="append", default=[])
    parser.add_argument("--refines", action="append", default=[])
    parser.add_argument("--deviates-from", action="append", default=[])
    parser.add_argument("--parent-spec")
    parser.add_argument("--evidence", action="append", default=[])
    parser.add_argument("--model-status", choices=adr_lib.MODEL_STATUSES)
    parser.add_argument("--model-element")
    parser.add_argument("--model-reason")
    parser.add_argument("--config")
    parser.add_argument("--repo-root")
    args = parser.parse_args()

    corpus = adr_lib.load_corpus(args.repo_root, args.config)
    if not corpus.dir.is_dir():
        print(f"ERROR: corpus directory {corpus.dir} does not exist", file=sys.stderr)
        return 2
    if args.description:
        # An old vendored schema or a custom schema has no `description`
        # property, and every new ADR would then get `schema-unknown-key`.
        # Stop here, and write no file.
        refusal = adr_lib.descriptions_guard(corpus)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    today = args.date or _dt.date.today().isoformat()
    paths = list(adr_lib.discover(corpus)) + list(adr_lib.discover_archived(corpus))
    prefix = args.id or next_id(corpus, today=today, paths=paths)

    if corpus.id_scheme == "sequential":
        taken = {adr_lib.prefix_of(p.name, "sequential") for p in paths}
        if prefix in taken:
            print(f"ERROR: prefix {prefix} is already in use", file=sys.stderr)
            return 1

    path = corpus.dir / f"{prefix}-{slugify(args.title)}.md"
    if path.exists():
        print(f"ERROR: {path.name} already exists", file=sys.stderr)
        return 1

    meta = {
        "type": "adr", "title": args.title, "area": args.area,
        "status": args.status, "date": today,
        "supersedes": args.supersedes or None,
        "superseded_by": None,
        "amends": args.amends or None,
        "refines": args.refines or None,
        "deviates_from": args.deviates_from or None,
        "related": args.related,
    }
    if args.description:
        meta["description"] = args.description
    if args.parent_spec:
        meta["parent_spec"] = args.parent_spec
    if args.evidence:
        meta["evidence"] = args.evidence
    if args.model_status:
        meta["model"] = {key: value for key, value in {
            "status": args.model_status, "element": args.model_element,
            "reason": args.model_reason,
        }.items() if value is not None}
    path.write_text("---\n" + adr_lib.dump_frontmatter(meta) + "---\n\n"
                    + body_for(prefix, args.title), encoding="utf-8")
    print(path.relative_to(corpus.root).as_posix())
    return 0


if __name__ == "__main__":
    sys.exit(main())
