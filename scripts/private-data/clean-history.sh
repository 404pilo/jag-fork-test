#!/usr/bin/env bash
# private-data clean step: make one clean history from main.
# The spec is docs/superpowers/specs/2026-10-14-STORY-47-go-public-with-a-clean-history-design.md.
#
# The clean history keeps each commit of main, with its date, its author, and
# its message. Each match of a private pattern becomes REDACTED (in each file,
# in each commit message, and in each file name). Each .pyc file goes out of the
# history. Each #N in a commit message becomes <OLD_REPO>#N.
#
# Input (environment):
#   REPO_DIR          a git folder with the branch main. This script never writes to it.
#   OLD_REPO          owner/name of the repository that #N names.
#   OUT_DIR           the folder for clean.bundle.
#   PRIVATE_PATTERNS  required. As for check.sh.
#   GITLEAKS          required. The path of the gitleaks binary.
#   FILTER_REPO       required. The path of the file git-filter-repo.
#   CLEAN_MAILMAP     optional. The text of a mailmap. It is never a file of a repository.
#
# Exit codes: 0 the bundle is ready, 1 findings remain, 2 the step cannot run.
#
# This script never uses "set -x" and never prints a pattern, a found value,
# or the output of gitleaks, Python, or git-filter-repo.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_DIR="${REPO_DIR:-}"
OLD_REPO="${OLD_REPO:-}"
OUT_DIR="${OUT_DIR:-}"
PRIVATE_PATTERNS="${PRIVATE_PATTERNS:-}"
GITLEAKS="${GITLEAKS:-}"
FILTER_REPO="${FILTER_REPO:-}"
CLEAN_MAILMAP="${CLEAN_MAILMAP:-}"

fail() { # exit-code message
  local code=$1
  shift
  echo "private-data: $*" >&2
  exit "$code"
}

# Step 1: the inputs, and a private folder.
[[ -n $REPO_DIR && -d $REPO_DIR ]] || fail 2 "REPO_DIR must name a git folder"
[[ -n $OUT_DIR ]] || fail 2 "OUT_DIR must name a folder"
[[ $OLD_REPO =~ ^[A-Za-z0-9-]+/[A-Za-z0-9._-]+$ ]] || fail 2 "OLD_REPO must be owner/name"
[[ -n $FILTER_REPO && -f $FILTER_REPO ]] || fail 2 "FILTER_REPO must name the git-filter-repo file"
[[ -n $GITLEAKS && -x $GITLEAKS ]] || fail 2 "GITLEAKS must name the gitleaks binary"
TMP=$(mktemp -d)
chmod 700 "$TMP"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$OUT_DIR"

# Step 2: the snapshot.
source_main=$(git -C "$REPO_DIR" rev-parse --verify -q refs/heads/main) || fail 2 "the repository has no main branch"
echo "private-data: source main ${source_main:0:12}"

# Step 3: scan 1. It lists each finding. Go on after exit 0 or exit 1.
rc=0
REPO_DIR="$REPO_DIR" HISTORY=main PRIVATE_PATTERNS="$PRIVATE_PATTERNS" GITLEAKS="$GITLEAKS" \
  "$HERE/check.sh" || rc=$?
if ((rc == 2)); then
  exit 2
fi

