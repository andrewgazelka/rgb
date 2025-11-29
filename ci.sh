#!/usr/bin/env bash
set -euo pipefail

echo "Running cargo +nightly fmt --check..."
cargo +nightly fmt --check

echo "Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "Running cargo nextest run..."
cargo nextest run

echo "CI passed!"
