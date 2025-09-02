#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."/gui
if ! command -v flutter >/dev/null 2>&1; then
  echo "Flutter is not installed or not on PATH." >&2
  exit 1
fi
flutter pub get
flutter run
