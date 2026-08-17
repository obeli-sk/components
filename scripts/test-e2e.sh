#!/usr/bin/env bash

# End-to-end tests for Obelisk components
# This script starts mock servers and runs integration tests against them.
#
# Usage: ./scripts/test-e2e.sh [component-name]
#   component-name: Optional. Run tests for specific component only.
#                   e.g., openai, http, github, fly

set -euo pipefail
cd "$(dirname "$0")/.."

# Ensure wasmtime is in PATH
export PATH="$HOME/.wasmtime/bin:$PATH"

# Configuration
MOCK_OPENAI_PORT=18080
MOCK_HTTP_PORT=18083

# PIDs of mock servers
MOCK_PIDS=()

cleanup() {
    echo "Cleaning up mock servers..."
    for pid in "${MOCK_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
}

trap cleanup EXIT

start_mock_server() {
    local name="$1"
    local script="$2"
    local port="$3"

    echo "Starting mock $name server on port $port..."
    python3 "$script" "$port" &
    MOCK_PIDS+=("$!")

    # Wait for server to be ready
    local retries=0
    while ! curl -s "http://127.0.0.1:$port" > /dev/null 2>&1; do
        if [[ $retries -ge 10 ]]; then
            echo "Failed to start mock $name server"
            exit 1
        fi
        sleep 0.5
        retries=$((retries + 1))
    done
    echo "Mock $name server is ready"
}

run_openai_tests() {
    echo "=== Running OpenAI e2e tests ==="
    start_mock_server "OpenAI" "./scripts/mocks/mock-openai-server.py" "$MOCK_OPENAI_PORT"

    export TEST_OPENAI_API_KEY="mock-api-key-for-testing"
    export TEST_OPENAI_API_BASE_URL="http://127.0.0.1:$MOCK_OPENAI_PORT/v1"

    (
        cd openai/activity-openai-responses
        cargo test --target wasm32-wasip2 -- --ignored --nocapture 2>&1
    )
}

run_http_tests() {
    echo "=== Running HTTP e2e tests ==="
    start_mock_server "HTTP" "./scripts/mocks/mock-http-server.py" "$MOCK_HTTP_PORT"

    export TEST_HTTP_BASE_URL="http://127.0.0.1:$MOCK_HTTP_PORT"

    (
        cd http/activity-http-generic
        cargo test --target wasm32-wasip2 -- --ignored --nocapture 2>&1
    )
}

run_github_tests() {
    echo "=== Running GitHub e2e tests ==="

    if [[ -z "${TEST_GITHUB_TOKEN:-}" ]]; then
        echo "TEST_GITHUB_TOKEN not set, skipping GitHub tests"
        return 0
    fi

    if [[ -z "${TEST_GITHUB_LOGIN:-}" ]]; then
        export TEST_GITHUB_LOGIN="obeli-sk"
    fi

    if [[ -z "${TEST_GITHUB_REPO:-}" ]]; then
        export TEST_GITHUB_REPO="https://github.com/obeli-sk/obelisk"
    fi

    (
        cd github/activity-github-graphql
        cargo test --target wasm32-wasip2 -- --ignored --nocapture 2>&1
    )
}

run_fly_tests() {
    echo "=== Running Fly.io e2e tests ==="

    if [[ -z "${TEST_FLY_API_TOKEN:-}" ]]; then
        echo "TEST_FLY_API_TOKEN not set, skipping Fly.io tests"
        return 0
    fi

    (
        cd fly/activity-fly-http
        cargo test --target wasm32-wasip2 -- --ignored --nocapture 2>&1
    )
}

# Main
COMPONENT="${1:-all}"

case "$COMPONENT" in
    openai)
        run_openai_tests
        ;;
    http)
        run_http_tests
        ;;
    github)
        run_github_tests
        ;;
    fly)
        run_fly_tests
        ;;
    all)
        run_openai_tests
        run_http_tests
        run_github_tests
        run_fly_tests
        ;;
    *)
        echo "Unknown component: $COMPONENT"
        echo "Valid options: openai, http, github, fly, all"
        exit 1
        ;;
esac

echo ""
echo "=== E2E tests completed ==="
