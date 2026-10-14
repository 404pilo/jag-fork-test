#!/usr/bin/env bash
# private-data issues scan: find keys and private words in the issues of a repository.
# The spec is docs/superpowers/specs/2026-10-14-STORY-47-go-public-with-a-clean-history-design.md.
#
# The script reads each issue (no pull request): the title, each old title, the
# body, each comment, and each revision of the body and of each comment. It also
# reads each label and each milestone. It writes the text and an index of the
# places, then runs check.sh in the text mode.
#
# Input (environment):
#   REPO              owner/name of the repository to read.
#   STATE             "open" or "all". Default: all.
#   GH_TOKEN          the token that gh uses (issues: read).
#   PRIVATE_PATTERNS, GITLEAKS  as for check.sh.
#   ISSUES_JSON       for the tests: a file with {"issues": [], "labels": [], "milestones": []}.
#                     The script then reads no network.
#
# Exit codes: as check.sh. This script never prints the text of an issue.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO="${REPO:-}"
STATE="${STATE:-all}"
ISSUES_JSON="${ISSUES_JSON:-}"

fail() { # exit-code message
  local code=$1
  shift
  echo "private-data: $*" >&2
  exit "$code"
}

case $STATE in
  open | all) ;;
  *) fail 2 "STATE must be open or all" ;;
esac
if [[ -z $ISSUES_JSON && ! $REPO =~ ^[A-Za-z0-9-]+/[A-Za-z0-9._-]+$ ]]; then
  fail 2 "REPO must be owner/name"
fi

TMP=$(mktemp -d)
chmod 700 "$TMP"
trap 'rm -rf "$TMP"' EXIT

# gh writes its errors to a file, never to the log: an error can hold a value.
gh_quiet() { # output-file gh-args...
  local out=$1 rc=0
  shift
  gh "$@" >"$out" 2>"$TMP/gh.err" || rc=$?
  if ((rc != 0)); then
    fail 2 "gh failed (exit $rc)"
  fi
}

QUERY='query($owner: String!, $name: String!, $after: String, $states: [IssueState!]) {
  repository(owner: $owner, name: $name) {
    issues(first: 10, after: $after, states: $states, orderBy: {field: CREATED_AT, direction: ASC}) {
      pageInfo { hasNextPage endCursor }
      nodes {
        number state title body
        userContentEdits(first: 100) { pageInfo { hasNextPage } nodes { diff } }
        timelineItems(first: 100, itemTypes: [RENAMED_TITLE_EVENT]) {
          pageInfo { hasNextPage }
          nodes { ... on RenamedTitleEvent { previousTitle } }
        }
        comments(first: 100) {
          pageInfo { hasNextPage }
          nodes { body userContentEdits(first: 100) { pageInfo { hasNextPage } nodes { diff } } }
        }
      }
    }
  }
}'

