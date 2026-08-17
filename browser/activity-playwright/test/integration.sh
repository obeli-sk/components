#!/usr/bin/env bash
# Integration test for activity-playwright.
# Starts a local Obelisk server, drives start -> eval -> cleanup against a
# data: URL, and checks the page result. Requires docker, jq, socat, and the
# components-playwright image (run `just build-image` first).
# Usage: ./test/integration.sh
set -euo pipefail

cd "$(dirname "$0")/.."

PREFIX="obelisk-browser:activity-playwright"
SERVER_PID=""
SERVER_LOG="${TMPDIR:-/tmp}/activity-playwright-obelisk-server-$$.log"
CONTAINER="obelisk-pw-test-$$"
SOCK="${TMPDIR:-/tmp}/obelisk-pw-test-$$/browser.sock"
PASS=0
FAIL=0

start_server() {
  echo "Starting Obelisk server..."
  just serve >"$SERVER_LOG" 2>&1 &
  SERVER_PID=$!
  local retries=0
  while ! obelisk component list >/dev/null 2>&1; do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
      echo "Obelisk server exited before becoming ready"; cat "$SERVER_LOG"; exit 1
    fi
    if [ "$retries" -ge 20 ]; then
      echo "Timed out waiting for Obelisk server"; cat "$SERVER_LOG"; exit 1
    fi
    sleep 0.5; retries=$((retries + 1))
  done
}

submit() {
  local ffqn="$1"; shift
  obelisk execution submit -f -j "$PREFIX/$ffqn" -- "$@" 2>/dev/null
}

cleanup() {
  echo "--- Cleanup ---"
  submit "browser.cleanup" "\"$CONTAINER\"" "\"$SOCK\"" >/dev/null 2>&1 || true
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

start_server

echo "=== start ==="
start_json=$(submit "browser.start" "\"$CONTAINER\"" "\"$SOCK\"" '"data:text/html,<title>hi</title>"')
if printf '%s' "$start_json" | jq -e '.ok.socket == "'"$SOCK"'"' >/dev/null 2>&1; then
  echo "PASS: start -> $(printf '%s' "$start_json" | jq -c '.ok')"; PASS=$((PASS + 1))
else
  echo "FAIL: start -> $start_json"; FAIL=$((FAIL + 1))
fi

echo "=== eval ==="
eval_json=$(submit "browser.eval" "\"$SOCK\"" '"return await page.title()"')
# ok is the page result JSON-encoded as a string, i.e. "\"hi\"".
if [ "$(printf '%s' "$eval_json" | jq -r '.ok // empty' | jq -r . 2>/dev/null)" = "hi" ]; then
  echo "PASS: eval title -> $(printf '%s' "$eval_json" | jq -c '.ok')"; PASS=$((PASS + 1))
else
  echo "FAIL: eval title -> $eval_json"; FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
[ "$FAIL" -eq 0 ]
