#!/usr/bin/env bash
# Tests of check.sh. They use invented patterns and values that this script
# builds at run time. They need no secret.
#
# Run: GITLEAKS=/path/to/gitleaks scripts/private-data/test.sh
set -uo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CHECK="$HERE/check.sh"
GITLEAKS="${GITLEAKS:-$(command -v gitleaks || true)}"
if [[ -z $GITLEAKS ]]; then
  echo "test.sh: set GITLEAKS to the gitleaks binary" >&2
  exit 2
fi
export GITLEAKS

T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
SCRATCH="$T/scratch" # the check runs from here, not from the repository
mkdir "$SCRATCH"

rand_alnum() { LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c "$1" || true; }

WORD="jagtest-private-word-$(printf '%04d' $((RANDOM % 10000)))"
KEY="ghp_$(rand_alnum 36)"
export PRIVATE_PATTERNS='jagtest-private-word-[0-9]{4}'

PASS=0
FAIL=0
CUR=""
OUT=""
RC=0
REPO=""

ok() { PASS=$((PASS + 1)); }
bad() { # message
  FAIL=$((FAIL + 1))
  echo "FAIL [$CUR]: $1"
  echo "  exit code: $RC"
  echo "  output:"
  sed 's/^/    /' <<<"$OUT"
}
assert_rc() { if ((RC == $1)); then ok; else bad "want exit $1"; fi; }
assert_has() { if grep -qF -- "$1" <<<"$OUT"; then ok; else bad "output lacks: $1"; fi; }
assert_lacks() { if grep -qF -- "$1" <<<"$OUT"; then bad "output holds a value it must hide"; else ok; fi; }
assert_lines() { if (($(wc -l <<<"$OUT") == $1)); then ok; else bad "want $1 output lines"; fi; }
# CRITERION-75: the output holds no private word, no key, and no pattern text.
assert_hides() {
  assert_lacks "$WORD"
  assert_lacks "${WORD^^}"
  assert_lacks "$KEY"
  assert_lacks "jagtest-private"
}

# new_repo: make a git repository with one clean commit on main.
new_repo() {
  REPO="$T/repo-$1"
  git init -q -b main "$REPO"
  git -C "$REPO" config user.email t@example.invalid
  git -C "$REPO" config user.name test
  git -C "$REPO" config commit.gpgsign false
  echo clean >"$REPO/readme.txt"
  git -C "$REPO" add -A
  git -C "$REPO" commit -q -m base
}
add_commit() { # message ; commits all changes of REPO
  git -C "$REPO" add -A
  git -C "$REPO" commit -q --allow-empty -m "$1"
}
# run_check [VAR=value ...]: run check.sh from another folder.
run_check() {
  OUT=$(cd "$SCRATCH" && env REPO_DIR="$REPO" "$@" "$CHECK" 2>&1)
  RC=$?
}

# PR repositories: pr_start makes a base commit and the branch "pr".
# pr_finish makes the merge commit on main, as GitHub does, and sets
# BASE and HEAD for run_check.
BASE=""
HEAD=""
pr_start() {
  new_repo "$1"
  BASE=$(git -C "$REPO" rev-parse HEAD)
  git -C "$REPO" checkout -q -b pr
}
pr_finish() {
  HEAD=$(git -C "$REPO" rev-parse HEAD)
  git -C "$REPO" checkout -q main
  git -C "$REPO" merge -q --no-ff pr -m "merge of the pull request"
}
run_pr() { run_check BASE_SHA="$BASE" HEAD_SHA="$HEAD" PR_TITLE=title PR_BODY=body "$@"; }

# ---- CRITERION-74 and CRITERION-75 ----

CUR="a: file with a private word"
new_repo a
echo "text $WORD text" >"$REPO/a1.txt"
add_commit x
run_check
assert_rc 1
assert_has "a1.txt:1: private-pattern-1"
assert_has "private-data: 1 findings."
assert_hides

CUR="b: key-shaped value"
new_repo b
echo "token = $KEY" >"$REPO/b1.txt"
add_commit x
run_check
assert_rc 1
assert_has "b1.txt:1: "
assert_hides

