#!/bin/bash

# --help for usage
if [[ "$1" == "--help" ]]; then
    echo ""
    echo "Usage: $0 [bucket-url] [asset] [root] [cache] [directory] [target-triplet] [variant](optional)"
    echo ""
    echo "  bucket-url: base of target-specific asset url"
    echo "  asset: the asset to retrieve (e.g. 'opencv', 'onnxruntime', 'roc')"
    echo "      (assumes archive file <asset>.tar.gz)"
    echo "  root: the base path to <cache> and <directory>"
    echo "  cache: the relative path for storing asset and target-specific archive files"
    echo "  directory: the relative path of the source url and destination extract location"
    echo "  target-triplet: aarch64-unknown-linux-gnu, x86_64-unknown-linux-gnu, aarch64-apple-darwin, etc."
    echo "      (defaults to current system triplet if unsupplied)"
    echo "  variant: only for testing, string of digits to force retrieval of a variant-specific build."
    echo "      (pass '35' for Jetpack 5, or '36' for Jetpack 6)"
    echo ""
    echo "**Note**: aarch64-unknown-linux-gnu treated as a Jetson device, variant by argument or local Jetpack version."
    exit 0
fi

# ------------------------------
# constants
# ------------------------------

FILE_EXTENSION=".tar.gz"
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd ../ && pwd)
CFG=$(cat $SCRIPT_DIR/config/target.json | jq)

# ------------------------------
# setup
# ------------------------------

function check_remote_data() {
    local url=$1
    local output
    output=$(wget -S --spider --max-redirect=5 "$url" 2>&1)
    local status=$?
    echo "$output" >&2
    return $status
}

function directory_create_recursive() {
    local directory=$1
    if [[ ! -d "$directory" ]]; then
        mkdir -p "$directory"
    fi
}

function extract_data() {
    local path=$1
    local directory=$2
    echo "--archive: $path"
    echo "--extract: $directory"
    tar -xf "$path" -C "$directory"
}

function operating_system() {
    TARGET_OS=$(triplet_map_key "$1" "os")
    if [[ "$TARGET_OS" == "androideabi" ]]; then
        TARGET_OS="android"
    fi
    echo "$TARGET_OS"
}

function remote_data() {
    local url=$1
    local path=$2
    if ! check_remote_data "$url"; then
        echo "--invalid url: $url"
        exit 1
    fi
    echo "--asset url: $url"
    wget -O "$path" "$url"
}

function split_string() {
    local string="$1"
    local delimiter="$2"
    local array=()
    IFS="$delimiter" read -r -a array <<< "$string"
    for item in "${array[@]}"; do
        echo "$item"
    done
}

