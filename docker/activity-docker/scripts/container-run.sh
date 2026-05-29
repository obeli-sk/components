#!/bin/sh
# Runs a Docker container. Idempotent: if the container already exists and is
# running, returns its ID.
#
# Args: $1 = name (JSON string), $2 = config (JSON object)
# stdout (exit 0): JSON string with container ID
# stdout (exit 1): JSON string with error message
set -eu

name=$(printf '%s' "$1" | jq -r .)
config=$2

image=$(printf '%s' "$config" | jq -r '.image')
env_list=$(printf '%s' "$config" | jq -r '.env // [] | .[] | "-e\n\(.[0])=\(.[1])"')
cmd_args=$(printf '%s' "$config" | jq -r '.cmd // [] | .[]')
network=$(printf '%s' "$config" | jq -r '.network // empty')

args="run -d --name $name"

# Environment
if [ -n "$env_list" ]; then
  args="$args $(printf '%s' "$config" | jq -r '[.env // [] | .[] | "-e", "\(.[0])=\(.[1])"] | join(" ")')"
fi

# Ports
port_args=$(printf '%s' "$config" | jq -r '[.ports // [] | .[] | "-p", "\(.["host-port"]):\(.["container-port"])/\(.protocol)"] | join(" ")')
if [ -n "$port_args" ]; then
  args="$args $port_args"
fi

# Mounts
mount_args=$(printf '%s' "$config" | jq -r '[.mounts // [] | .[] | "-v", "\(.source):\(.target):\(if .readonly then "ro" else "rw" end)"] | join(" ")')
if [ -n "$mount_args" ]; then
  args="$args $mount_args"
fi

# Network
if [ -n "$network" ]; then
  args="$args --network $network"
fi

args="$args $image"

# Command
if [ -n "$cmd_args" ]; then
  args="$args $cmd_args"
fi

# shellcheck disable=SC2086
if output=$(docker $args 2>&1); then
  # Trim and return container ID
  printf '%s' "$output" | tr -d '[:space:]' | jq -Rs .
  exit 0
fi

# Check if conflict (already exists)
case "$output" in
  *"Conflict"*|*"is already in use"*)
    # Idempotency: check if running
    if inspect_json=$(docker inspect --type container "$name" 2>/dev/null); then
      state=$(printf '%s' "$inspect_json" | jq -r '.[0].State.Status')
      if [ "$state" = "running" ]; then
        printf '%s' "$inspect_json" | jq -r '.[0].Id' | jq -Rs .
        exit 0
      fi
      printf '%s' "Container '$name' exists but is in state '$state'. Use 'start' to resume or 'rm' to replace." | jq -Rs .
      exit 1
    fi
    ;;
esac

printf '%s' "$output" | jq -Rs .
exit 1
