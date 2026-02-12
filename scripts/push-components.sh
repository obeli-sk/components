#!/usr/bin/env bash

# Pushes all WASM components referred in current directory's obelisk-local.toml
# to the Docker Hub and creates or updates obelisk-oci.toml
# Expects that all components are already built.
set -exuo pipefail

OBELISK_TOML_DIR_VALUE="${PWD}"
PARENT=$(basename "$(dirname "$PWD")")
PREFIX="docker.io/getobelisk/components_${PARENT}_"
TAG="$1"

SOURCE_TOML_FILE="obelisk-local.toml"
TARGET_TOML_FILE="obelisk-oci.toml"
# Determine COMPONENT_TYPE from the current directory name prefix
DIR_NAME=$(basename "$PWD")
if [[ "$DIR_NAME" == activity-* ]]; then
    COMPONENT_TYPE="activity_wasm"
elif [[ "$DIR_NAME" == webhook-* ]]; then
    COMPONENT_TYPE="webhook_endpoint"
elif [[ "$DIR_NAME" == workflow-* ]]; then
    COMPONENT_TYPE="workflow"
else
    echo "Error: directory '${DIR_NAME}' does not start with a known prefix (activity-, webhook-, workflow-)" >&2
    exit 1
fi

push() {
    RELATIVE_PATH=$1
    FILE_NAME_WITHOUT_EXT=$(basename "$RELATIVE_PATH" | sed 's/\.[^.]*$//')
    OCI_LOCATION="${PREFIX}${FILE_NAME_WITHOUT_EXT}:${TAG}"
    echo "Pushing ${RELATIVE_PATH} to ${OCI_LOCATION}..."
    if [ "$TAG" != "dryrun" ]; then
        OUTPUT=$(obelisk component push "$RELATIVE_PATH" "$OCI_LOCATION")
    else
        OUTPUT="dryrun"
    fi
    # Replace the old location with the actual OCI location
    obelisk component add ${COMPONENT_TYPE} ${OUTPUT} --name ${FILE_NAME_WITHOUT_EXT} -c $TARGET_TOML_FILE
}

cp "$SOURCE_TOML_FILE" "$TARGET_TOML_FILE"


while IFS= read -r line; do
  [[ $line != location\ =\ * ]] && continue

  # extract quoted path
  raw_path=${line#*\"}
  raw_path=${raw_path%\"*}

  # interpolate ${OBELISK_TOML_DIR}
  path=${raw_path//\$\{OBELISK_TOML_DIR\}/$OBELISK_TOML_DIR_VALUE}

  push $path

done < "$TARGET_TOML_FILE"


echo "All components pushed and TOML file updated successfully."
