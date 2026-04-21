#!/bin/bash
# =============================================================================
# Integration benchmark runner — Ascent++ batch + DD-batch backends.
#
# Runs each ported bench against one or more datasets under ../SRDatalog/benchmarks/data/.
# Prints a per-run line and a tab-separated summary at the end.
#
# Usage:
#   ./run_all.sh                              # all benches, default datasets
#   ./run_all.sh --only cspa,andersen         # subset
#   ./run_all.sh --skip doop,ddisasm
#   ./run_all.sh --runs 3
#   ./run_all.sh --data /path/to/data         # override data root
#   ./run_all.sh --variant batch              # only _batch bins (skip _dd)
#   ./run_all.sh --variant dd                 # only _dd bins
#   ./run_all.sh --timeout 300                # seconds per run (default 600)
# =============================================================================
set -u
set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA_ROOT="${BENCH_DATA_ROOT:-$REPO_ROOT/../SRDatalog/benchmarks/data}"
DOOP_META_ROOT="${BENCH_DOOP_META_ROOT:-$REPO_ROOT/../SRDatalog/integration_tests/examples/doop}"
DDISASM_META_FILE_NAME="ddisasm_consts.json"
RESULTS_DIR="${BENCH_RESULTS_DIR:-$SCRIPT_DIR/results}"

NUM_RUNS=1
TIMEOUT=600
ONLY=""
SKIP=""
VARIANT="all"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --runs)    NUM_RUNS="$2"; shift 2;;
        --only)    ONLY="$2"; shift 2;;
        --skip)    SKIP="$2"; shift 2;;
        --data)    DATA_ROOT="$2"; shift 2;;
        --timeout) TIMEOUT="$2"; shift 2;;
        --variant) VARIANT="$2"; shift 2;;
        -h|--help)
            sed -n '2,18p' "$0"; exit 0;;
        *) echo "Unknown arg: $1" >&2; exit 2;;
    esac
done

mkdir -p "$RESULTS_DIR"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
OUT_CSV="$RESULTS_DIR/run_all_${TIMESTAMP}.csv"
echo "bench,variant,dataset,run,status,wall_ms,exec_ms" > "$OUT_CSV"

should_run() {
    local name="$1"
    [[ -n "$ONLY" && ",$ONLY," != *",$name,"* ]] && return 1
    [[ -n "$SKIP" && ",$SKIP," == *",$name,"* ]] && return 1
    return 0
}

want_variant() {
    case "$VARIANT" in
        all)   return 0;;
        batch) [[ "$1" == "batch" ]];;
        dd)    [[ "$1" == "dd" ]];;
    esac
}

# -----------------------------------------------------------------------------
# Build every bench up-front (in release). Fail fast if anything's broken.
# -----------------------------------------------------------------------------
build_one() {
    local crate="$1"
    echo "── build $crate ──"
    if ! (cd "$SCRIPT_DIR/$crate" && cargo build --release 2>&1 | tail -3); then
        echo "[FATAL] build failed for $crate" >&2
        exit 1
    fi
}

echo "Building all bench crates..."
for c in cspa andersen sg polonius doop ddisasm; do
    should_run "$c" || { echo "── skip build $c ──"; continue; }
    build_one "$c"
done
echo ""

# -----------------------------------------------------------------------------
# Run helper: invokes the binary, extracts "Execution:" duration, records CSV.
# -----------------------------------------------------------------------------
parse_exec_ms() {
    # Accepts stderr lines like "Execution: 12.3ms", "Execution: 1.2s",
    # "Execution: 123µs" and prints ms as float (or empty on miss).
    awk '
        /^Execution:/ {
            v = $2
            # unit may be ms, s, µs, us, ns
            if (v ~ /ms$/)       { sub(/ms$/, "", v); printf "%.3f", v + 0; exit }
            else if (v ~ /s$/)   { sub(/s$/,  "", v); printf "%.3f", (v + 0) * 1000; exit }
            else if (v ~ /µs$/)  { sub(/µs$/, "", v); printf "%.3f", (v + 0) / 1000; exit }
            else if (v ~ /us$/)  { sub(/us$/, "", v); printf "%.3f", (v + 0) / 1000; exit }
            else if (v ~ /ns$/)  { sub(/ns$/, "", v); printf "%.6f", (v + 0) / 1e6; exit }
        }'
}

run_bench() {
    local bench="$1" variant="$2" dataset="$3" data_dir="$4"; shift 4
    local extra_args=("$@")
    local bin_path="$SCRIPT_DIR/$bench/target/release/${bench}_${variant}"
    if [[ ! -x "$bin_path" ]]; then
        echo "  [skip] binary missing: $bin_path"
        return
    fi
    if [[ ! -d "$data_dir" ]]; then
        echo "  [skip] data missing: $data_dir"
        return
    fi

    for run in $(seq 1 "$NUM_RUNS"); do
        local stderr_file
        stderr_file="$(mktemp)"
        local t0 t1 status wall_ms exec_ms
        t0=$(date +%s%N)
        if timeout "$TIMEOUT" "$bin_path" "$data_dir" "${extra_args[@]}" \
                >/dev/null 2>"$stderr_file"; then
            status="ok"
        else
            rc=$?
            status="fail(rc=$rc)"
            if [[ $rc -eq 124 ]]; then status="timeout"; fi
        fi
        t1=$(date +%s%N)
        wall_ms=$(( (t1 - t0) / 1000000 ))
        exec_ms="$(parse_exec_ms < "$stderr_file")"
        echo "  ${bench}_${variant}  ${dataset}  run${run}  ${status}  wall=${wall_ms}ms  exec=${exec_ms:-?}ms"
        [[ "$status" != "ok" && -s "$stderr_file" ]] && tail -3 "$stderr_file" | sed 's/^/    | /'
        echo "$bench,$variant,$dataset,$run,$status,$wall_ms,$exec_ms" >> "$OUT_CSV"
        rm -f "$stderr_file"
    done
}

