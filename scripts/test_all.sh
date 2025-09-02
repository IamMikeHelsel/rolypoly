#!/usr/bin/env bash
set -euo pipefail

echo "==> Formatting check"
cargo fmt --all -- --check || true

echo "==> Lint (all targets, all features)"
cargo clippy --all-targets --all-features -- -D warnings || true

echo "==> Run CLI tests (no GUI feature)"
cargo test --all --no-default-features

echo "==> Build GUI binary (feature gui)"
cargo build --features gui --bin rusty-gui

echo "==> Run GUI tests (feature gui)"
cargo test --features gui --tests

echo "==> Done"

