#!/bin/sh
# Inspects a Docker container.
#
# Args: $1 = name (JSON string)
# stdout (exit 0): JSON option<record { id, state }> — object or null
# stdout (exit 1): JSON error string
set -eu

name=$(printf '%s' "$1" | jq -r .)

if inspect_json=$(docker inspect --type container "$name" 2>&1); then
  printf '%s' "$inspect_json" | jq '{id: .[0].Id, state: .[0].State.Status}'
else
  case "$inspect_json" in
    *"No such"*|*"Error: No such object"*)
      echo "null"
      exit 0
      ;;
  esac
  printf '%s' "$inspect_json" | jq -Rs .
  exit 1
fi
