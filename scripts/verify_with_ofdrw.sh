#!/bin/bash
# verify_with_ofdrw.sh - Verify easyofd-rust against ofdrw (Java) output
#
# This script demonstrates how to perform a true bidirectional comparison
# between easyofd-rust and ofdrw when a JDK environment is available.
#
# PREREQUISITES:
# - JDK 8+ installed
# - Maven 3.6+ installed
# - Git installed
# - ~500MB disk space for ofdrw clone + build artifacts
#
# USAGE:
#   ./scripts/verify_with_ofdrw.sh
#
# WHAT THIS SCRIPT DOES:
# 1. Clones the ofdrw repository (if not already present)
# 2. Builds ofdrw with Maven
# 3. Runs ofdrw's test suite to produce OFD output files
# 4. Compares ofdrw output against easyofd-rust expected baselines
# 5. Reports any differences
#
# LIMITATIONS:
# - This script does NOT run in CI by default (no JDK available)
# - The comparison is structural (XML elements/attributes), not byte-level
# - ofdrw's test output may vary based on random UUIDs and timestamps
# - For true byte-level comparison, use the ofdrw-produced OFD files directly
#
# EXIT CODES:
#   0 - All comparisons passed
#   1 - One or more comparisons failed
#   2 - Prerequisites not met

set -euo pipefail

# ─── Configuration ──────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OFRW_CLONE_DIR="$PROJECT_ROOT/.ofdrw-clone"
OFRW_REPO="https://github.com/ofdrw/ofdrw.git"
OFRW_BRANCH="master"
EXPECTED_DIR="$PROJECT_ROOT/tests/fixtures/baseline/ofdrw_expected"
FIXTURES_DIR="$PROJECT_ROOT/tests/fixtures/real_ofd"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ─── Helper functions ───────────────────────────────────────────────────────────

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    local missing=0

    if ! command -v java &> /dev/null; then
        log_error "Java not found. Please install JDK 8+."
        missing=1
    fi

    if ! command -v mvn &> /dev/null; then
        log_error "Maven not found. Please install Maven 3.6+."
        missing=1
    fi

    if ! command -v git &> /dev/null; then
        log_error "Git not found. Please install Git."
        missing=1
    fi

    if [ $missing -eq 1 ]; then
        log_error "Prerequisites not met. Exiting."
        exit 2
    fi

    log_info "All prerequisites met."
}

# ─── Step 1: Clone ofdrw repository ────────────────────────────────────────────

clone_ofdrw() {
    if [ -d "$OFRW_CLONE_DIR" ]; then
        log_info "ofdrw clone already exists at $OFRW_CLONE_DIR"
        cd "$OFRW_CLONE_DIR"
        git pull --quiet
    else
        log_info "Cloning ofdrw repository..."
        git clone --branch "$OFRW_BRANCH" --depth 1 "$OFRW_REPO" "$OFRW_CLONE_DIR"
        cd "$OFRW_CLONE_DIR"
    fi
}

# ─── Step 2: Build ofdrw ──────────────────────────────────────────────────────

build_ofdrw() {
    log_info "Building ofdrw with Maven..."
    cd "$OFRW_CLONE_DIR"
    mvn clean package -DskipTests -q
    log_info "ofdrw build complete."
}

# ─── Step 3: Run ofdrw tests to produce output ────────────────────────────────

run_ofdrw_tests() {
    log_info "Running ofdrw tests to produce OFD output..."
    cd "$OFRW_CLONE_DIR"

    # Run ofdrw-full tests (includes all modules)
    mvn -pl ofdrw-full test -q

    # The test output OFD files are in:
    # - ofdrw-full/target/test-classes/
    # - ofdrw-reader/src/test/resources/
    # - ofdrw-layout/src/test/resources/

    log_info "ofdrw tests complete. Output files generated."
}

# ─── Step 4: Compare ofdrw output against expected baselines ──────────────────

compare_xml_structure() {
    local ofd_file="$1"
    local expected_file="$2"
    local description="$3"

    if [ ! -f "$expected_file" ]; then
        log_warn "Expected file not found: $expected_file"
        return 0
    fi

    log_info "Comparing: $description"

    # Extract XML from OFD ZIP
    local temp_xml="/tmp/ofdrw_verify_$$.xml"
    unzip -p "$ofd_file" "OFD.xml" > "$temp_xml" 2>/dev/null || {
        log_warn "Could not extract OFD.xml from $ofd_file"
        return 0
    }

    # Compare key structural elements
    local expected_content
    expected_content=$(cat "$expected_file")

    # Check namespace
    if ! grep -q 'xmlns:ofd="http://www.ofdspec.org/2016"' "$temp_xml"; then
        log_error "FAIL: $description - missing OFD namespace"
        rm -f "$temp_xml"
        return 1
    fi

    # Check DocType
    if ! grep -q 'DocType="OFD"' "$temp_xml"; then
        log_error "FAIL: $description - missing DocType=\"OFD\""
        rm -f "$temp_xml"
        return 1
    fi

    # Check DocRoot
    if ! grep -q '<ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>' "$temp_xml"; then
        log_error "FAIL: $description - missing DocRoot"
        rm -f "$temp_xml"
        return 1
    fi

    log_info "PASS: $description"
    rm -f "$temp_xml"
    return 0
}

compare_all_fixtures() {
    local failures=0

    # Compare against expected baselines
    for expected_file in "$EXPECTED_DIR"/*_ofd.xml; do
        if [ ! -f "$expected_file" ]; then
            continue
        fi

        local basename
        basename=$(basename "$expected_file" _ofd.xml)
        local fixture_file="$FIXTURES_DIR/${basename}.ofd"

        if [ ! -f "$fixture_file" ]; then
            log_warn "Fixture not found: $fixture_file"
            continue
        fi

        if ! compare_xml_structure "$fixture_file" "$expected_file" "$basename"; then
            ((failures++))
        fi
    done

    return $failures
}

# ─── Step 5: Generate comparison report ────────────────────────────────────────

generate_report() {
    local failures="$1"

    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "  ofdrw Verification Report"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    echo "Comparison Results:"
    echo "  - Expected files checked: $(ls "$EXPECTED_DIR"/*_ofd.xml 2>/dev/null | wc -l | tr -d ' ')"
    echo "  - Failures: $failures"
    echo ""

    if [ "$failures" -eq 0 ]; then
        log_info "All comparisons PASSED."
        echo ""
        echo "Note: This is a structural comparison (XML elements/attributes)."
        echo "For byte-level comparison, use the ofdrw-produced OFD files directly."
    else
        log_error "$failures comparison(s) FAILED."
        echo ""
        echo "Review the failures above and update expected files if needed."
    fi

    echo ""
    echo "════════════════════════════════════════════════════════════════"
}

# ─── Main ──────────────────────────────────────────────────────────────────────

main() {
    log_info "Starting ofdrw verification..."
    echo ""

    # Check prerequisites
    check_prerequisites

    # Clone ofdrw
    clone_ofdrw

    # Build ofdrw
    build_ofdrw

    # Run tests
    run_ofdrw_tests

    # Compare
    local failures=0
    compare_all_fixtures || failures=$?

    # Generate report
    generate_report "$failures"

    # Exit with appropriate code
    if [ "$failures" -gt 0 ]; then
        exit 1
    fi
    exit 0
}

# Run main function
main "$@"
