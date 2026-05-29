#!/bin/sh
# Removes a Docker container. Idempotent: no-op if not found.
#
# Args: $1 = name (JSON string), $2 = force (JSON bool)
# exit 0: success, exit 1: stdout = JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)
force=$(printf '%s' "$2" | jq -r .)

if ! docker inspect --type container "$name" >/dev/null 2>&1; then
  exit 0
fi

args="rm"
if [ "$force" = "true" ]; then
  args="$args -f"
fi
args="$args $name"

# shellcheck disable=SC2086
if ! output=$(docker $args 2>&1); then
  printf '%s' "$output" | jq -Rs .
  exit 1
fi
