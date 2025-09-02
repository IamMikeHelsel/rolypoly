#!/usr/bin/env bash
set -euo pipefail

# Generate a simple changelog section from git log since last tag
# Usage: ./scripts/changelog.sh [NEW_TAG]

NEW_TAG=${1:-UNRELEASED}
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

echo "## ${NEW_TAG} ($(date +%Y-%m-%d))"
echo
if [[ -n "$LAST_TAG" ]]; then
  git log --pretty=format:'- %s (%h)' "${LAST_TAG}..HEAD"
else
  git log --pretty=format:'- %s (%h)'
fi
echo

