#!/usr/bin/env bash
set -euo pipefail

fail() {
  local message=$1
  printf '%s\n' "$message" >&2
  jq -Rn --arg message "$message" '$message'
  exit 1
}

fail_permanent() {
  local message=$1
  printf 'permanent: %s\n' "$message" >&2
  jq -Rn --arg message "$message" '$message'
  exit 1
}

container_name=$(jq -er 'if type == "string" then . else error("must be a string") end' <<<"${1-}") \
  || fail_permanent "invalid JSON argument for container-name"
socket_path=$(jq -er 'if type == "string" then . else error("must be a string") end' <<<"${2-}") \
  || fail_permanent "invalid JSON argument for socket"

output=$(docker rm -f "$container_name" 2>&1) || {
  if [[ "$output" != *"No such container"* ]]; then
    fail "docker cleanup failed: $output"
  fi
}

rm -f "$socket_path"
printf 'null'
