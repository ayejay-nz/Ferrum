#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

DEV="${DEV:-./target/release/ferrum}"
STABLE="${STABLE:-./stable/ferrum}"
DEPTH="${DEPTH:-10}"
MOVETIME="${MOVETIME:-}"
FENS_FILE="${FENS_FILE:-}"

run_engine() {
  local engine="$1"
  local position_cmd="$2"
  local output
  local go_cmd

  if [[ -n "$MOVETIME" ]]; then
    go_cmd="go movetime $MOVETIME"
  else
    go_cmd="go depth $DEPTH"
  fi

  output="$(
    printf 'uci\nisready\nsetoption name UseBook value false\n%s\n%s\nquit\n' \
      "$position_cmd" "$go_cmd" | "$engine"
  )"

  local info_line
  local bestmove_line
  info_line="$(printf '%s\n' "$output" | grep '^info depth ' | tail -n 1 || true)"
  bestmove_line="$(printf '%s\n' "$output" | grep '^bestmove ' | tail -n 1 || true)"

  printf '%s\n' "$info_line"
  printf '%s\n' "$bestmove_line"
}

extract_field() {
  local line="$1"
  local key="$2"

  awk -v key="$key" '
    {
      for (i = 1; i < NF; i++) {
        if ($i == key) {
          print $(i + 1)
          exit
        }
      }
    }
  ' <<<"$line"
}

print_report() {
  local name="$1"
  local position_cmd="$2"
  local dev_info="$3"
  local dev_best="$4"
  local stable_info="$5"
  local stable_best="$6"

  local dev_nodes dev_time dev_nps stable_nodes stable_time stable_nps
  dev_nodes="$(extract_field "$dev_info" nodes)"
  dev_time="$(extract_field "$dev_info" time)"
  dev_nps="$(extract_field "$dev_info" nps)"
  stable_nodes="$(extract_field "$stable_info" nodes)"
  stable_time="$(extract_field "$stable_info" time)"
  stable_nps="$(extract_field "$stable_info" nps)"

  printf '=== %s ===\n' "$name"
  printf 'Position: %s\n' "$position_cmd"
  printf 'dev   : %s\n' "$dev_info"
  printf '        %s\n' "$dev_best"
  printf 'stable: %s\n' "$stable_info"
  printf '        %s\n' "$stable_best"

  if [[ -n "$dev_nps" && -n "$stable_nps" ]]; then
    printf 'NPS   : dev=%s stable=%s\n' "$dev_nps" "$stable_nps"
  fi
  if [[ -n "$dev_nodes" && -n "$stable_nodes" ]]; then
    printf 'Nodes : dev=%s stable=%s\n' "$dev_nodes" "$stable_nodes"
  fi
  if [[ -n "$dev_time" && -n "$stable_time" ]]; then
    printf 'Time  : dev=%sms stable=%sms\n' "$dev_time" "$stable_time"
  fi

  printf '\n'
}

load_cases() {
  local -n out_cases=$1

  if [[ -n "$FENS_FILE" ]]; then
    while IFS='|' read -r name fen; do
      [[ -z "${name// }" ]] && continue
      [[ "${name:0:1}" == "#" ]] && continue
      out_cases+=("$name|$fen")
    done <"$FENS_FILE"
    return
  fi

  out_cases+=(
    "Start Position|position startpos"
    "Open Sicilian|position fen r1bq1rk1/pp2bppp/2n2n2/2ppp3/4P3/2NP1NP1/PPP2PBP/R1BQ1RK1 w - - 0 8"
    "Tactical Middlegame|position fen 2r2rk1/pp1n1ppp/2pbpn2/q7/3PP3/2N1BN2/PPQ1BPPP/2RR2K1 w - - 0 12"
    "Quiet Endgame|position fen 8/5pk1/2p3p1/1pP1p2p/1P2P2P/5PK1/8/8 w - - 0 1"
    "Rook Endgame|position fen 8/8/4k3/2p2p2/2Pp1K2/3P4/8/3R4 w - - 0 1"
  )
}

if [[ ! -x "$DEV" ]]; then
  printf 'dev binary not executable: %s\n' "$DEV" >&2
  exit 1
fi

if [[ ! -x "$STABLE" ]]; then
  printf 'stable binary not executable: %s\n' "$STABLE" >&2
  exit 1
fi

cases=()
load_cases cases

if [[ "${#cases[@]}" -eq 0 ]]; then
  printf 'no benchmark positions loaded\n' >&2
  exit 1
fi

printf 'Comparing engines with '
if [[ -n "$MOVETIME" ]]; then
  printf 'movetime=%sms\n\n' "$MOVETIME"
else
  printf 'depth=%s\n\n' "$DEPTH"
fi

for case in "${cases[@]}"; do
  name="${case%%|*}"
  position_cmd="${case#*|}"

  mapfile -t dev_lines < <(run_engine "$DEV" "$position_cmd")
  mapfile -t stable_lines < <(run_engine "$STABLE" "$position_cmd")

  dev_info="${dev_lines[0]:-}"
  dev_best="${dev_lines[1]:-}"
  stable_info="${stable_lines[0]:-}"
  stable_best="${stable_lines[1]:-}"

  print_report \
    "$name" \
    "$position_cmd" \
    "$dev_info" \
    "$dev_best" \
    "$stable_info" \
    "$stable_best"
done