function target_triplet_map() {
    # check that at least 3 "-" are present
    if [[ "$1" != *-*-* ]]; then
        echo "Invalid target triplet: $1"
        exit 1
    fi
    # split string by "-"
    SPLIT=($(split_string "$1" "-"))
    # first item is architecture, second is vendor, third is os
    ARCHITECTURE="${SPLIT[0]}"
    VENDOR="${SPLIT[1]}"
    OS="${SPLIT[2]}"
    # if fourth item is present, it is environment
    ENVIRONMENT=""
    if [[ ${#SPLIT[@]} -ge 4 ]]; then
        ENVIRONMENT="${SPLIT[3]}"
    fi
    echo "{\"architecture\": \"$ARCHITECTURE\", \"vendor\": \"$VENDOR\", \"os\": \"$OS\", \"environment\": \"$ENVIRONMENT\"}"
}

function triplet_map_key() {
    key_name=$2
    TARGET_VALUE=$(echo $1 | jq -r ".$key_name")
    if [[ "$TARGET_VALUE" == "null" ]]; then
        echo "Invalid $key_name"
        exit 1
    fi
    echo "$TARGET_VALUE"
}

function triplet_map_key_check() {
    value=$1
    array=($2)
    descr=$3
    CHECK=$([[ " ${array[*]} " =~ " ${value} " ]] && echo 1 || echo 0)
    if [[ "$CHECK" == 0 ]]; then
        echo "Invalid $descr: $value"
        exit 1
    fi
}

function variant() {
    local target_variant=$1
    local os=$2
    local arch=$2
    if [[ "$TARGET_ARCH" == "aarch64" ]] && [[ "$TARGET_OS" == "linux" ]]; then
        # determine variant if target_variant is empty
        if [[ "$target_variant" == "" ]]; then
            # check that dpkg-query is available
            local dpkg_query=$(command -v dpkg-query)
            if ! [[ -x "$dpkg_query" ]]; then
                echo "command 'dpkg-query' not found"
                exit 1
            fi
            # retrieve kernel version
            local jetson_l4t_string=$(dpkg-query --showformat='${Version}' --show nvidia-l4t-core)
            local target_variant=$(echo "$jetson_l4t_string" | cut -f 1 -d '.')
        fi
        # check that variant (release number) is either 35 or 36
        if [[ "$target_variant" == "35" ]] || [[ "$target_variant" == "36" ]]; then
            echo "$target_variant"
        else
            echo "unknown jetpack version: $target_variant"
            exit 1
        fi
    else
        echo ""
    fi
}

function variant_check() {
    # variant must be either an empty string or a string of digits with periods
    if [[ "$1" != "" ]] && ! [[ "$1" =~ ^[0-9.]+$ ]]; then
        echo "Failed to determine variant: $1"
        exit 1
    fi
}

# ------------------------------
# execution
# ------------------------------

# arguments
BUCKET_URL=$1
ASSET=$2
ROOT=$3
CACHE=$4
DIRECTORY=$5
TARGET_TRIPLET=$6
VARIANT=$7

# architecture, vendor, os, environment
TARGET_TRIPLET_MAP=$(target_triplet_map "$TARGET_TRIPLET")
TARGET_OS=$(operating_system "$TARGET_TRIPLET_MAP")
TARGET_ARCH=$(triplet_map_key "$TARGET_TRIPLET_MAP" "architecture")
TARGET_ENVIRONMENT=$(triplet_map_key "$TARGET_TRIPLET_MAP" "environment")

# check that operating system is valid, retrieve builds
OPERATING_SYSTEMS=$(echo "$CFG" | jq -r -c 'keys[]')
triplet_map_key_check "$TARGET_OS" "$OPERATING_SYSTEMS" "operating system"
BUILDS=$(echo "$CFG" | jq -r -c ".${TARGET_OS}.build")

# filter builds by architecture
TARGET_BUILDS=$(echo "$BUILDS" | jq -r -c "[.[] | select(.architecture == \"$TARGET_ARCH\")]")

# filter builds by environment
TARGET_BUILDS=$(echo "$TARGET_BUILDS" | jq -r -c "[.[] | select(.environment == \"$TARGET_ENVIRONMENT\")]")

# if relevant (i.e. arm64 linux, jetpack), filter builds by variant
TARGET_VARIANT=$(variant "$VARIANT" "$TARGET_OS" "$TARGET_ARCH")
variant_check "$TARGET_VARIANT"
TARGET_BUILDS=$(echo "$TARGET_BUILDS" | jq -r -c "[.[] | select(.variant == \"$TARGET_VARIANT\")]")

# if number of TARGET_BUILDS isn't 1, then exit
NUM_TARGET_BUILDS=$(echo "$TARGET_BUILDS" | jq -r -c '. | length')
case $NUM_TARGET_BUILDS in
    1)
        ;;
    *)
        echo "More than one matching build: $TARGET_BUILDS"
        exit 1
    ;;
esac

TARGET_BUILD=$(echo "$TARGET_BUILDS" | jq -r -c ".[]")
TARGET_ARCH=$(echo "$TARGET_BUILD" | jq -r -c ".architecture_alias")
TARGET_ENVIRONMENT=$(echo "$TARGET_BUILD" | jq -r -c ".environment_alias")
TARGET_VARIANT=$(echo "$TARGET_BUILD" | jq -r -c ".variant_alias")

# source url of asset archive file and destination path
ASSET_URL="$BUCKET_URL/$DIRECTORY/$ASSET/$TARGET_OS/$TARGET_ARCH"
ASSET_PATH="$ROOT/$CACHE/$DIRECTORY/$ASSET/$TARGET_OS/$TARGET_ARCH"
if [[ "$TARGET_ENVIRONMENT" != "" ]]; then
    ASSET_URL="$ASSET_URL/$TARGET_ENVIRONMENT"
    ASSET_PATH="$ASSET_PATH/$TARGET_ENVIRONMENT"
fi
if [[ "$TARGET_VARIANT" != "" ]]; then
    ASSET_URL="$ASSET_URL/$TARGET_VARIANT"
    ASSET_PATH="$ASSET_PATH/$TARGET_VARIANT"
fi
directory_create_recursive "$ASSET_PATH"
ASSET_URL="$ASSET_URL/$ASSET$FILE_EXTENSION"
ASSET_PATH="$ASSET_PATH/$ASSET$FILE_EXTENSION"

# Only re-download if the asset(s) don't already exist
if [[ ! -f "$ASSET_PATH" ]]; then
    # download the asset
    remote_data "$ASSET_URL" "$ASSET_PATH"
    # error out if the asset doesn't exist
    if [[ ! -f "$ASSET_PATH" ]]; then
        echo "--missing asset: $ASSET_PATH"
        exit 1
    fi
fi

# Extract archive
EXTRACT_PATH=$ROOT/$DIRECTORY/$ASSET
directory_create_recursive "$EXTRACT_PATH"
extract_data "$ASSET_PATH" "$EXTRACT_PATH"

# ------------------------------
# Teardown
# ------------------------------

# nothing to do
exit 0