# Step 4: check each pattern line. git-filter-repo cuts a line at the last
# "==>" and compiles it with re.compile and no flags.
numbers=()
lineno=0
while IFS= read -r line || [[ -n $line ]]; do
  lineno=$((lineno + 1))
  line="${line%"${line##*[![:space:]]}"}"
  [[ -z $line || $line == '#'* ]] && continue
  if [[ $line == *'==>'* ]]; then
    fail 2 "pattern line $lineno cannot be used"
  fi
  printf '(?i)%s' "$line" >"$TMP/re-$lineno.txt"
  prc=0
  python3 -I -c 'import re, sys; re.compile(open(sys.argv[1], "rb").read())' \
    "$TMP/re-$lineno.txt" >"$TMP/re-$lineno.log" 2>&1 || prc=$?
  if ((prc != 0)); then
    fail 2 "pattern line $lineno is not a valid Python regular expression"
  fi
  numbers+=("$lineno")
done <<<"$PRIVATE_PATTERNS"
if ((${#numbers[@]} == 0)); then
  fail 2 "the PRIVATE_PATTERNS secret is not set. Ask the owner to set it."
fi

# Step 5: the files for git-filter-repo.
: >"$TMP/text.txt"
for n in "${numbers[@]}"; do
  printf 'regex:%s==>REDACTED\n' "$(cat "$TMP/re-$n.txt")" >>"$TMP/text.txt"
done
{
  cat "$TMP/text.txt"
  printf '%s\n' 'regex:(?<![A-Za-z0-9_/&])#([0-9]+)\b==>'"$OLD_REPO"'#\1'
} >"$TMP/messages.txt"
cat >"$TMP/names.py" <<'PY'
import os
import re
if "_jag_patterns" not in globals():
    _folder = os.environ["CLEAN_TMP"]
    _files = sorted((f for f in os.listdir(_folder) if f.startswith("re-") and f.endswith(".txt")),
                    key=lambda f: int(f[3:-4]))
    globals()["_jag_patterns"] = [re.compile(open(os.path.join(_folder, f), "rb").read()) for f in _files]
if filename is None:
    return None
for _pattern in globals()["_jag_patterns"]:
    filename = _pattern.sub(b"REDACTED", filename)
return filename
PY
mailmap_args=()
if [[ -n $CLEAN_MAILMAP ]]; then
  printf '%s\n' "$CLEAN_MAILMAP" >"$TMP/mailmap"
  mailmap_args=(--mailmap "$TMP/mailmap")
fi

# Step 6: a fresh clone. This script never writes to REPO_DIR.
git clone -q --no-local --no-tags --single-branch --branch main "$REPO_DIR" "$TMP/work" >"$TMP/clone.log" 2>&1 \
  || fail 2 "the clone of main failed"

# Step 7: the clean. Never print the log: it can show an expression.
frc=0
(
  cd "$TMP/work"
  CLEAN_TMP="$TMP" python3 -I "$FILTER_REPO" \
    --replace-text "$TMP/text.txt" \
    --replace-message "$TMP/messages.txt" \
    --filename-callback "$TMP/names.py" \
    --invert-paths --path-glob '*.pyc' \
    "${mailmap_args[@]}"
) >"$TMP/filter-repo.log" 2>&1 || frc=$?
if ((frc != 0)); then
  fail 2 "the clean failed (exit $frc)"
fi
echo "private-data: clean"

# Step 8: scan 2 of the clean main.
rc=0
REPO_DIR="$TMP/work" HISTORY=main PRIVATE_PATTERNS="$PRIVATE_PATTERNS" GITLEAKS="$GITLEAKS" \
  "$HERE/check.sh" >"$TMP/scan2.out" 2>&1 || rc=$?
cat "$TMP/scan2.out"
if ((rc == 2)); then
  exit 2
fi
if ((rc == 1)) || grep -q '^private-data: binary files: ' "$TMP/scan2.out"; then
  fail 1 "findings remain after the clean. Revoke each key and add a pattern line for it. For a binary file, send an ASK. Then run again."
fi

# Step 9: the last tree of main must not change.
if [[ $(git -C "$TMP/work" rev-parse 'main^{tree}') != "$(git -C "$REPO_DIR" rev-parse 'main^{tree}')" ]]; then
  fail 1 "the clean changed the last tree of main. The last tree holds private data or a .pyc file, or a pattern matches more text in Python than in RE2."
fi

# Step 10: the bundle.
git -C "$TMP/work" bundle create "$OUT_DIR/clean.bundle" refs/heads/main >"$TMP/bundle.log" 2>&1 \
  || fail 2 "the bundle failed"
git -C "$TMP/work" bundle verify "$OUT_DIR/clean.bundle" >"$TMP/verify.log" 2>&1 \
  || fail 2 "the bundle is not valid"
clean_main=$(git -C "$TMP/work" rev-parse refs/heads/main)
count=$(git -C "$TMP/work" rev-list --count refs/heads/main)
echo "private-data: clean history ready: $count commits, last commit ${clean_main:0:12}, source main ${source_main:0:12}"
