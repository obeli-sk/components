#!/usr/bin/env bash
# Integration test for activity-docker.
# Starts a local obelisk server with deployment-local.toml applied.
# Usage: ./test/integration.sh
set -euo pipefail

cd "$(dirname "$0")/.."

PASS=0
FAIL=0
PREFIX="obelisk-docker:activity-docker"
SERVER_PID=""
SERVER_LOG="${TMPDIR:-/tmp}/activity-docker-obelisk-server-$$.log"

start_server() {
  echo "Starting Obelisk server..."
  just serve >"$SERVER_LOG" 2>&1 &
  SERVER_PID=$!

  local retries=0
  while ! obelisk component list >/dev/null 2>&1; do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
      echo "Obelisk server exited before becoming ready"
      cat "$SERVER_LOG"
      exit 1
    fi
    if [ "$retries" -ge 20 ]; then
      echo "Timed out waiting for Obelisk server to become ready"
      cat "$SERVER_LOG"
      exit 1
    fi
    sleep 0.5
    retries=$((retries + 1))
  done
}

# Submit an execution and return the JSON result.
# Sets $RESULT_JSON to the raw JSON output.
# Returns 0 if {"ok": ...}, 1 if {"error": ...} or unexpected output.
submit_json() {
  local ffqn="$1"
  shift
  RESULT_JSON=$(obelisk execution submit -f -j "$PREFIX/$ffqn" -- "$@" 2>/dev/null)
  if printf '%s' "$RESULT_JSON" | jq -e 'has("ok")' > /dev/null 2>&1; then
    return 0
  fi
  return 1
}

# Get the ok value as compact JSON string.
get_ok_value() {
  printf '%s' "$RESULT_JSON" | jq -c '.ok'
}

assert_ok() {
  local label="$1"
  shift
  if submit_json "$@"; then
    echo "PASS: $label -> $(get_ok_value)"
    PASS=$((PASS + 1))
  else
    echo "FAIL: $label -> $RESULT_JSON"
    FAIL=$((FAIL + 1))
  fi
}

assert_ok_eq() {
  local label="$1"
  local expected="$2"
  shift 2
  if submit_json "$@"; then
    local actual
    actual=$(get_ok_value)
    if [ "$actual" = "$expected" ]; then
      echo "PASS: $label"
      PASS=$((PASS + 1))
    else
      echo "FAIL: $label -> expected $expected, got $actual"
      FAIL=$((FAIL + 1))
    fi
  else
    echo "FAIL: $label -> $RESULT_JSON"
    FAIL=$((FAIL + 1))
  fi
}

TEST_VOL="obelisk-test-vol-$$"
TEST_NET="obelisk-test-net-$$"
TEST_CONTAINER="obelisk-test-ctr-$$"

cleanup() {
  echo "--- Cleanup ---"
  submit_json "containers.rm" "\"$TEST_CONTAINER\"" true 2>/dev/null || true
  submit_json "volumes.rm" "\"$TEST_VOL\"" 2>/dev/null || true
  submit_json "networks.rm" "\"$TEST_NET\"" 2>/dev/null || true
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

start_server

echo "=== Volumes ==="

assert_ok_eq "volume does not exist yet" "false" \
  "volumes.exists" "\"$TEST_VOL\""

assert_ok "volume create" \
  "volumes.create" "\"$TEST_VOL\""

assert_ok_eq "volume exists after create" "true" \
  "volumes.exists" "\"$TEST_VOL\""

assert_ok "volume create idempotent" \
  "volumes.create" "\"$TEST_VOL\""

assert_ok "volume rm" \
  "volumes.rm" "\"$TEST_VOL\""

assert_ok_eq "volume does not exist after rm" "false" \
  "volumes.exists" "\"$TEST_VOL\""

assert_ok "volume rm idempotent" \
  "volumes.rm" "\"$TEST_VOL\""

echo ""
echo "=== Networks ==="

assert_ok "network create" \
  "networks.create" "\"$TEST_NET\"" null

assert_ok "network create idempotent" \
  "networks.create" "\"$TEST_NET\"" null

assert_ok "network rm" \
  "networks.rm" "\"$TEST_NET\""

assert_ok "network rm idempotent" \
  "networks.rm" "\"$TEST_NET\""

echo ""
echo "=== Containers ==="

assert_ok "container run" \
  "containers.run" "\"$TEST_CONTAINER\"" "{\"image\":\"alpine:latest\",\"env\":[],\"cmd\":[\"sleep\",\"300\"],\"ports\":[],\"mounts\":[],\"network\":null}"

assert_ok "container inspect running" \
  "containers.inspect" "\"$TEST_CONTAINER\""

assert_ok "container list (running)" \
  "containers.list" true

assert_ok "container run idempotent" \
  "containers.run" "\"$TEST_CONTAINER\"" "{\"image\":\"alpine:latest\",\"env\":[],\"cmd\":[\"sleep\",\"300\"],\"ports\":[],\"mounts\":[],\"network\":null}"

assert_ok "container stop" \
  "containers.stop" "\"$TEST_CONTAINER\""

assert_ok "container stop idempotent" \
  "containers.stop" "\"$TEST_CONTAINER\""

assert_ok "container start" \
  "containers.start" "\"$TEST_CONTAINER\""

assert_ok "container stop again" \
  "containers.stop" "\"$TEST_CONTAINER\""

assert_ok "container rm" \
  "containers.rm" "\"$TEST_CONTAINER\"" false

assert_ok "container rm idempotent" \
  "containers.rm" "\"$TEST_CONTAINER\"" false

assert_ok_eq "container inspect after rm" "null" \
  "containers.inspect" "\"$TEST_CONTAINER\""

echo ""
echo "=== Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
