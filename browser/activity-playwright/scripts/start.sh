#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="docker.io/getobelisk/components-playwright:latest"
VNC_CONTAINER_PORT="5900/tcp"

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

json_arg() {
  local raw=${1-}
  local name=$2
  if [ -z "$raw" ]; then
    fail_permanent "missing argument: $name"
  fi
  jq -er 'if type == "string" then . else error("must be a string") end' <<<"$raw" \
    || fail_permanent "invalid JSON argument for $name"
}

container_name=$(json_arg "${1-}" "container-name")
socket_path=$(json_arg "${2-}" "socket")
url=$(json_arg "${3-}" "url")

use_host_network=false
if [[ "$url" =~ ^https?://(localhost|127\.0\.0\.1)(:[0-9]+)?(/|$) ]]; then
  use_host_network=true
fi

socket_dir=$(dirname "$socket_path")
socket_base=$(basename "$socket_path")
mkdir -p -m 700 "$socket_dir"
rm -f "$socket_path"
docker rm -f "$container_name" >/dev/null 2>&1 || true

docker_args=(
  run
  --detach
  --name "$container_name"
  --user "$(id -u):$(id -g)"
  --mount "type=bind,src=$socket_dir,dst=/sockets"
)

if [ "$use_host_network" = "true" ]; then
  docker_args+=(--network host)
elif [ "${HEADED:-}" = "true" ]; then
  docker_args+=(-p "127.0.0.1::5900")
fi

docker_args+=(
  -e "HEADED=${HEADED:-false}"
  -e "IGNORE_HTTPS_ERRORS=${IGNORE_HTTPS_ERRORS:-false}"
  "$IMAGE_NAME"
  "$(jq -cn --arg value "/sockets/$socket_base" '$value')"
  "$(jq -cn --arg value "$url" '$value')"
)

printf 'Starting Playwright container %s with headed=%s host_network=%s\n' \
  "$container_name" "${HEADED:-false}" "$use_host_network" >&2
container_id=$(docker "${docker_args[@]}") || fail "docker failed to start Playwright"

for _ in $(seq 1 120); do
  if [ -S "$socket_path" ]; then
    break
  fi
  if [ "$(docker inspect "$container_name" --format '{{.State.Running}}' 2>/dev/null || true)" != "true" ]; then
    logs=$(docker logs "$container_name" 2>&1 || true)
    fail "Playwright container stopped before creating its socket: $logs"
  fi
  sleep 0.25
done

if [ ! -S "$socket_path" ]; then
  fail "timed out waiting for Playwright socket $socket_path"
fi

vnc_port=null
if [ "${HEADED:-}" = "true" ]; then
  if [ "$use_host_network" = "true" ]; then
    vnc_port=5900
  else
    mapping=$(docker port "$container_name" "$VNC_CONTAINER_PORT" 2>/dev/null | head -1 || true)
    if [[ "$mapping" =~ :([0-9]+)$ ]]; then
      vnc_port=${BASH_REMATCH[1]}
    fi
  fi
  if [ "$vnc_port" != "null" ]; then
    printf 'Playwright VNC is available at 127.0.0.1:%s\n' "$vnc_port" >&2
  fi
fi

jq -cn \
  --arg container "$container_name" \
  --arg image "$IMAGE_NAME" \
  --arg socket "$socket_path" \
  --arg id "$container_id" \
  --argjson vncport "$vnc_port" \
  '{container: $container, image: $image, socket: $socket, id: $id, vncport: $vncport}'