CUR="e: private word in other case"
new_repo e
echo "${WORD^^}" >"$REPO/e1.txt"
add_commit x
run_check
assert_rc 1
assert_has "e1.txt:1: private-pattern-1"
assert_hides

CUR="f: clean change"
new_repo f
echo "nothing here" >"$REPO/f1.txt"
add_commit x
run_check
assert_rc 0
assert_has "private-data: no findings"

CUR="g: no PRIVATE_PATTERNS"
new_repo g
run_check PRIVATE_PATTERNS=
assert_rc 2
assert_has "PRIVATE_PATTERNS secret is not set"
run_check PRIVATE_PATTERNS=$'# only a comment\n\n   \n'
assert_rc 2

CUR="h: gitleaks:allow does not hide a finding"
new_repo h
echo "$WORD gitleaks:allow" >"$REPO/a.txt"
add_commit x
run_check
assert_rc 1
assert_has "a.txt:1: private-pattern-1"

CUR="i: .gitleaksignore in the tree"
new_repo i
rm "$REPO/readme.txt"
echo "anything" >"$REPO/.gitleaksignore"
echo ok >"$REPO/b.txt"
add_commit x
run_check
assert_rc 1
assert_has "remove the ignore file at file path number 1"
assert_lacks ".gitleaksignore"

CUR="i2: .gitleaksignore as a symbolic link"
new_repo i2
ln -s readme.txt "$REPO/.gitleaksignore"
add_commit x
run_check
assert_rc 1
assert_has "remove the ignore file at file path number 1"

CUR="i3: .gitleaksignore in a folder"
new_repo i3
mkdir "$REPO/sub"
echo anything >"$REPO/sub/.gitleaksignore"
add_commit x
run_check
assert_rc 1
assert_has "remove the ignore file"

CUR="i4: control character in a path"
new_repo i4
printf 'x\n' >"$REPO/line1"$'\n'"$WORD"
add_commit x
run_check
assert_rc 1
assert_has "control character in a path"
assert_hides

CUR="j: svg and lock file"
new_repo j
echo "$WORD" >"$REPO/b.svg"
echo "$WORD" >"$REPO/package-lock.json"
add_commit x
run_check
assert_rc 1
assert_has "b.svg:1: private-pattern-1"
assert_has "package-lock.json:1: private-pattern-1"
assert_has "private-data: 2 findings."

CUR="k: private word only in a file name"
new_repo k
rm "$REPO/readme.txt"
echo clean >"$REPO/$WORD.txt"
add_commit x
run_check
assert_rc 1
assert_has "file path number 1 in git ls-files order: private-pattern-1"
assert_lacks "$WORD"

CUR="l: export-ignore does not hide a file"
new_repo l
echo "c.txt export-ignore" >"$REPO/.gitattributes"
echo "$WORD" >"$REPO/c.txt"
add_commit x
run_check
assert_rc 1
assert_has "c.txt:1: private-pattern-1"

CUR="m: private word in a file name and in its content"
new_repo m
rm "$REPO/readme.txt"
echo "$WORD" >"$REPO/$WORD.txt"
add_commit x
run_check
assert_rc 1
assert_has "a file whose path holds private data:1: private-pattern-1"
assert_lacks "$WORD"

CUR="n: symbolic link with a private target"
new_repo n
ln -s "$WORD" "$REPO/link1"
add_commit x
run_check
assert_rc 1
assert_has "link number 1 in git ls-files -s order: private-pattern-1"
assert_hides

CUR="p: submodule entry"
new_repo p
git -C "$REPO" update-index --add --cacheinfo 160000,"$(git -C "$REPO" rev-parse HEAD)",private-sub-path
git -C "$REPO" commit -q -m x
run_check
assert_rc 1
assert_has "remove the submodule at entry"
assert_lacks "private-sub-path"

CUR="t: working-tree-encoding does not hide a file"
new_repo t
echo "$WORD" >"$REPO/d.txt"
add_commit x
echo "*.txt working-tree-encoding=UTF-16LE" >"$REPO/.gitattributes"
git -C "$REPO" add .gitattributes
git -C "$REPO" commit -q -m attributes
run_check
assert_rc 1
assert_has "d.txt:1: private-pattern-1"