# The same data in one document, as ISSUES_JSON has it.
fetch() {
  local owner=${REPO%%/*} name=${REPO#*/} cursor="" args more
  : >"$TMP/issues.ndjson"
  while :; do
    args=(-f "query=$QUERY" -f "owner=$owner" -f "name=$name")
    [[ $STATE == open ]] && args+=(-f 'states[]=OPEN')
    [[ -n $cursor ]] && args+=(-f "after=$cursor")
    gh_quiet "$TMP/page.json" api graphql "${args[@]}"
    if jq -e '.errors' "$TMP/page.json" >/dev/null 2>&1 || ! jq -e '.data.repository.issues' "$TMP/page.json" >/dev/null 2>&1; then
      fail 2 "the issues query failed"
    fi
    # One query reads at most 100 old titles, comments, and revisions of an issue.
    if jq -e '.data.repository.issues.nodes[]
        | select(.userContentEdits.pageInfo.hasNextPage or .timelineItems.pageInfo.hasNextPage
            or .comments.pageInfo.hasNextPage
            or any(.comments.nodes[]; .userContentEdits.pageInfo.hasNextPage))' "$TMP/page.json" >/dev/null 2>&1; then
      number=$(jq -r '[.data.repository.issues.nodes[]
        | select(.userContentEdits.pageInfo.hasNextPage or .timelineItems.pageInfo.hasNextPage
            or .comments.pageInfo.hasNextPage
            or any(.comments.nodes[]; .userContentEdits.pageInfo.hasNextPage))][0].number' "$TMP/page.json")
      fail 2 "issue $number has more than 100 old titles, comments, or revisions; the scan cannot read it"
    fi
    jq -c '.data.repository.issues.nodes[] | {
        number, state: (.state | ascii_downcase), title, body,
        oldTitles: [.timelineItems.nodes[] | .previousTitle],
        bodyRevisions: [.userContentEdits.nodes[] | .diff],
        comments: [.comments.nodes[] | {body, revisions: [.userContentEdits.nodes[] | .diff]}]
      }' "$TMP/page.json" >>"$TMP/issues.ndjson"
    more=$(jq -r '.data.repository.issues.pageInfo.hasNextPage' "$TMP/page.json")
    [[ $more == true ]] || break
    cursor=$(jq -r '.data.repository.issues.pageInfo.endCursor' "$TMP/page.json")
  done
  gh_quiet "$TMP/labels.raw" api --paginate "repos/$REPO/labels?per_page=100"
  gh_quiet "$TMP/milestones.raw" api --paginate "repos/$REPO/milestones?state=all&per_page=100"
  jq -n --slurpfile i "$TMP/issues.ndjson" --slurpfile l "$TMP/labels.raw" --slurpfile m "$TMP/milestones.raw" '
    {issues: $i,
     labels: ($l | add // [] | map({name, description})),
     milestones: ($m | add // [] | map({number, title, description}))}'
}

if [[ -n $ISSUES_JSON ]]; then
  [[ -f $ISSUES_JSON ]] || fail 2 "ISSUES_JSON must name a file"
  cp "$ISSUES_JSON" "$TMP/data.json"
else
  fetch >"$TMP/data.json"
fi

# One record for each line of text: {place, text}. A place holds only numbers.
# shellcheck disable=SC2016
JQ_RECORDS='
  def lines($place): (. // "") | split("\n") | to_entries[]
    | {place: "\($place), line \(.key + 1)", text: .value};
  def one($place): {place: $place, text: ((. // "") | gsub("[\r\n]"; " "))};
  [ .issues[] | select(.pull_request == null) | select($state == "all" or .state == "open")
    | .number as $n
    | (.title | one("issue \($n), title")),
      (.oldTitles // [] | to_entries[] | (.key + 1) as $t | .value | one("issue \($n), old title \($t)")),
      (.body | lines("issue \($n), body")),
      (.bodyRevisions // [] | to_entries[] | (.key + 1) as $r | .value | lines("issue \($n), body revision \($r)")),
      (.comments // [] | to_entries[] | (.key + 1) as $c | .value
        | (.body | lines("issue \($n), comment \($c)")),
          (.revisions // [] | to_entries[] | (.key + 1) as $r | .value | lines("issue \($n), comment \($c) revision \($r)")))
  ]
  + [ (.labels // [] | sort_by(.name) | to_entries[] | (.key + 1) as $i | .value
      | (.name | one("label \($i), name")), (.description | select(. != null) | one("label \($i), description"))) ]
  + [ (.milestones // [] | sort_by(.number)[] | .number as $m
      | (.title | one("milestone \($m), title")), (.description | select(. != null) | lines("milestone \($m), description"))) ]
'

jq -r --arg state "$STATE" "$JQ_RECORDS | .[] | .text" "$TMP/data.json" >"$TMP/text.txt"
jq -r --arg state "$STATE" "$JQ_RECORDS | .[] | .place" "$TMP/data.json" >"$TMP/text.idx"

# Counts only: chores uses them to check that the scan read all the text.
jq -r --arg state "$STATE" '
  [.issues[] | select(.pull_request == null) | select($state == "all" or .state == "open")] as $i
  | "private-data: issues scan: \($i | length) issues, \([$i[] | (.comments // [])[]] | length) comments, \([$i[] | ((.bodyRevisions // []) + ([(.comments // [])[] | (.revisions // [])[]]))[]] | length) revisions"' \
  "$TMP/data.json"

TEXT_FILE="$TMP/text.txt" TEXT_INDEX="$TMP/text.idx" "$HERE/check.sh"
