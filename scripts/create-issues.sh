#!/usr/bin/env bash
set -euo pipefail

repo="${1:-o3kio/araf}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Creating Araf planning issues in $repo"
echo "Requires: gh authenticated with issue-write access to the target repository."

epic_url=$(gh issue create --repo "$repo" --title "[EPIC] Araf MVP — O3K next-generation cloud console" --body-file "$root/issues/EPIC.md")
echo "EPIC: $epic_url"

while IFS=$'\t' read -r code title body_file; do
  [[ "$code" == "code" ]] && continue
  url=$(gh issue create --repo "$repo" --title "$title" --body-file "$root/$body_file")
  echo "$code: $url"
done < "$root/issues/manifest.tsv"
