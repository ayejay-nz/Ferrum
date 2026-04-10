#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

ENGINE="${ENGINE:-./target/release/ferrum}"
DEPTH="${DEPTH:-14}"
MOVETIME="${MOVETIME:-}"
OUT_DIR="${OUT_DIR:-output/perf}"
CASE_FILTER="${CASE_FILTER:-}"
PERF_RECORD_ARGS="${PERF_RECORD_ARGS:--g --call-graph dwarf}"

slugify() {
  tr '[:upper:]' '[:lower:]' \
    | tr -cs '[:alnum:]' '_' \
    | sed 's/^_//; s/_$//'
}

run_case() {
  local name="$1"
  local position_cmd="$2"
  local go_cmd
  local slug
  local data_file
  local log_file

  if [[ -n "$MOVETIME" ]]; then
    go_cmd="go movetime $MOVETIME"
  else
    go_cmd="go depth $DEPTH"
  fi

  slug="$(printf '%s' "$name" | slugify)"
  data_file="$OUT_DIR/${slug}.data"
  log_file="$OUT_DIR/${slug}.log"

  printf 'Profiling %-20s -> %s\n' "$name" "$data_file"

  read -r -a perf_args <<<"$PERF_RECORD_ARGS"

  printf 'uci\nisready\nsetoption name UseBook value false\n%s\n%s\nquit\n' \
    "$position_cmd" "$go_cmd" \
    | perf record "${perf_args[@]}" -o "$data_file" -- "$ENGINE" >"$log_file" 2>&1

  printf '  log: %s\n' "$log_file"
  printf '  report command: perf report -i %s\n\n' "$data_file"
}

if [[ ! -x "$ENGINE" ]]; then
  printf 'engine binary not executable: %s\n' "$ENGINE" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

cases=(
  "Start Position|position startpos"
  "Open Sicilian|position fen r1bq1rk1/pp2bppp/2n2n2/2ppp3/4P3/2NP1NP1/PPP2PBP/R1BQ1RK1 w - - 0 8"
  "Tactical Middlegame|position fen 2r2rk1/pp1n1ppp/2pbpn2/q7/3PP3/2N1BN2/PPQ1BPPP/2RR2K1 w - - 0 12"
  "Quiet Endgame|position fen 8/5pk1/2p3p1/1pP1p2p/1P2P2P/5PK1/8/8 w - - 0 1"
  "Rook Endgame|position fen 8/8/4k3/2p2p2/2Pp1K2/3P4/8/3R4 w - - 0 1"
)

printf 'Perf profiling engine=%s ' "$ENGINE"
if [[ -n "$MOVETIME" ]]; then
  printf 'movetime=%sms\n' "$MOVETIME"
else
  printf 'depth=%s\n' "$DEPTH"
fi
printf 'Output directory: %s\n\n' "$OUT_DIR"

for case in "${cases[@]}"; do
  name="${case%%|*}"
  position_cmd="${case#*|}"

  if [[ -n "$CASE_FILTER" && "$name" != *"$CASE_FILTER"* ]]; then
    continue
  fi

  run_case "$name" "$position_cmd"
done