CUR="u: fork pull request prints no place"
new_repo u
echo "$WORD" >"$REPO/a1.txt"
add_commit x
run_check FORK_PR=true
assert_rc 1
assert_lines 1
assert_has "private-data: private data found"

CUR="u2: fork pull request, no finding"
new_repo u2
run_check FORK_PR=true
assert_rc 0

# ---- pattern list rules ----

CUR="pattern with three quotes"
new_repo q3
run_check PRIVATE_PATTERNS="jagtest-private-word-[0-9]{4}'''x"
assert_rc 2
assert_has "pattern line 1 cannot be used"
assert_lacks "jagtest"

CUR="pattern that is not valid RE2"
new_repo q4
run_check PRIVATE_PATTERNS=$'jagtest-private-word-[0-9]{4}\njagtest-word(unclosed'
assert_rc 2
assert_has "pattern line 2 is not a valid RE2 regular expression"
assert_lacks "unclosed"
assert_lacks "jagtest"

CUR="comments, blank lines and CRLF"
new_repo q5
echo "$WORD" >"$REPO/a1.txt"
add_commit x
run_check PRIVATE_PATTERNS=$'# comment\r\n\r\njagtest-private-word-[0-9]{4}   \r\n'
assert_rc 1
assert_has "a1.txt:1: private-pattern-3"

CUR="GITLEAKS missing"
new_repo q6
run_check GITLEAKS=/nonexistent
assert_rc 2

# ---- commit scan and text scan ----

CUR="c: private word only in a removed commit"
pr_start c
echo "$WORD" >"$REPO/c1.txt"
add_commit "add"
git -C "$REPO" rm -q c1.txt
git -C "$REPO" commit -q -m "remove"
pr_finish
run_pr
assert_rc 1
assert_has " c1.txt:1: private-pattern-1"
assert_hides

CUR="c2: clean pull request"
pr_start c2
echo fine >"$REPO/c2.txt"
add_commit "add"
pr_finish
run_pr
assert_rc 0

CUR="d: private word only in the title"
pr_start d
echo fine >"$REPO/d1.txt"
add_commit "add"
pr_finish
run_check BASE_SHA="$BASE" HEAD_SHA="$HEAD" PR_TITLE="fix $WORD" PR_BODY=body
assert_rc 1
assert_has "PR text or commit message, line 1: private-pattern-1"
assert_hides

CUR="d2: private word only in the body"
run_check BASE_SHA="$BASE" HEAD_SHA="$HEAD" PR_TITLE=title PR_BODY="line one
$WORD"
assert_rc 1
assert_has "PR text or commit message, line 4: private-pattern-1"

CUR="d3: private word only in a commit message"
pr_start d3
echo fine >"$REPO/d3.txt"
add_commit "message $WORD"
pr_finish
run_pr
assert_rc 1
assert_has "PR text or commit message, line "
assert_hides

CUR="o: private word added only by a merge commit"
pr_start o
echo one >"$REPO/o1.txt"
add_commit "one"
git -C "$REPO" checkout -q -b side "$BASE"
echo two >"$REPO/o2.txt"
add_commit "two"
git -C "$REPO" checkout -q pr
git -C "$REPO" merge -q --no-commit --no-ff side >/dev/null
echo "$WORD" >"$REPO/o3.txt"
git -C "$REPO" add -A
git -C "$REPO" commit -q -m "merge side"
git -C "$REPO" rm -q o3.txt
git -C "$REPO" commit -q -m "remove"
pr_finish
run_pr
assert_rc 1
assert_has " o3.txt:1: private-pattern-1"

CUR="s: '* -diff' does not hide a commit"
pr_start s
echo "* -diff" >"$REPO/.gitattributes"
echo "$WORD" >"$REPO/s1.txt"
add_commit "add"
git -C "$REPO" rm -q s1.txt
git -C "$REPO" commit -q -m "remove"
pr_finish
run_pr
assert_rc 1
assert_has " s1.txt:1: private-pattern-1"

