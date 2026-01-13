#!/usr/bin/env bash

set -euo pipefail
cd "$(dirname "$0")/.."

find . -type f -name "obelisk-oci.toml" -printf '%P\n' | sort | while read -r file; do
  echo "# see https://github.com/obeli-sk/components/tree/main/$(dirname ${file})"
  awk '
    /^\[\[/ { start=1 }
    start { print "# " $0 }
  ' "$file"
  echo
done
