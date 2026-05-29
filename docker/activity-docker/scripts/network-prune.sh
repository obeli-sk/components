#!/bin/sh
# Prunes unused Docker networks.
#
# No args.
# exit 0: success, exit 1: stdout = JSON error string
set -eu

if ! output=$(docker network prune -f 2>&1); then
  printf '%s' "$output" | jq -Rs .
  exit 1
fi