CUR="q: HEAD_SHA is not the second parent"
pr_start q
echo fine >"$REPO/q1.txt"
add_commit "add"
pr_finish
run_check BASE_SHA="$BASE" HEAD_SHA="$BASE" PR_TITLE=t PR_BODY=b
assert_rc 2
assert_has "merge commit of the pull request is not ready"

CUR="name added and then renamed"
pr_start nm
echo fine >"$REPO/$WORD.txt"
add_commit "add"
git -C "$REPO" mv "$WORD.txt" ok.txt
git -C "$REPO" commit -q -m "rename"
pr_finish
run_pr
assert_rc 1
assert_has "in git log --name-only order: private-pattern-1"
assert_hides

CUR="fork pull request with a commit finding"
pr_start fk
echo "$WORD" >"$REPO/fk.txt"
add_commit "add"
pr_finish
run_pr FORK_PR=true
assert_rc 1
assert_lines 1

# ---- history mode (STORY-47) ----

short() { git -C "$REPO" rev-parse "$1" | cut -c1-12; }
# run_hist [VAR=value ...]: scan a mirror clone of REPO. HISTORY is main unless a case says another value.
MIRRORS=0
run_hist() {
  MIRRORS=$((MIRRORS + 1))
  local mirror="$T/mirror-$MIRRORS"
  git clone -q --mirror "$REPO" "$mirror"
  run_check REPO_DIR="$mirror" HISTORY=main "$@"
}

CUR="hist-a: private word only in an old commit"
new_repo hist-a
printf 'x\n%s\n' "$WORD" >"$REPO/a.txt"
add_commit "add"
C1=$(short HEAD)
git -C "$REPO" rm -q a.txt
add_commit "remove"
run_hist
assert_rc 1
assert_has "$C1 a.txt:2: private-pattern-1"
assert_has "private-data: 1 findings."
assert_hides

CUR="hist-b: private word in a commit message"
new_repo hist-b
echo one >"$REPO/b.txt"
git -C "$REPO" add -A
git -C "$REPO" commit -q -m "title" -m "$WORD"
C1=$(short HEAD)
run_hist
assert_rc 1
assert_has "$C1 commit message, line 3: private-pattern-1"
assert_hides

CUR="hist-c: private word in a name and in an address"
new_repo hist-c
echo one >"$REPO/c.txt"
git -C "$REPO" add -A
GIT_AUTHOR_NAME="$WORD" git -C "$REPO" commit -q -m one
C1=$(short HEAD)
echo two >"$REPO/c.txt"
git -C "$REPO" add -A
GIT_COMMITTER_EMAIL="$WORD@example.invalid" git -C "$REPO" commit -q -m two
C2=$(short HEAD)
run_hist
assert_rc 1
assert_has "$C1 author name: private-pattern-1"
assert_has "$C2 committer email: private-pattern-1"
assert_hides

CUR="hist-d: private word in a file name, then a rename"
new_repo hist-d
echo "$WORD" >"$REPO/$WORD.txt"
add_commit "add"
C1=$(short HEAD)
git -C "$REPO" mv "$WORD.txt" ok.txt
add_commit "rename"
run_hist
assert_rc 1
assert_has "$C1 file name number 1 of the commit: private-pattern-1"
assert_has "$C1 a file whose path holds private data:1: private-pattern-1"
assert_hides

CUR="hist-e: key-shaped value in an old commit"
new_repo hist-e
echo "token = $KEY" >"$REPO/e.txt"
add_commit "add"
C1=$(short HEAD)
git -C "$REPO" rm -q e.txt
add_commit "remove"
run_hist
assert_rc 1
assert_has "$C1 e.txt:1: "
assert_hides

CUR="hist-f: private word only in another branch"
new_repo hist-f
git -C "$REPO" checkout -q -b other
echo "$WORD" >"$REPO/f.txt"
add_commit "add"
git -C "$REPO" checkout -q main
run_hist
assert_rc 0
run_hist HISTORY=all
assert_rc 1
assert_has " f.txt:1: private-pattern-1"
assert_hides

