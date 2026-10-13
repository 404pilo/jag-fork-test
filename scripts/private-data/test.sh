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

echo
echo "private-data tests: $PASS checks passed, $FAIL failed"
((FAIL == 0))
