#!/usr/bin/env bash

# Pushes all WASM components referred in current directory's obelisk-local.toml
# to the Docker Hub and creates or updates obelisk-oci.toml.
# Expects that all components are already built.
#
# Usage: just all-build all-push <tag>
# Examples:
# just all-push-dryrun
# just all-build all-push "$(date +%Y-%m-%d)"
set -exuo pipefail

TAG="$1"
PARENT=$(basename "$(dirname "$PWD")")
PREFIX="oci://docker.io/getobelisk/components_${PARENT}_"

SOURCE_TOML_FILE="obelisk-local.toml"
TARGET_TOML_FILE="obelisk-oci.toml"

push_component() {
    local LOCAL_DEPLOYMENT_TOML="$1"
    local COMPONENT_NAME="$2"

    OCI_LOCATION="${PREFIX}${COMPONENT_NAME}:${TAG}"
    if [ "$TAG" != "dryrun" ]; then
        obelisk component push --deployment "$LOCAL_DEPLOYMENT_TOML" "$COMPONENT_NAME" "$OCI_LOCATION"
    else
        echo "$OCI_LOCATION"
    fi
}

push_and_update() {
    local LOCAL_DEPLOYMENT_TOML="$1"
    local COMPONENT_NAME="$2"
    shift 2
    DST_TOML_FILES=("$@")

    OCI_LOCATION=$(push_component "$LOCAL_DEPLOYMENT_TOML" "$COMPONENT_NAME")

    for DST_TOML_FILE in "${DST_TOML_FILES[@]}"; do
        obelisk component add --deployment "$DST_TOML_FILE" "$OCI_LOCATION" "$COMPONENT_NAME"
    done
}

# Seed the target TOML from the source so that `obelisk component add` has
# existing entries to update.
cp "$SOURCE_TOML_FILE" "$TARGET_TOML_FILE"

# Push every component declared in the source TOML.
while IFS= read -r COMPONENT_NAME; do
    push_and_update "$SOURCE_TOML_FILE" "$COMPONENT_NAME" "$TARGET_TOML_FILE"
done < <(grep -E '^name = "' "$SOURCE_TOML_FILE" | sed -E 's/^name = "([^"]+)".*/\1/')

echo "All components pushed and TOML file updated successfully."