CUR="hist-g: private word only in a pull request ref"
new_repo hist-g
git -C "$REPO" checkout -q -b tmp
echo "$WORD" >"$REPO/g.txt"
add_commit "add"
git -C "$REPO" update-ref refs/pull/1/head tmp
git -C "$REPO" checkout -q main
git -C "$REPO" branch -q -D tmp
run_hist
assert_rc 0
run_hist HISTORY=all
assert_rc 1
assert_has " g.txt:1: private-pattern-1"

CUR="hist-h: private word in a branch name"
new_repo hist-h
git -C "$REPO" branch "$WORD"
run_hist
assert_rc 0
run_hist HISTORY=all
assert_rc 1
assert_has "in git for-each-ref order: private-pattern-1"
assert_hides

CUR="hist-i: binary file in an old commit"
new_repo hist-i
printf 'a\0b' >"$REPO/bin.dat"
add_commit "add"
C1=$(short HEAD)
run_hist
assert_rc 0
assert_has "private-data: binary files: 1"
assert_has "$C1 bin.dat"
printf 'a\0b' >"$REPO/$WORD.dat"
add_commit "add 2"
C2=$(short HEAD)
run_hist
assert_rc 1
assert_has "private-data: binary files: 2"
assert_has "$C2 a file whose path holds private data"
assert_hides

CUR="hist-j: '* -diff' does not hide a commit"
new_repo hist-j
echo "* -diff" >"$REPO/.gitattributes"
add_commit "attributes"
echo "$WORD" >"$REPO/j.txt"
add_commit "add"
git -C "$REPO" rm -q j.txt
add_commit "remove"
run_hist
assert_rc 1
assert_has " j.txt:1: private-pattern-1"
assert_lacks "binary files"

CUR="hist-k: private word added only by a merge commit"
new_repo hist-k
BASE=$(git -C "$REPO" rev-parse HEAD)
git -C "$REPO" checkout -q -b side
echo two >"$REPO/k2.txt"
add_commit "two"
git -C "$REPO" checkout -q main
echo one >"$REPO/k1.txt"
add_commit "one"
git -C "$REPO" merge -q --no-commit --no-ff side >/dev/null
echo "$WORD" >"$REPO/k3.txt"
git -C "$REPO" add -A
git -C "$REPO" commit -q -m "merge side"
git -C "$REPO" rm -q k3.txt
add_commit "remove"
run_hist
assert_rc 1
assert_has " k3.txt:1: private-pattern-1"

CUR="hist-l: two modes, and a wrong value"
new_repo hist-l
run_hist BASE_SHA=abc
assert_rc 2
run_hist HISTORY=other
assert_rc 2
assert_has "HISTORY must be main or all"

CUR="hist-m: a clone with a work tree gives the same output"
new_repo hist-m
printf 'x\n%s\n' "$WORD" >"$REPO/a.txt"
add_commit "add"
C1=$(short HEAD)
git -C "$REPO" rm -q a.txt
add_commit "remove"
git clone -q "$REPO" "$T/wt-m"
run_check REPO_DIR="$T/wt-m" HISTORY=main
assert_rc 1
assert_has "$C1 a.txt:2: private-pattern-1"
assert_hides

CUR="hist-n: a history with no private data"
new_repo hist-n
echo fine >"$REPO/n.txt"
add_commit "add"
run_hist
assert_rc 0
assert_has "private-data: no findings"

# ---- text mode and issues scan (STORY-61) ----

ISSUES="$HERE/issues.sh"
run_issues() { # [VAR=value ...]
  OUT=$(cd "$SCRATCH" && env "$@" "$ISSUES" 2>&1)
  RC=$?
}
# issue_json number state title body old-titles-json revisions-json comments-json
issue_json() {
  jq -n --argjson n "$1" --arg s "$2" --arg t "$3" --arg b "$4" \
    --argjson o "$5" --argjson r "$6" --argjson c "$7" \
    '{number: $n, state: $s, title: $t, body: $b, oldTitles: $o, bodyRevisions: $r, comments: $c}'
}
# write_issues file issue-json... (labels and milestones: LABELS and MILESTONES)
LABELS='[]'
MILESTONES='[]'
write_issues() {
  local file=$1
  shift
  printf '%s\n' "$@" | jq -s --argjson l "$LABELS" --argjson m "$MILESTONES" \
    '{issues: ., labels: $l, milestones: $m}' >"$file"
}

