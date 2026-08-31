#!/usr/bin/env bash
set -euo pipefail

repo="${1:-o3kio/araf}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

command -v gh >/dev/null 2>&1 || {
  echo "error: GitHub CLI (gh) is required" >&2
  exit 1
}

echo "Synchronizing Araf planning issues in $repo"
echo "Existing exact-title matches are never recreated."

while IFS=$'\t' read -r code expected_number title body_file; do
  [[ "$code" == "code" ]] && continue

  existing_number="$(gh issue list \
    --repo "$repo" \
    --state all \
    --limit 200 \
    --json number,title \
    --jq ".[] | select(.title == \"$title\") | .number" \
    | head -n1)"

  if [[ -n "$existing_number" ]]; then
    if [[ -n "$expected_number" && "$existing_number" != "$expected_number" ]]; then
      echo "warning: $code expected #$expected_number but exact title exists as #$existing_number" >&2
    fi
    echo "$code: exists as #$existing_number"
    continue
  fi

  tmp_body="$(mktemp)"
  trap 'rm -f "$tmp_body"' EXIT
  awk 'NR == 1 && /^# / { next } { print }' "$root/$body_file" > "$tmp_body"

  url="$(gh issue create --repo "$repo" --title "$title" --body-file "$tmp_body")"
  echo "$code: created $url"
  rm -f "$tmp_body"
  trap - EXIT
done < "$root/issues/manifest.tsv"
