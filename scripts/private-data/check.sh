#!/usr/bin/env bash
# private-data check: find keys and private words in a repository.
# The spec of the pull request mode is docs/superpowers/specs/2026-10-13-STORY-46-block-private-data-in-pull-requests-design.md.
# The spec of the history mode and the text mode is docs/superpowers/specs/2026-10-14-STORY-47-go-public-with-a-clean-history-design.md.
#
# Input (environment):
#   PRIVATE_PATTERNS  required. One Go (RE2) regular expression on each line.
#   GITLEAKS          required. The path of the gitleaks binary.
#   REPO_DIR          the git folder to scan. Default: .
#   BASE_SHA, HEAD_SHA, PR_TITLE, PR_BODY  optional. They turn on the commit scan and the text scan.
#   FORK_PR           "true" for a pull request from a fork: print no place.
#   HISTORY           "main" or "all". Scan each commit of main, or of each ref (history mode).
#   TEXT_FILE, TEXT_INDEX  Scan a text file. Line N of TEXT_INDEX names the place of line N (text mode).
#   Set only one of BASE_SHA, HISTORY, and TEXT_FILE.
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
HISTORY="${HISTORY:-}"
TEXT_FILE="${TEXT_FILE:-}"
TEXT_INDEX="${TEXT_INDEX:-}"

fail() { # exit-code message
  local code=$1
  shift
  echo "private-data: $*" >&2
  exit "$code"
}

# Step 00: one mode at a time. The tree mode (with or without BASE_SHA) scans
# the work tree. The history mode and the text mode scan no work tree.
modes=0
[[ -n $BASE_SHA ]] && modes=$((modes + 1))
[[ -n $HISTORY ]] && modes=$((modes + 1))
[[ -n $TEXT_FILE || -n $TEXT_INDEX ]] && modes=$((modes + 1))
if ((modes > 1)); then
  fail 2 "set only one of BASE_SHA, HISTORY, and TEXT_FILE"
fi
SCAN_TREE=true
REVS=""
if [[ -n $HISTORY ]]; then
  SCAN_TREE=false
  case $HISTORY in
    main) REVS=refs/heads/main ;;
    all) REVS=--all ;;
    *) fail 2 "HISTORY must be main or all" ;;
  esac
  if [[ $HISTORY == main ]] && ! git -C "$REPO_DIR" rev-parse --verify -q refs/heads/main >/dev/null; then
    fail 2 "the repository has no main branch"
  fi
fi
if [[ -n $TEXT_FILE || -n $TEXT_INDEX ]]; then
  SCAN_TREE=false
  if [[ -z $TEXT_FILE || -z $TEXT_INDEX || ! -f $TEXT_FILE || ! -f $TEXT_INDEX ]]; then
    fail 2 "TEXT_FILE and TEXT_INDEX must name two files"
  fi
  if (($(wc -l <"$TEXT_FILE") != $(wc -l <"$TEXT_INDEX"))); then
    fail 2 "TEXT_INDEX must have as many lines as TEXT_FILE"
  fi
fi

# Step 0: the merge commit of the pull request must be ready.
if [[ $SCAN_TREE == true && -n $HEAD_SHA ]]; then
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
if [[ $SCAN_TREE == true ]]; then
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
fi

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

if [[ $SCAN_TREE == true ]]; then
  scan_both tree /dev/null dir "$TMP/tree"

  git -C "$REPO_DIR" -c core.quotePath=false ls-files >"$TMP/paths.txt"
  scan_both paths "$TMP/paths.txt" stdin

  scan_both links "$TMP/links.txt" stdin
fi

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