CUR="text-a: a text file with an index"
printf 'one\ntwo\n%s\nfour\n' "$WORD" >"$T/text-a.txt"
printf 'issue 7, title\nissue 7, body, line 1\nissue 7, comment 2, line 1\nissue 7, comment 3, line 1\n' >"$T/text-a.idx"
run_check TEXT_FILE="$T/text-a.txt" TEXT_INDEX="$T/text-a.idx"
assert_rc 1
assert_has "issue 7, comment 2, line 1: private-pattern-1"
assert_hides

CUR="text-b: wrong index"
printf 'one\ntwo\n' >"$T/text-b.txt"
printf 'a\n' >"$T/text-b.idx"
run_check TEXT_FILE="$T/text-b.txt" TEXT_INDEX="$T/text-b.idx"
assert_rc 2
run_check TEXT_FILE="$T/text-b.txt"
assert_rc 2
run_check TEXT_FILE="$T/text-b.txt" TEXT_INDEX="$T/text-b.idx" HISTORY=main
assert_rc 2

CUR="text-c: places of the issues scan"
LABELS=$(jq -n --arg w "$WORD" '[{name: "a-label", description: "x"}, {name: "b-label", description: $w}]')
MILESTONES=$(jq -n --arg w "$WORD" '[{number: 3, title: $w, description: null}]')
I4=$(issue_json 4 open "title four" "body four" '[]' '[]' "$(jq -n --arg w "$WORD" '[{body: "c", revisions: [$w]}]')")
I5=$(issue_json 5 open "title five" "body five" "$(jq -n --arg w "$WORD" '[$w]')" '[]' '[]')
write_issues "$T/text-c.json" "$I4" "$I5"
run_issues ISSUES_JSON="$T/text-c.json" STATE=all
assert_rc 1
assert_has "issue 4, comment 1 revision 1, line 1: private-pattern-1"
assert_has "issue 5, old title 1: private-pattern-1"
assert_has "label 2, description: private-pattern-1"
assert_has "milestone 3, title: private-pattern-1"
assert_has "private-data: 4 findings."
assert_hides
LABELS='[]'
MILESTONES='[]'

CUR="text-c2: places in a body and in a body revision"
I8=$(issue_json 8 open "t" "$(printf 'one\n%s' "$WORD")" '[]' "$(jq -n --arg w "$WORD" '["a", $w]')" '[]')
write_issues "$T/text-c2.json" "$I8"
run_issues ISSUES_JSON="$T/text-c2.json"
assert_rc 1
assert_has "issue 8, body, line 2: private-pattern-1"
assert_has "issue 8, body revision 2, line 1: private-pattern-1"
assert_hides

CUR="text-d: a pull request is not read"
PR=$(jq -n --arg w "$WORD" '{number: 9, state: "open", title: $w, body: $w, pull_request: {}}')
write_issues "$T/text-d.json" "$PR"
run_issues ISSUES_JSON="$T/text-d.json"
assert_rc 0

CUR="text-e: issues with no private data"
write_issues "$T/text-e.json" "$(issue_json 1 open "t" "b" '[]' '[]' '[{"body": "c", "revisions": ["r"]}]')"
run_issues ISSUES_JSON="$T/text-e.json"
assert_rc 0
assert_has "private-data: no findings"
assert_has "private-data: issues scan: 1 issues, 1 comments, 1 revisions"

CUR="text-f: a closed issue"
I6=$(issue_json 6 closed "t" "b" '[]' '[]' "$(jq -n --arg w "$WORD" '[{body: $w, revisions: []}]')")
write_issues "$T/text-f.json" "$I6"
run_issues ISSUES_JSON="$T/text-f.json" STATE=all
assert_rc 1
assert_has "issue 6, comment 1, line 1: private-pattern-1"
run_issues ISSUES_JSON="$T/text-f.json" STATE=open
assert_rc 0
run_issues ISSUES_JSON="$T/text-f.json" STATE=other
assert_rc 2

