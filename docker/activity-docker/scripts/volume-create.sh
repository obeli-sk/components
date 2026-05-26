#!/bin/sh
# Idempotently creates a Docker volume.
#
# Args: $1 = name (JSON string)
# stdout (exit 0): JSON string with volume name
# stdout (exit 1): JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)

# Idempotency check
if docker inspect --type volume "$name" >/dev/null 2>&1; then
  printf '%s' "$name" | jq -Rs .
  exit 0
fi

if ! output=$(docker volume create "$name" 2>&1); then
  printf '%s' "$output" | jq -Rs .
  exit 1
fi

printf '%s' "$name" | jq -Rs .