# The history mode: each line that a commit adds is in its patch. The other
# text of a commit goes to a file with an index of its places.
: >"$TMP/binlist.tsv"
: >"$TMP/binpaths.txt"
HIST_TEXTS=()
has_nul() { # blob: true when a zero byte is in the first 8192 bytes (the rule of git-filter-repo)
  local count
  count=$(git -C "$REPO_DIR" cat-file blob "$1" | head -c 8192 | tr -d -c '\0' | wc -c) || true
  ((count > 0))
}
if [[ -n $HISTORY ]]; then
  scan_both hist /dev/null git \
    --log-opts="--text --no-textconv --no-ext-diff --diff-merges=first-parent $REVS" \
    "$REPO_DIR"

  # The messages. Index line: <commit> commit message, line <n>
  git -C "$REPO_DIR" log -z --format='%H%n%B' "$REVS" >"$TMP/msg.raw"
  : >"$TMP/messages.txt"
  : >"$TMP/messages.idx"
  while IFS= read -r -d '' rec; do
    sha=${rec%%$'\n'*}
    msg=${rec#*$'\n'}
    n=0
    while IFS= read -r l || [[ -n $l ]]; do
      n=$((n + 1))
      printf '%s\n' "$l" >&3
      printf '%s commit message, line %s\n' "${sha:0:12}" "$n" >&4
    done <<<"$msg"
  done <"$TMP/msg.raw" 3>>"$TMP/messages.txt" 4>>"$TMP/messages.idx"
  scan_both messages "$TMP/messages.txt" stdin
  HIST_TEXTS+=(messages)

  # The people: four lines for each commit.
  git -C "$REPO_DIR" log --format='%H%n%an%n%ae%n%cn%n%ce' "$REVS" >"$TMP/people.raw"
  awk -v t="$TMP/people.txt" -v i="$TMP/people.idx" '
    BEGIN { kinds[2] = "author name"; kinds[3] = "author email"; kinds[4] = "committer name"; kinds[0] = "committer email" }
    NR % 5 == 1 { sha = substr($0, 1, 12); next }
    { print > t; print sha " " kinds[NR % 5] > i }
  ' "$TMP/people.raw"
  : >>"$TMP/people.txt"
  : >>"$TMP/people.idx"
  scan_both people "$TMP/people.txt" stdin
  HIST_TEXTS+=(people)

  # The file names, and the binary files, from the raw diff of each commit.
  git -C "$REPO_DIR" log --raw --no-abbrev --no-renames --diff-merges=first-parent -z --format='%x01%H' "$REVS" >"$TMP/raw.bin"
  : >"$TMP/hnames.txt"
  : >"$TMP/hnames.idx"
  : >"$TMP/bincand.tsv"
  state=meta
  sha=""
  k=0
  newmode=""
  newobj=""
  status=""
  while IFS= read -r -d '' tok; do
    if [[ $state == meta ]]; then
      if [[ $tok == $'\001'* ]]; then
        sha=${tok#$'\001'}
        sha=${sha:0:12}
        k=0
        continue
      fi
      tok=${tok#$'\n'}
      meta=${tok#:}
      read -r _ newmode _ newobj status <<<"$meta"
      state=path
      continue
    fi
    k=$((k + 1))
    if [[ $tok == *[[:cntrl:]]* ]]; then
      fail 1 "commit $sha has a file name with a control character; the check does not accept it"
    fi
    printf '%s\n' "$tok" >>"$TMP/hnames.txt"
    printf '%s file name number %s of the commit\n' "$sha" "$k" >>"$TMP/hnames.idx"
    if [[ $status != D* && $newmode != 160000 ]]; then
      printf '%s\t%s\t%s\n' "$newobj" "$sha" "$tok" >>"$TMP/bincand.tsv"
    fi
    state=meta
  done <"$TMP/raw.bin"
  scan_both hnames "$TMP/hnames.txt" stdin
  HIST_TEXTS+=(hnames)

  declare -A isbin=()
  while IFS=$'\t' read -r obj csha cpath; do
    if [[ -z ${isbin[$obj]:-} ]]; then
      if has_nul "$obj"; then isbin[$obj]=1; else isbin[$obj]=0; fi
    fi
    if [[ ${isbin[$obj]} == 1 ]]; then
      printf '%s\t%s\n' "$csha" "$cpath" >>"$TMP/binlist.tsv"
      printf '%s\n' "$cpath" >>"$TMP/binpaths.txt"
    fi
  done <"$TMP/bincand.tsv"

  # The refs.
  : >"$TMP/refs.txt"
  : >"$TMP/refs.idx"
  if [[ $HISTORY == all ]]; then
    git -C "$REPO_DIR" for-each-ref --format='%(refname)' >"$TMP/refs.txt"
    awk '{ print "ref number " NR " in git for-each-ref order" }' "$TMP/refs.txt" >"$TMP/refs.idx"
  fi
  scan_both refs "$TMP/refs.txt" stdin
  HIST_TEXTS+=(refs)
fi

# The text mode.
if [[ -n $TEXT_FILE ]]; then
  scan_both textmode "$TEXT_FILE" stdin
fi

# Step 8: find the paths that hold private data, so the output does not name them.
report() { # label ; prints the report array, or [] when the file is missing
  if [[ -f $TMP/$1.json ]]; then cat "$TMP/$1.json"; else echo '[]'; fi
}
{
  for label in tree commits hist; do
    for cfg in keys private; do
      report "$label-$cfg" | jq -r --arg pre "$TMP/tree/" '.[]? | .File | ltrimstr($pre)'
    done
  done
  cat "$TMP/binpaths.txt"
} | sort -u >"$TMP/finding-paths.txt"
: >"$TMP/hidden.txt"
if [[ -s $TMP/finding-paths.txt ]]; then
  scan_both hidden "$TMP/finding-paths.txt" stdin
  for cfg in keys private; do
    report "hidden-$cfg" | jq -r '.[]? | .StartLine' | while read -r ln; do
      sed -n "${ln}p" "$TMP/finding-paths.txt" >>"$TMP/hidden.txt"
    done
  done
fi

indexed() { # label index-file ; prints "<place>: <RuleID>" for each finding
  report "$1" | jq -r '.[]? | "\(.StartLine)\t\(.RuleID)"' | while IFS=$'\t' read -r ln rule; do
    printf '%s: %s\n' "$(sed -n "${ln}p" "$2")" "$rule"
  done
}

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
  if [[ -n $HISTORY ]]; then
    report "hist-$cfg" | jq -r --rawfile hid "$TMP/hidden.txt" '
      ($hid | split("\n")) as $h
      | .[]? | .File as $p
      | "\(.Commit[0:12]) \(if any($h[]; . == $p) then "a file whose path holds private data" else $p end):\(.StartLine): \(.RuleID)"' >>"$TMP/out.txt"
    indexed "messages-$cfg" "$TMP/messages.idx" >>"$TMP/out.txt"
    indexed "people-$cfg" "$TMP/people.idx" >>"$TMP/out.txt"
    indexed "hnames-$cfg" "$TMP/hnames.idx" >>"$TMP/out.txt"
    indexed "refs-$cfg" "$TMP/refs.idx" >>"$TMP/out.txt"
  fi
  if [[ -n $TEXT_FILE ]]; then
    indexed "textmode-$cfg" "$TEXT_INDEX" >>"$TMP/out.txt"
  fi
done

# Step 9: the result. A binary file is not a finding (history mode).
count=$(wc -l <"$TMP/out.txt")
bincount=$(wc -l <"$TMP/binlist.tsv")
print_binaries() {
  ((bincount > 0)) || return 0
  echo "private-data: binary files: $bincount"
  while IFS=$'\t' read -r csha cpath; do
    if grep -qxF -- "$cpath" "$TMP/hidden.txt"; then cpath="a file whose path holds private data"; fi
    printf '%s %s\n' "$csha" "$cpath"
  done <"$TMP/binlist.tsv"
}
if ((count == 0)); then
  echo "private-data: no findings"
  print_binaries
  exit 0
fi
if [[ $FORK_PR == true && -z $HISTORY ]]; then
  echo "private-data: private data found"
  exit 1
fi
cat "$TMP/out.txt"
if [[ $SCAN_TREE == true ]]; then
  echo "private-data: $count findings. Remove the private data, then push again."
else
  echo "private-data: $count findings."
fi
print_binaries
exit 1
