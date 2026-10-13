#!/usr/bin/env bash
# private-data check: find keys and private words in a repository.
# The spec is docs/superpowers/specs/2026-10-13-STORY-46-block-private-data-in-pull-requests-design.md.
#
# Input (environment):
#   PRIVATE_PATTERNS  required. One Go (RE2) regular expression on each line.
#   GITLEAKS          required. The path of the gitleaks binary.
#   REPO_DIR          the git folder to scan. Default: .
#   BASE_SHA, HEAD_SHA, PR_TITLE, PR_BODY  optional. They turn on the commit scan and the text scan.
#   FORK_PR           "true" for a pull request from a fork: print no place.
#
# Exit codes: 0 no findings, 1 findings, 2 the check cannot run.
#
# This script never uses "set -x" and never prints a pattern, a found value,
# or the output of gitleaks. It never runs or sources a file of REPO_DIR.
set -euo pipefail

REPO_DIR="${REPO_DIR:-.}"
BASE_SHA="${BASE_SHA:-}"
HEAD_SHA="${HEAD_SHA:-}"
PR_TITLE="${PR_TITLE:-}"
PR_BODY="${PR_BODY:-}"
FORK_PR="${FORK_PR:-false}"
PRIVATE_PATTERNS="${PRIVATE_PATTERNS:-}"
GITLEAKS="${GITLEAKS:-}"

fail() { # exit-code message
  local code=$1
  shift
  echo "private-data: $*" >&2
  exit "$code"
}

# Step 0: the merge commit of the pull request must be ready.
if [[ -n $HEAD_SHA ]]; then
  second_parent=$(git -C "$REPO_DIR" rev-parse --verify -q 'HEAD^2' 2>/dev/null || true)
  if [[ $second_parent != "$HEAD_SHA" ]]; then
    fail 2 "the merge commit of the pull request is not ready; run the job again"
  fi
fi

