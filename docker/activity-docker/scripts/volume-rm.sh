#!/bin/sh
# Removes a Docker volume. Idempotent: no-op if not found.
#
# Args: $1 = name (JSON string)
# exit 0: success, exit 1: stdout = JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)

if ! docker inspect --type volume "$name" >/dev/null 2>&1; then
  exit 0
fi

if ! output=$(docker volume rm "$name" 2>&1); then
  printf '%s' "$output" | jq -Rs .
  exit 1
fi
