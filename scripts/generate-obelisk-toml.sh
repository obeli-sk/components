#!/bin/bash
# Generates obelisk-generate.toml by collecting all obelisk-oci.toml files
# and commenting out their contents.
#
# Usage: ./scripts/generate-obelisk-toml.sh [output_file]
# Default output: obelisk-generate.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_FILE="${1:-obelisk-generate.toml}"

cd "$ROOT_DIR"

# Find all obelisk-oci.toml files and sort them
mapfile -t OCI_FILES < <(find . -name 'obelisk-oci.toml' -type f | sort)

# Clear/create output file
> "$OUTPUT_FILE"

first=true
for oci_file in "${OCI_FILES[@]}"; do
    # Get the directory path relative to root (e.g., fly/activity-fly-http)
    dir_path=$(dirname "$oci_file" | sed 's|^\./||')
    
    # Add blank line between sections (except before first)
    if [ "$first" = true ]; then
        first=false
    else
        echo >> "$OUTPUT_FILE"
    fi
    
    # Add the see comment
    echo "# see https://github.com/obeli-sk/components/tree/main/$dir_path" >> "$OUTPUT_FILE"
    
    # Process the file:
    # 1. Remove api.listening_addr and webui.listening_addr lines
    # 2. Prefix each line with "# " (including empty lines)
    first_content=""
    while IFS= read -r line || [[ -n "$line" ]]; do
        # Skip api.listening_addr and webui.listening_addr lines
        if [[ "$line" =~ ^api\.listening_addr ]] || [[ "$line" =~ ^webui\.listening_addr ]]; then
            continue
        fi
        
        # Skip empty lines that appear at the start (after the header lines we're removing)
        if [[ -z "$line" ]] && [[ "$first_content" != "seen" ]]; then
            continue
        fi
        first_content="seen"
        
        # Comment out the line
        if [[ -z "$line" ]]; then
            echo "#" >> "$OUTPUT_FILE"
        else
            echo "# $line" >> "$OUTPUT_FILE"
        fi
    done < "$oci_file"
done

echo "Generated $OUTPUT_FILE"
