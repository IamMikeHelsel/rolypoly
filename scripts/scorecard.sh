#!/usr/bin/env bash
set -euo pipefail

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "# RolyPoly Release Scorecard" > SCORECARD.md
echo >> SCORECARD.md

echo "## Test Summary" >> SCORECARD.md
echo '```' >> SCORECARD.md
TEST_OUT=$(cargo test --all -q 2>&1 || true)
echo "$TEST_OUT" >> SCORECARD.md
echo '```' >> SCORECARD.md

echo >> SCORECARD.md
echo "## Performance (Synthetic)" >> SCORECARD.md

# Create synthetic files
WORK="$TMP/work"; mkdir -p "$WORK"
python3 - <<'PY' "$WORK"
import os, sys
work = sys.argv[1]
with open(os.path.join(work, 'small.txt'), 'w') as f: f.write('Hello World')
with open(os.path.join(work, 'medium.txt'), 'w') as f: f.write('A'*1024*50)
with open(os.path.join(work, 'large.txt'), 'w') as f: f.write('B'*1024*1024)
os.makedirs(os.path.join(work, 'sub'), exist_ok=True)
with open(os.path.join(work, 'sub', 'nested.txt'), 'w') as f: f.write('Nested')
PY

ARCH="$TMP/test.zip"
EXTR="$TMP/extract"
mkdir -p "$EXTR"

measure(){
  local cmd="$1"; shift
  local start=$(date +%s%3N)
  "$@" >/dev/null 2>&1 || true
  local end=$(date +%s%3N)
  echo $((end-start))
}

SIZE_MB=$(du -sm "$WORK" | awk '{print $1}')

BUILD_BIN=./target/release/rolypoly
[ -x "$BUILD_BIN" ] || cargo build --release --bin rolypoly >/dev/null

CREATE_MS=$(measure create "$BUILD_BIN" create "$ARCH" "$WORK/small.txt" "$WORK/medium.txt" "$WORK/large.txt" "$WORK/sub")
EXTRACT_MS=$(measure extract "$BUILD_BIN" extract "$ARCH" -o "$EXTR")

THR_CREATE=$(python3 - <<PY $SIZE_MB $CREATE_MS
import sys
size_mb=int(sys.argv[1]); ms=int(sys.argv[2]) or 1
print(f"{(size_mb*1000)/ms:.2f}")
PY
)
THR_EXTRACT=$(python3 - <<PY $SIZE_MB $EXTRACT_MS
import sys
size_mb=int(sys.argv[1]); ms=int(sys.argv[2]) or 1
print(f"{(size_mb*1000)/ms:.2f}")
PY
)

echo >> SCORECARD.md
echo '| Operation | Data (MB) | Time (ms) | MB/s |' >> SCORECARD.md
echo '|-----------|-----------|-----------|------|' >> SCORECARD.md
echo "| Create    | ${SIZE_MB}       | ${CREATE_MS}      | ${THR_CREATE} |" >> SCORECARD.md
echo "| Extract   | ${SIZE_MB}       | ${EXTRACT_MS}      | ${THR_EXTRACT} |" >> SCORECARD.md

echo >> SCORECARD.md
echo "Generated at $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> SCORECARD.md

echo "SCORECARD.md"

