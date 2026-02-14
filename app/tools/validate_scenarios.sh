#!/bin/bash

# Scenario validation script
# Usage: ./tools/validate_scenarios.sh [OPTIONS] [FILES...]
#
# This script wraps the scenario-validator tool to provide easy validation
# of TOML scenario files with additional convenience features.

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
STRICT_MODE=false
CHECK_ASSETS=true
WATCH_MODE=false
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --strict|-s)
            STRICT_MODE=true
            shift
            ;;
        --no-assets)
            CHECK_ASSETS=false
            shift
            ;;
        --watch|-w)
            WATCH_MODE=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Scenario Validation Script"
            echo "Validates visual novel scenario TOML files"
            echo ""
            echo "USAGE:"
            echo "    ./tools/validate_scenarios.sh [OPTIONS] [FILES...]"
            echo ""
            echo "OPTIONS:"
            echo "    -s, --strict        Enable strict validation mode"
            echo "        --no-assets     Skip asset file validation"
            echo "    -w, --watch         Watch files for changes and re-validate"
            echo "    -v, --verbose       Enable verbose output"
            echo "    -h, --help          Show this help message"
            echo ""
            echo "EXAMPLES:"
            echo "    ./tools/validate_scenarios.sh                      # Validate all scenarios"
            echo "    ./tools/validate_scenarios.sh chapter_01.toml      # Validate specific file"
            echo "    ./tools/validate_scenarios.sh --strict --watch     # Strict validation with watch mode"
            exit 0
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}" >&2
            exit 1
            ;;
        *)
            # Remaining arguments are files to validate
            break
            ;;
    esac
done

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo command not found. Make sure Rust is installed.${NC}" >&2
    exit 1
fi

# Build the validator tool if needed
echo -e "${BLUE}🔨 Building scenario validator...${NC}"
if ! cargo build --bin scenario-validator --release; then
    echo -e "${RED}Error: Failed to build scenario validator${NC}" >&2
    exit 1
fi

# Prepare validator arguments
VALIDATOR_ARGS=""
if [ "$STRICT_MODE" = true ]; then
    VALIDATOR_ARGS="$VALIDATOR_ARGS --strict"
fi
if [ "$CHECK_ASSETS" = false ]; then
    VALIDATOR_ARGS="$VALIDATOR_ARGS --no-assets"
fi

# Function to run validation
run_validation() {
    echo -e "${BLUE}🔍 Running scenario validation...${NC}"
    
    if [ "$VERBOSE" = true ]; then
        echo "Command: ./target/release/scenario-validator $VALIDATOR_ARGS $*"
    fi
    
    if ./target/release/scenario-validator $VALIDATOR_ARGS "$@"; then
        echo -e "${GREEN}✅ Validation completed successfully!${NC}"
        return 0
    else
        echo -e "${RED}❌ Validation failed!${NC}" >&2
        return 1
    fi
}

# Watch mode implementation
if [ "$WATCH_MODE" = true ]; then
    echo -e "${YELLOW}👀 Watching for changes in scenario files...${NC}"
    echo "Press Ctrl+C to stop watching."
    
    # Check if inotifywait is available (Linux)
    if command -v inotifywait &> /dev/null; then
        while true; do
            run_validation "$@" || true
            echo -e "${BLUE}Waiting for changes...${NC}"
            inotifywait -r -e modify,create,delete assets/scenarios/ 2>/dev/null || true
            echo ""
        done
    # Check if fswatch is available (macOS)
    elif command -v fswatch &> /dev/null; then
        while true; do
            run_validation "$@" || true
            echo -e "${BLUE}Waiting for changes...${NC}"
            fswatch -1 assets/scenarios/ 2>/dev/null || true
            echo ""
        done
    else
        echo -e "${YELLOW}Warning: Watch mode requires inotifywait (Linux) or fswatch (macOS)${NC}"
        echo "Falling back to single validation run."
        run_validation "$@"
    fi
else
    # Single validation run
    run_validation "$@"
fi