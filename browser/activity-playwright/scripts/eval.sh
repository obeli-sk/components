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

socket_path=$(jq -er 'if type == "string" then . else error("must be a string") end' <<<"${1-}") \
  || fail_permanent "invalid JSON argument for socket"
code_json=${2-}
jq -e 'type == "string"' >/dev/null <<<"$code_json" \
  || fail_permanent "invalid JSON argument for code"

# Secrets are optional: Obelisk writes a `{ "secrets": { ... } }` document to
# stdin only when the activity declares `secrets = [...]`. Default to an empty
# map when stdin is absent so callers that need no secrets work unchanged.
stdin_json=$(</dev/stdin || true)
if [ -z "${stdin_json//[[:space:]]/}" ]; then
  stdin_json='{}'
fi
secrets=$(jq -ce '.secrets // {}' <<<"$stdin_json") \
  || fail_permanent "invalid secret document on stdin"

request=$(jq -cn --argjson secrets "$secrets" --argjson code "$code_json" \
  '{secrets: $secrets, code: $code}') \
  || fail_permanent "failed to build request"

response=$(
  printf '%s\n' "$request" \
    | socat STDIO,ignoreeof "UNIX-CONNECT:$socket_path" \
    | head -n 1
) || true
if [ -z "$response" ]; then
  fail "cannot communicate with Playwright socket $socket_path"
fi

if ! jq -e 'type == "object" and (.ok | type == "boolean")' >/dev/null <<<"$response"; then
  fail "invalid Playwright socket response"
fi

if [ "$(jq -r '.ok' <<<"$response")" != "true" ]; then
  fail "$(jq -r '.error // "Playwright evaluation failed"' <<<"$response")"
fi

# Return the page-eval result as a JSON-encoded string (the ok arm of
# result<string, string>); callers JSON.parse it to recover the value.
jq -c '.result | @json' <<<"$response"
