#!/usr/bin/env bash
# egglog_te.sh — run egglog's term-encoding + desugar pipeline on a .egg file.
#
# Usage: scripts/egglog_te.sh <path/to/file.egg>
#
# Writes <path/to/file.te.egg> containing union-free egglog (only
# delete/insert/set, no (union …)). See EGGLOG_PLAN.md for why this is the
# input format for the ascent-dd port.
#
# Builds the egglog binary on first use (release mode). Subsequent runs are
# free — cargo skips rebuild.

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <file.egg>" >&2
  exit 64
fi

INPUT="$1"
if [[ ! -f "$INPUT" ]]; then
  echo "error: input file not found: $INPUT" >&2
  exit 66
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EGGLOG_DIR="$REPO_ROOT/others/egglog"
EGGLOG_BIN="$EGGLOG_DIR/target/release/egglog"

if [[ ! -x "$EGGLOG_BIN" ]]; then
  echo "building egglog release binary…" >&2
  cargo build --manifest-path "$EGGLOG_DIR/Cargo.toml" --release --bin egglog >&2
fi

OUT="${INPUT%.egg}.te.egg"
"$EGGLOG_BIN" --term-encoding --mode desugar "$INPUT" > "$OUT"
echo "wrote $OUT ($(wc -l < "$OUT") lines)"