# -----------------------------------------------------------------------------
# Dispatch per-bench. Each block enumerates datasets & constructs CLI args.
# -----------------------------------------------------------------------------

run_cspa() {
    local datasets=(cspa-httpd cspa-linux cspa-postgre)
    for d in "${datasets[@]}"; do
        local dir="$DATA_ROOT/andersen/$d"
        want_variant batch && run_bench cspa batch "$d" "$dir"
        want_variant dd    && run_bench cspa dd    "$d" "$dir"
    done
}

run_andersen() {
    local datasets=(andersen-large andersen-medium)
    for d in "${datasets[@]}"; do
        local dir="$DATA_ROOT/andersen/$d"
        want_variant batch && run_bench andersen batch "$d" "$dir"
        want_variant dd    && run_bench andersen dd    "$d" "$dir"
    done
}

run_sg() {
    local datasets=()
    # galen is the canonical dataset for sg rules; the others under data/sg/
    # are smaller graphs used for plain transitive-closure tests in SRDatalog
    # and don't populate the ternary R/C/U/S relations — include only galen here.
    if [[ -d "$DATA_ROOT/galen/galen" ]]; then datasets+=(galen); fi
    for d in "${datasets[@]}"; do
        local dir="$DATA_ROOT/galen/$d"
        want_variant batch && run_bench sg batch "$d" "$dir"
        want_variant dd    && run_bench sg dd    "$d" "$dir"
    done
}

run_polonius() {
    local dirs=(clap-rs materialize-render polonius-facts scallop-parser wgpu)
    for d in "${dirs[@]}"; do
        local dir="$DATA_ROOT/polonius/$d"
        want_variant batch && run_bench polonius batch "$d" "$dir"
    done
}

run_doop() {
    # Doop needs 15 dataset-specific interned constants read from
    # $DOOP_META_ROOT/<dataset>_meta.json.
    local datasets=(batik eclipse biojava xalan zxing)
    for d in "${datasets[@]}"; do
        local meta="$DOOP_META_ROOT/${d}_meta.json"
        local dir="$DATA_ROOT/doop/${d}_interned"
        if [[ ! -f "$meta" ]]; then
            echo "  [skip] meta missing: $meta (bench: doop/$d)"
            continue
        fi
        local args
        args=$(python3 -c '
import json, sys
m = json.load(open(sys.argv[1]))
keys = ["abstract","public","static","main","main_descriptor",
        "java_lang_Object","java_lang_Cloneable","java_io_Serializable",
        "clinit","clinit_descriptor","class_init_method",
        "register_natives_method","desiredAssertionStatus_method",
        "java_lang_Object_array","java_lang_String_type"]
print(" ".join(str(m[k]) for k in keys))' "$meta")
        # shellcheck disable=SC2086
        want_variant batch && run_bench doop batch "$d" "$dir" $args
    done
}

run_ddisasm() {
    local datasets=(z3 cvc5 lean4 coqidetop)
    for d in "${datasets[@]}"; do
        local dir="$DATA_ROOT/ddisasm/$d"
        local consts="$dir/$DDISASM_META_FILE_NAME"
        if [[ ! -f "$consts" ]]; then
            echo "  [skip] consts missing: $consts (bench: ddisasm/$d)"
            continue
        fi
        local args
        args=$(python3 -c '
import json, sys
c = json.load(open(sys.argv[1]))
print(c["LOAD"], c["STORE"], c["NONE"], c["PCRelative"])' "$consts")
        # shellcheck disable=SC2086
        want_variant batch && run_bench ddisasm batch "$d" "$dir" $args
    done
}

echo "Data root: $DATA_ROOT"
echo "Doop meta: $DOOP_META_ROOT"
echo "Results:   $OUT_CSV"
echo "Runs:      $NUM_RUNS   Timeout: ${TIMEOUT}s   Variant: $VARIANT"
echo ""

for bench in cspa andersen sg polonius doop ddisasm; do
    if ! should_run "$bench"; then
        echo "── skip $bench ──"
        continue
    fi
    echo "━━━ $bench ━━━"
    case "$bench" in
        cspa)     run_cspa;;
        andersen) run_andersen;;
        sg)       run_sg;;
        polonius) run_polonius;;
        doop)     run_doop;;
        ddisasm)  run_ddisasm;;
    esac
    echo ""
done

echo "Done. Results → $OUT_CSV"
