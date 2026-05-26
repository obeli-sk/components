#!/bin/sh
# Checks if a Docker volume exists.
#
# Args: $1 = name (JSON string)
# stdout (exit 0): JSON bool (true/false)
# stdout (exit 1): JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)

if docker inspect --type volume "$name" >/dev/null 2>&1; then
  echo "true"
else
  echo "false"
fi
