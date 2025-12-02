#!/bin/bash
# Benchmark script for testing different JS runtimes (qjs, deno, boa, node, bun)
# Usage: ./bench.sh [-n count] [runtime...]
# Example: ./bench.sh qjs deno boa node bun  # test all runtimes, all cases
#          ./bench.sh -n 10 qjs              # test only qjs with first 10 cases
#          ./bench.sh -n 0 qjs deno          # test all cases (0 = all)

set -e

CASES_FILE="cases.csv"
PLAYERS_DIR="players"
EXE="ejs"
MAX_TESTS=0  # 0 means all tests

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
RUNTIMES=()
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--count)
            MAX_TESTS="$2"
            shift 2
            ;;
        *)
            RUNTIMES+=("$1")
            shift
            ;;
    esac
done

# Default runtimes if none specified
if [ ${#RUNTIMES[@]} -eq 0 ]; then
    RUNTIMES=("qjs" "deno" "boa" "node" "bun")
fi

# Check if executable exists
if ! command -v "$EXE" &> /dev/null; then
    echo -e "${RED}Error: Cannot find $EXE in PATH${NC}"
    exit 1
fi

echo -e "${BLUE}Using executable: $(which $EXE)${NC}"
echo -e "${BLUE}Cases file: $CASES_FILE${NC}"
echo -e "${BLUE}Players dir: $PLAYERS_DIR${NC}"
if [ "$MAX_TESTS" -gt 0 ]; then
    echo -e "${BLUE}Max tests: $MAX_TESTS${NC}"
else
    echo -e "${BLUE}Max tests: all${NC}"
fi
echo ""

# Function to run tests for a specific runtime
run_tests() {
    local runtime=$1
    local passed=0
    local failed=0
    local total=0
    local test_count=0
    local start_time=$(date +%s.%N 2>/dev/null || date +%s)

    echo -e "${BLUE}Testing runtime: ${YELLOW}$runtime${NC}"
    echo "----------------------------------------"

    # Read and process tests line by line
    local current_player=""
    local n_args=""
    local sig_args=""
    local expected_list=""

    while read -r player type input expected || [ -n "$player" ]; do
        [ -z "$player" ] && continue

        # Check if we've reached max tests
        if [ "$MAX_TESTS" -gt 0 ] && [ "$test_count" -ge "$MAX_TESTS" ]; then
            break
        fi

        local player_file="$PLAYERS_DIR/$player"
        if [ ! -f "$player_file" ]; then
            continue
        fi

        # If player changed, run tests for previous player
        if [ -n "$current_player" ] && [ "$player_file" != "$current_player" ]; then
            run_player_tests "$runtime" "$current_player" "$n_args" "$sig_args" "$expected_list"
            n_args=""
            sig_args=""
            expected_list=""
        fi

        current_player="$player_file"

        if [ "$type" = "n" ]; then
            n_args="$n_args n:$input"
        else
            sig_args="$sig_args sig:$input"
        fi
        expected_list="$expected_list|$type:$input:$expected"
        ((test_count++))
    done < "$CASES_FILE"

    # Run tests for last player
    if [ -n "$current_player" ]; then
        run_player_tests "$runtime" "$current_player" "$n_args" "$sig_args" "$expected_list"
    fi

    local end_time=$(date +%s.%N 2>/dev/null || date +%s)
    local duration=$(awk "BEGIN {printf \"%.3f\", $end_time - $start_time}")

    echo ""
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}Results: $passed/$total passed${NC} (${duration}s)"
    else
        echo -e "${YELLOW}Results: $passed/$total passed, $failed failed${NC} (${duration}s)"
    fi
    echo ""

    # Return results
    echo "$runtime:$passed:$failed:$total:$duration"
}

# Helper function to run tests for a single player
run_player_tests() {
    local runtime=$1
    local player_file=$2
    local n_args=$3
    local sig_args=$4
    local expected_list=$5

    # Run the command
    local output
    output=$("$EXE" --runtime "$runtime" "$player_file" $n_args $sig_args 2>&1) || true

    # Check each expected result
    IFS='|'
    for item in $expected_list; do
        [ -z "$item" ] && continue

        local type=$(echo "$item" | cut -d: -f1)
        local input=$(echo "$item" | cut -d: -f2)
        local expected=$(echo "$item" | cut -d: -f3)

        ((total++))

        # Check if output contains expected result
        if echo "$output" | grep -q "\"$input\":\"$expected\""; then
            ((passed++))
        else
            ((failed++))
            local player_name=$(basename "$player_file")
            echo -e "${RED}FAIL${NC}: $player_name $type"
            echo "  Input: $input"
            echo "  Expected: $expected"
        fi
    done
    IFS=$' \t\n'
}

# Make counters accessible in subshell
passed=0
failed=0
total=0

# Store results for summary
RESULTS=""

# Run tests for each runtime
for runtime in "${RUNTIMES[@]}"; do
    passed=0
    failed=0
    total=0
    result=$(run_tests "$runtime" | tail -1)
    RESULTS="$RESULTS$result
"
done

# Print summary
echo "========================================"
echo -e "${BLUE}SUMMARY${NC}"
echo "========================================"
printf "%-10s %8s %8s %8s %12s\n" "Runtime" "Passed" "Failed" "Total" "Time"
echo "----------------------------------------"

echo "$RESULTS" | while IFS=':' read -r runtime passed failed total duration; do
    [ -z "$runtime" ] && continue
    if [ "$failed" -eq 0 ]; then
        color=$GREEN
    else
        color=$YELLOW
    fi
    printf "${color}%-10s %8s %8s %8s %10.3fs${NC}\n" "$runtime" "$passed" "$failed" "$total" "$duration"
done