# Step 1: the pattern list.
patterns=() # patterns[i] holds the pattern of list line numbers[i]
numbers=()
lineno=0
while IFS= read -r line || [[ -n $line ]]; do
  lineno=$((lineno + 1))
  line="${line%"${line##*[![:space:]]}"}" # remove spaces, tabs and \r at the end
  [[ -z $line || $line == '#'* ]] && continue
  patterns+=("$line")
  numbers+=("$lineno")
done <<<"$PRIVATE_PATTERNS"
if ((${#patterns[@]} == 0)); then
  fail 2 "the PRIVATE_PATTERNS secret is not set. Ask the owner to set it."
fi
if [[ -z $GITLEAKS || ! -x $GITLEAKS ]]; then
  fail 2 "GITLEAKS must name the gitleaks binary"
fi

# Step 2: a private temporary folder.
TMP=$(mktemp -d)
chmod 700 "$TMP"
trap 'rm -rf "$TMP"' EXIT
mkdir "$TMP/no-ignore"

# Step 3: the two configs.
printf '[extend]\nuseDefault = true\n' >"$TMP/keys.toml"
: >"$TMP/private.toml"
write_rule() { # file number pattern
  printf '[[rules]]\nid = "private-pattern-%s"\ndescription = "private pattern %s"\nregex = '"'''"'(?i)%s'"'''"'\n\n' \
    "$2" "$2" "$3" >>"$1"
}
for i in "${!patterns[@]}"; do
  n=${numbers[$i]}
  if [[ ${patterns[$i]} == *"'''"* ]]; then
    fail 2 "pattern line $n cannot be used"
  fi
  write_rule "$TMP/private.toml" "$n" "${patterns[$i]}"
done

# Step 4: check each pattern by itself. A bad pattern makes gitleaks write the pattern.
for i in "${!patterns[@]}"; do
  n=${numbers[$i]}
  : >"$TMP/one-$n.toml"
  write_rule "$TMP/one-$n.toml" "$n" "${patterns[$i]}"
  rc=0
  "$GITLEAKS" stdin -c "$TMP/one-$n.toml" --no-banner --log-level error \
    --exit-code 0 </dev/null >"$TMP/pattern-$n.log" 2>&1 || rc=$?
  if ((rc != 0)); then
    fail 2 "pattern line $n is not a valid RE2 regular expression"
  fi
done

# Step 5: copy the stored bytes of the tree. Read the links, refuse submodules and ignore files.
mkdir "$TMP/tree"
: >"$TMP/links.txt"
index=0
while IFS= read -r -d '' entry; do
  index=$((index + 1))
  meta=${entry%%$'\t'*}
  path=${entry#*$'\t'}
  read -r mode object _ <<<"$meta"
  # The name .gitleaksignore is refused for every mode and at every folder level.
  if [[ ${path##*/} == .gitleaksignore ]]; then
    fail 1 "remove the ignore file at file path number $index in git ls-files order; the check does not accept ignore files"
  fi
  if [[ $path == *[[:cntrl:]]* ]]; then
    fail 1 "remove the file at file path number $index in git ls-files order; the check does not accept a control character in a path"
  fi
  case $mode in
    160000)
      fail 1 "remove the submodule at entry $index of git ls-files; the check does not accept submodules"
      ;;
    120000)
      git -C "$REPO_DIR" cat-file blob "$object" >>"$TMP/links.txt"
      printf '\n' >>"$TMP/links.txt"
      ;;
    100644 | 100755)
      if [[ $path == */* ]]; then mkdir -p "$TMP/tree/${path%/*}"; fi
      git -C "$REPO_DIR" cat-file blob "$object" >"$TMP/tree/$path"
      ;;
  esac
done < <(git -C "$REPO_DIR" ls-files -s -z)

# Step 6: the scans. Each scan runs with both configs.
run_gl() { # label config input-file gitleaks-args...
  local label=$1 cfg=$2 input=$3 rc=0
  shift 3
  "$GITLEAKS" "$@" -c "$TMP/$cfg.toml" --redact --no-banner --log-level error \
    --report-format json --report-path "$TMP/$label-$cfg.json" --exit-code 0 \
    --ignore-gitleaks-allow --gitleaks-ignore-path "$TMP/no-ignore" \
    <"$input" >"$TMP/$label-$cfg.log" 2>&1 || rc=$?
  if ((rc != 0)); then # Step 7: never print the log of the run.
    fail 2 "gitleaks failed (exit $rc)"
  fi
}
scan_both() { # label input-file gitleaks-args...
  local label=$1 input=$2
  shift 2
  run_gl "$label" keys "$input" "$@"
  run_gl "$label" private "$input" "$@"
}

scan_both tree /dev/null dir "$TMP/tree"

git -C "$REPO_DIR" -c core.quotePath=false ls-files >"$TMP/paths.txt"
scan_both paths "$TMP/paths.txt" stdin

scan_both links "$TMP/links.txt" stdin

if [[ -n $BASE_SHA ]]; then
  scan_both commits /dev/null git \
    --log-opts="--text --no-textconv --no-ext-diff --diff-merges=first-parent $BASE_SHA..$HEAD_SHA" \
    "$REPO_DIR"
  # The names of the files that the commits touch, one time each, in git log order.
  : >"$TMP/names.txt"
  nindex=0
  while IFS= read -r -d '' name; do
    [[ -z $name ]] && continue
    nindex=$((nindex + 1))
    if [[ $name == *[[:cntrl:]]* ]]; then
      fail 1 "a commit has a file name with a control character, at name number $nindex of git log --name-only; the check does not accept it"
    fi
    printf '%s\n' "$name"
  done < <(git -C "$REPO_DIR" log --name-only --format= -z --no-renames \
    --diff-merges=first-parent "$BASE_SHA..$HEAD_SHA") | awk '!seen[$0]++' >"$TMP/names.txt"
  scan_both names "$TMP/names.txt" stdin
  {
    printf '%s\n\n%s\n\n' "$PR_TITLE" "$PR_BODY"
    git -C "$REPO_DIR" log --format=%B "$BASE_SHA..$HEAD_SHA"
  } >"$TMP/text.txt"
  scan_both text "$TMP/text.txt" stdin
fi

# Step 8: find the paths that hold private data, so the output does not name them.
report() { # label ; prints the report array, or [] when the file is missing
  if [[ -f $TMP/$1.json ]]; then cat "$TMP/$1.json"; else echo '[]'; fi
}
for label in tree commits; do
  for cfg in keys private; do
    report "$label-$cfg" | jq -r --arg pre "$TMP/tree/" '.[]? | .File | ltrimstr($pre)'
  done
done | sort -u >"$TMP/finding-paths.txt"
: >"$TMP/hidden.txt"
if [[ -s $TMP/finding-paths.txt ]]; then
  scan_both hidden "$TMP/finding-paths.txt" stdin
  for cfg in keys private; do
    report "hidden-$cfg" | jq -r '.[]? | .StartLine' | while read -r ln; do
      sed -n "${ln}p" "$TMP/finding-paths.txt" >>"$TMP/hidden.txt"
    done
  done
fi

: >"$TMP/out.txt"
for cfg in keys private; do
  report "tree-$cfg" | jq -r --arg pre "$TMP/tree/" --rawfile hid "$TMP/hidden.txt" '
    ($hid | split("\n")) as $h
    | .[]? | (.File | ltrimstr($pre)) as $p
    | "\(if any($h[]; . == $p) then "a file whose path holds private data" else $p end):\(.StartLine): \(.RuleID)"' >>"$TMP/out.txt"
  report "paths-$cfg" | jq -r '.[]? | "file path number \(.StartLine) in git ls-files order: \(.RuleID)"' >>"$TMP/out.txt"
  report "links-$cfg" | jq -r '.[]? | "link number \(.StartLine) in git ls-files -s order: \(.RuleID)"' >>"$TMP/out.txt"
  if [[ -n $BASE_SHA ]]; then
    report "commits-$cfg" | jq -r --rawfile hid "$TMP/hidden.txt" '
      ($hid | split("\n")) as $h
      | .[]? | .File as $p
      | "\(.Commit[0:12]) \(if any($h[]; . == $p) then "a file whose path holds private data" else $p end):\(.StartLine): \(.RuleID)"' >>"$TMP/out.txt"
    report "names-$cfg" | jq -r '.[]? | "commit file name number \(.StartLine) in git log --name-only order: \(.RuleID)"' >>"$TMP/out.txt"
    report "text-$cfg" | jq -r '.[]? | "PR text or commit message, line \(.StartLine): \(.RuleID)"' >>"$TMP/out.txt"
  fi
done

# Step 9: the result.
count=$(wc -l <"$TMP/out.txt")
if ((count == 0)); then
  echo "private-data: no findings"
  exit 0
fi
if [[ $FORK_PR == true ]]; then
  echo "private-data: private data found"
  exit 1
fi
cat "$TMP/out.txt"
echo "private-data: $count findings. Remove the private data, then push again."
exit 1
