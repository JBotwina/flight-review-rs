#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "cargo-watch is required. Install it with: cargo install cargo-watch" >&2
  exit 1
fi

mkdir -p data/files

exec cargo watch \
  --ignore data \
  --ignore frontend \
  --ignore target \
  -x 'run -p flight-review-server -- serve --db sqlite://data/flight-review.db?mode=rwc --storage file://data/files --host 127.0.0.1 --port 8080'
