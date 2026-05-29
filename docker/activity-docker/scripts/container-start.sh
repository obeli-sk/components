#!/bin/sh
# Starts an existing Docker container. Idempotent: no-op if already running.
#
# Args: $1 = name (JSON string)
# exit 0: success, exit 1: stdout = JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)

# Check existence and state
if inspect_json=$(docker inspect --type container "$name" 2>/dev/null); then
  state=$(printf '%s' "$inspect_json" | jq -r '.[0].State.Status')
  if [ "$state" = "running" ]; then
    exit 0
  fi
else
  printf '%s' "Container '$name' not found" | jq -Rs .
  exit 1
fi

if ! output=$(docker start "$name" 2>&1); then
  printf '%s' "$output" | jq -Rs .
  exit 1
fi