# ---- the clean step (STORY-52) ----

CLEAN="$HERE/clean-history.sh"
CLEANS=0
OUTD=""
# run_clean [VAR=value ...]: run clean-history.sh on REPO. OUTD is the folder of the bundle.
run_clean() {
  CLEANS=$((CLEANS + 1))
  OUTD="$T/out-$CLEANS"
  OUT=$(cd "$SCRATCH" && env REPO_DIR="$REPO" OLD_REPO=ai-ready-future/test OUT_DIR="$OUTD" \
    FILTER_REPO="$FILTER_REPO" "$@" "$CLEAN" 2>&1)
  RC=$?
}
# bundle_clone name: a clone of the bundle of the last run, in CLONE.
CLONE=""
bundle_clone() {
  CLONE="$T/clone-$1"
  git clone -q --branch main "$OUTD/clean.bundle" "$CLONE"
}
assert_true() { # description command...
  local what=$1
  shift
  if "$@"; then ok; else bad "$what"; fi
}
no_bundle() { [[ ! -e $OUTD/clean.bundle ]]; }

clean_cases() {
  CUR="clean-a: private word in a line, a message, and a file name; two .pyc files"
  new_repo clean-a
  printf 'x\n%s\n' "$WORD" >"$REPO/a.txt"
  echo old >"$REPO/old-$WORD.txt"
  mkdir "$REPO/p"
  printf 'a\0b' >"$REPO/p/x.pyc"
  printf 'a\0c' >"$REPO/y.pyc"
  git -C "$REPO" add -A
  git -C "$REPO" commit -q -m "add $WORD"
  git -C "$REPO" rm -q -r a.txt "old-$WORD.txt" p y.pyc
  add_commit "remove"
  run_clean
  assert_rc 0
  assert_has "private-data: source main "
  assert_has "a.txt:2: private-pattern-1"
  assert_has "private-data: clean"
  assert_has "private-data: no findings"
  assert_has "private-data: clean history ready: 3 commits"
  assert_hides
  assert_true "the bundle exists" test -f "$OUTD/clean.bundle"
  bundle_clone clean-a
  run_check REPO_DIR="$CLONE" HISTORY=main
  assert_rc 0
  assert_lacks "binary files"
  first=$(git -C "$CLONE" log --diff-filter=A --format=%H -- a.txt | tail -n 1)
  assert_true "the old line holds REDACTED" grep -q REDACTED < <(git -C "$CLONE" show "$first:a.txt")
  assert_true "the old name has REDACTED" grep -qx 'old-REDACTED.txt' < <(git -C "$CLONE" log --name-only --format=)
  assert_true "no .pyc path" test "$(git -C "$CLONE" log --name-only --format= | grep -c '\.pyc$' || true)" = 0
  assert_true "the last tree is the same" test "$(git -C "$CLONE" rev-parse 'main^{tree}')" = "$(git -C "$REPO" rev-parse 'main^{tree}')"

  CUR="clean-b: #N in a message"
  new_repo clean-b
  echo one >"$REPO/b.txt"
  git -C "$REPO" add -A
  git -C "$REPO" commit -q -m "fix thing (#12)" -m "Refs #5 #6 and &#39; and a#1"
  run_clean
  assert_rc 0
  bundle_clone clean-b
  msgs=$(git -C "$CLONE" log --format=%B)
  assert_true "title" grep -qF '(ai-ready-future/test#12)' <<<"$msgs"
  assert_true "body" grep -qF 'Refs ai-ready-future/test#5 ai-ready-future/test#6' <<<"$msgs"
  assert_true "entity and a#1 stay" grep -qF "&#39; and a#1" <<<"$msgs"

  CUR="clean-c: a hash in a message"
  new_repo clean-c
  echo "$WORD" >"$REPO/c.txt"
  add_commit "add"
  C1=$(git -C "$REPO" rev-parse HEAD)
  git -C "$REPO" rm -q c.txt
  add_commit "undo ${C1:0:10}"
  run_clean
  assert_rc 0
  bundle_clone clean-c
  new_c1=$(git -C "$CLONE" log --diff-filter=A --format=%H -- c.txt)
  assert_true "the message names the new commit" grep -qF "undo ${new_c1:0:10}" < <(git -C "$CLONE" log --format=%B)
  assert_true "the old hash is gone" test "${new_c1:0:10}" != "${C1:0:10}"

  CUR="clean-d: the mailmap"
  new_repo clean-d
  echo "$WORD" >"$REPO/d.txt"
  add_commit "add"
  git -C "$REPO" rm -q d.txt
  add_commit "remove"
  run_clean CLEAN_MAILMAP='<new@example.invalid> <t@example.invalid>'
  assert_rc 0
  bundle_clone clean-d-1
  assert_true "each address changes" test "$(git -C "$CLONE" log --format='%ae %ce' | sort -u)" = "new@example.invalid new@example.invalid"
  run_clean
  assert_rc 0
  bundle_clone clean-d-2
  assert_true "each address stays" test "$(git -C "$CLONE" log --format='%ae %ce' | sort -u)" = "t@example.invalid t@example.invalid"

  CUR="clean-e: the source stays as it is"
  new_repo clean-e
  echo "$WORD" >"$REPO/e.txt"
  add_commit "add"
  git -C "$REPO" rm -q e.txt
  add_commit "remove"
  git -C "$REPO" branch other
  refs_before=$(git -C "$REPO" for-each-ref)
  run_clean
  assert_rc 0
  assert_true "the refs are the same" test "$(git -C "$REPO" for-each-ref)" = "$refs_before"

  CUR="clean-f: a key with no pattern"
  new_repo clean-f
  echo "token = $KEY" >"$REPO/f.txt"
  add_commit "add"
  git -C "$REPO" rm -q f.txt
  add_commit "remove"
  run_clean
  assert_rc 1
  assert_has "findings remain after the clean"
  assert_has " f.txt:1: "
  assert_hides
  assert_true "no bundle" no_bundle

  CUR="clean-g: a pattern line with ==>"
  new_repo clean-g
  run_clean PRIVATE_PATTERNS='abc==>def'
  assert_rc 2
  assert_has "pattern line 1 cannot be used"
  assert_lacks "abc"

  CUR="clean-h: a pattern that Python does not accept"
  new_repo clean-h
  run_clean PRIVATE_PATTERNS='jagtest-\pL+'
  assert_rc 2
  assert_has "pattern line 1 is not a valid Python regular expression"
  assert_lacks "jagtest"

  CUR="clean-i: the last tree holds the word"
  new_repo clean-i
  echo "$WORD" >"$REPO/i.txt"
  add_commit "add"
  run_clean
  assert_rc 1
  assert_has "the clean changed the last tree of main"
  assert_hides
  assert_true "no bundle" no_bundle

  CUR="clean-j: a binary file that is not a .pyc file"
  new_repo clean-j
  printf 'a\0b' >"$REPO/j.dat"
  add_commit "add"
  git -C "$REPO" rm -q j.dat
  add_commit "remove"
  run_clean
  assert_rc 1
  assert_has "findings remain after the clean"
  assert_has "private-data: binary files: 1"
  assert_true "no bundle" no_bundle

  CUR="clean-k: two runs give the same history"
  new_repo clean-k
  echo "$WORD" >"$REPO/k.txt"
  add_commit "add"
  git -C "$REPO" rm -q k.txt
  add_commit "remove"
  run_clean
  assert_rc 0
  first_run=$(git bundle list-heads "$OUTD/clean.bundle")
  run_clean
  assert_rc 0
  assert_true "the same main commit" test "$(git bundle list-heads "$OUTD/clean.bundle")" = "$first_run"
}

if [[ -n ${FILTER_REPO:-} ]]; then
  clean_cases
else
  echo "test.sh: FILTER_REPO is not set. The clean cases are skipped."
fi

echo
echo "private-data tests: $PASS checks passed, $FAIL failed"
((FAIL == 0))
