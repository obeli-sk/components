#!/bin/sh
# Lists Docker containers.
#
# Args: $1 = all (JSON bool)
# stdout (exit 0): JSON list<record { id, name, image, state, status }>
# stdout (exit 1): JSON error string
set -eu

all=$(printf '%s' "$1" | jq -r .)

if [ "$all" = "true" ]; then
  all_flag="-a"
else
  all_flag=""
fi

# shellcheck disable=SC2086
if ! output=$(docker ps --format '{{json .}}' $all_flag 2>&1); then
  printf '%s' "$output" | jq -Rs .
  exit 1
fi

# docker ps --format json outputs one JSON object per line
printf '%s' "$output" | jq -s '[.[] | {id: .ID, name: .Names, image: .Image, state: .State, status: .Status}]'
