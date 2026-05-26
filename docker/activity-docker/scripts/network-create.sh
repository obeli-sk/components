#!/bin/sh
# Idempotently creates a Docker network.
#
# Args: $1 = name (JSON string), $2 = driver (JSON option<string> — string or null)
# stdout (exit 0): JSON string with network name/ID
# stdout (exit 1): JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)
driver=$(printf '%s' "$2" | jq -r '. // empty')

# Idempotency check
if docker inspect --type network "$name" >/dev/null 2>&1; then
  printf '%s' "$name" | jq -Rs .
  exit 0
fi

args="network create"
if [ -n "$driver" ]; then
  args="$args --driver $driver"
fi
args="$args $name"

# shellcheck disable=SC2086
if output=$(docker $args 2>&1); then
  printf '%s' "$output" | tr -d '[:space:]' | jq -Rs .
else
  printf '%s' "$output" | jq -Rs .
  exit 1
fi
