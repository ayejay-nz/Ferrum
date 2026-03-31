#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

if [[ ! -x ./target/release/ferrum ]]; then
    echo "missing ./target/release/ferrum; build it first" >&2
    exit 1
fi

printf 'uci\nquit\n' | ./target/release/ferrum >/dev/null

mkdir -p stable
install -m 755 ./target/release/ferrum ./stable/ferrum.new
mv ./stable/ferrum.new ./stable/ferrum
