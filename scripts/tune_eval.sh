#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

TUNE_FILE="${TUNE_FILE:-./src/tune.rs}"
RUN_ID="$(date +%s)"
LOG_DIR="${LOG_DIR:-./output/tune_$RUN_ID}"
RESULTS_CSV="$LOG_DIR/results.csv"

TC="${TC:-3+0.05}"
ROUNDS="${ROUNDS:-1000}"
CONCURRENCY="${CONCURRENCY:-12}"
RESTORE_BUILD_ON_EXIT="${RESTORE_BUILD_ON_EXIT:-1}"

mkdir -p "$LOG_DIR"

backup_file="$(mktemp)"
cp "$TUNE_FILE" "$backup_file"

cleanup() {
    cp "$backup_file" "$TUNE_FILE"

    if [[ "$RESTORE_BUILD_ON_EXIT" == "1" ]]; then
        cargo build --release >/dev/null 2>&1 || true
    fi

    rm -f "$backup_file"
}

trap cleanup EXIT

if [[ -n "${TARGETS:-}" ]]; then
    read -r -a TARGET_LIST <<< "$TARGETS"
else
    TARGET_LIST=(
        # TRIPLED_PAWNS
        # BISHOP_PAIR
        ISOLATED_PAWN
        DOUBLED_PAWNS
        PASSED_PAWN
    )
fi

candidate_list() {
    case "$1" in
        # TRIPLED_PAWNS)
        #     printf '%s\n' "${TRIPLED_PAWNS_CANDIDATES:--40:-50 -30:-40 -50:-60}"
        #     ;;
        # BISHOP_PAIR)
        #     printf '%s\n' "${BISHOP_PAIR_CANDIDATES:--5:15 0:15 5:15 10:20}"
        #     ;;
        ISOLATED_PAWN)
            if [[ -n "${ISOLATED_PAWN_CANDIDATES:-}" ]]; then
                printf '%s\n' "${ISOLATED_PAWN_CANDIDATES}"
            else
                local mg_min="${ISOLATED_PAWN_MG_MIN:--3}"
                local mg_max="${ISOLATED_PAWN_MG_MAX:-3}"
                local eg_min="${ISOLATED_PAWN_EG_MIN:--3}"
                local eg_max="${ISOLATED_PAWN_EG_MAX:-3}"
                local mg
                local eg

                for ((mg = mg_min; mg <= mg_max; mg++)); do
                    for ((eg = eg_min; eg <= eg_max; eg++)); do
                        printf '%s:%s ' "$mg" "$eg"
                    done
                done
            fi
            ;;
        DOUBLED_PAWNS)
            printf '%s\n' "${DOUBLED_PAWNS_CANDIDATES:--3:-5 -5:-8 -10:-14 -13:-18 -18:-23}"
            ;;
        PASSED_PAWN)
            printf '%s\n' "${PASSED_PAWN_CANDIDATES: 5:10 20:40 30:60 40:80}"
            ;;
        *)
            echo "Unknown target: $1" >&2
            return 1
            ;;
    esac
}

set_tune_pair() {
    local name="$1"
    local mg="$2"
    local eg="$3"

    perl -0pi -e \
        "s{
            pub\\ const\\ ${name}:\\ Score\\ =\\ Score\\ \\{
            \\s*mg:\\s*-?\\d+\\s*,
            \\s*eg:\\s*-?\\d+\\s*
            \\}\\s*;
        }{pub const ${name}: Score = Score { mg: ${mg}, eg: ${eg} };}gx;" \
        "$TUNE_FILE"

    grep -Eq "pub const ${name}: Score = Score \{ mg: ${mg}, eg: ${eg} \};" "$TUNE_FILE"
}

extract_summary() {
    local log_file="$1"
    local elo
    local sprt

    elo="$(grep 'Elo difference:' "$log_file" | tail -n 1 | sed 's/^[[:space:]]*//')"
    sprt="$(grep '^SPRT:' "$log_file" | tail -n 1 | sed 's/^[[:space:]]*//')"

    printf '%s | %s\n' "${elo:-no_elo_summary}" "${sprt:-no_sprt_summary}"
}

printf 'target,mg,eg,status,summary,log,pgn\n' > "$RESULTS_CSV"

for target in "${TARGET_LIST[@]}"; do
    read -r -a candidates <<< "$(candidate_list "$target")"

    for pair in "${candidates[@]}"; do
        mg="${pair%%:*}"
        eg="${pair##*:}"

        cp "$backup_file" "$TUNE_FILE"
        if ! set_tune_pair "$target" "$mg" "$eg"; then
            status="rewrite_failed"
            printf '%s,%s,%s,%s,%s,%s,%s\n' \
                "$target" "$mg" "$eg" "$status" "rewrite_failed" "" "" \
                >> "$RESULTS_CSV"
            echo "Rewrite failed for $target mg=$mg eg=$eg"
            continue
        fi

        echo "=== $target mg=$mg eg=$eg ==="

        build_log="$LOG_DIR/${target}_${mg}_${eg}.build.log"
        if cargo build --release >"$build_log" 2>&1; then
            status="ok"
        else
            status="build_failed"
            printf '%s,%s,%s,%s,%s,%s,%s\n' \
                "$target" "$mg" "$eg" "$status" "build_failed" "$build_log" "" \
                >> "$RESULTS_CSV"
            echo "Build failed for $target mg=$mg eg=$eg"
            continue
        fi

        run_log="$LOG_DIR/${target}_${mg}_${eg}.log"
        pgn_out="$LOG_DIR/${target}_${mg}_${eg}.pgn"

        if OUT="$pgn_out" TC="$TC" ROUNDS="$ROUNDS" CONCURRENCY="$CONCURRENCY" \
            ./sprt.sh >"$run_log" 2>&1; then
            status="ok"
        else
            status="sprt_failed"
        fi

        summary="$(extract_summary "$run_log" | tr ',' ';')"
        printf '%s,%s,%s,%s,"%s",%s,%s\n' \
            "$target" "$mg" "$eg" "$status" "$summary" "$run_log" "$pgn_out" \
            >> "$RESULTS_CSV"

        echo "$summary"
    done
done

echo "Results written to $RESULTS_CSV"
