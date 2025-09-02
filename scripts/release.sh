#!/usr/bin/env bash
set -euo pipefail

# Simple release helper for rolypoly
# Usage:
#   ./scripts/release.sh cli 0.1.0         # stable release -> tag cli-v0.1.0
#   ./scripts/release.sh cli 0.1.0-rc2     # pre-release   -> tag cli-v0.1.0-rc2
#   ./scripts/release.sh gui 0.0.1-rc1     # GUI prerelease -> tag gui-v0.0.1-rc1

KIND=${1:-}
VERSION=${2:-}

if [[ -z "$KIND" || -z "$VERSION" ]]; then
  echo "Usage: $0 <cli|gui> <version>" >&2
  exit 1
fi

case "$KIND" in
  cli)
    TAG="cli-v${VERSION}"
    BRANCH="release/cli"
    ;;
  gui)
    TAG="gui-v${VERSION}"
    BRANCH="release/gui"
    ;;
  *)
    echo "Unknown KIND: $KIND (expected cli|gui)" >&2
    exit 1
    ;;
esac

echo "Ensuring branch $BRANCH tracks main..."
MAIN=$(git rev-parse main)
if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
  git update-ref "refs/heads/$BRANCH" "$MAIN"
else
  git branch "$BRANCH" "$MAIN"
fi

echo "Creating annotated tag $TAG at $BRANCH..."
COMMIT=$(git rev-parse "$BRANCH")
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  echo "Tag already exists: $TAG" >&2
  exit 1
fi
git tag -a "$TAG" -m "${KIND^^} $VERSION" "$COMMIT"

cat <<MSG

Created tag: $TAG at $COMMIT
Next steps:
  git push origin $BRANCH
  git push origin $TAG

The GitHub Actions workflow will build and (for CLI/GUI release tags) create a Release.
MSG

