#!/usr/bin/env bash

set -euo pipefail
cd "$(dirname "$0")/.."

find . -type f -name "obelisk-oci.toml" | sort | while read -r file; do
  echo "# see ${file}"
  awk '
    /^\[\[/ { start=1 }
    start { print "# " $0 }
  ' "$file"
  echo
done
