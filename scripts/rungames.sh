#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

engine_location="${APP:-$HOME/Cute_Chess-1.4.0-x86_64.AppImage}"

engine_strength="${1:?usage: $0 <engine_strength> <tc> <games> <concurrency>}"
tc="${2:?usage: $0 <engine_strength> <tc> <games> <concurrency>}"
games="${3:?usage: $0 <engine_strength> <tc> <games> <concurrency>}"
concurrency="${4:?usage: $0 <engine_strength> <tc> <games> <concurrency>}"

mkdir -p ./output

current_unix_time="$(date +%s)"
output_file="./output/ferrum_vs_sf${engine_strength}_${games}_${tc}_${current_unix_time}.pgn"

command=(
  "$engine_location" cli
  -engine name=ferrum cmd=./target/release/ferrum proto=uci
  -engine name="SF${engine_strength}" cmd=stockfish proto=uci
  option.UCI_LimitStrength=true
  option.UCI_Elo="$engine_strength"
  option.Threads=1
  -each tc="$tc" timemargin=50
  -games "$games"
  -repeat
  -recover
  -concurrency "$concurrency"
  -pgnout "$output_file"
)

printf '%q ' "${command[@]}"
printf '\n'

"${command[@]}"
