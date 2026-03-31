#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

APP="${APP:-$HOME/Cute_Chess-1.4.0-x86_64.AppImage}"
OUT="${OUT:-./output/sprt_$(date +%s).pgn}"
TC="${TC:-5+0.05}"
CONCURRENCY="${CONCURRENCY:-12}"
ROUNDS="${ROUNDS:-25000}"
ELO0="${ELO0:-0}"
ELO1="${ELO1:-5}"
ALPHA="${ALPHA:-0.05}"
BETA="${BETA:-0.05}"

# Optional external opening suite for lower-variance SPRT runs.
# Example:
# OPENINGS_FILE=../../openings/UHO_4060_v3.epd
# OPENINGS_FORMAT=epd
# OPENINGS_ORDER=random
OPENINGS_FILE="${OPENINGS_FILE:-}"
OPENINGS_FORMAT="${OPENINGS_FORMAT:-epd}"
OPENINGS_ORDER="${OPENINGS_ORDER:-random}"
OPENINGS_POLICY="${OPENINGS_POLICY:-encounter}"

cmd=(
  "$APP" cli
  -engine name=dev cmd=./target/release/ferrum proto=uci restart=on option.UseBook=false
  -engine name=stable cmd=./stable/ferrum proto=uci restart=on option.UseBook=false
  -each tc="$TC" timemargin=50
)

if [[ -n "$OPENINGS_FILE" ]]; then
  cmd+=(
    -openings
    file="$OPENINGS_FILE"
    format="$OPENINGS_FORMAT"
    order="$OPENINGS_ORDER"
    policy="$OPENINGS_POLICY"
  )
fi

cmd+=(
  -repeat
  -sprt
  elo0="$ELO0"
  elo1="$ELO1"
  alpha="$ALPHA"
  beta="$BETA"
  -rounds "$ROUNDS"
  -concurrency "$CONCURRENCY"
  -recover
  -draw movenumber=40 movecount=8 score=10
  -resign movecount=3 score=600 twosided=true
  -ratinginterval 20
  -outcomeinterval 20
  -pgnout "$OUT"
)

printf '%q ' "${cmd[@]}"
printf '\n'

"${cmd[@]}"
