#!/usr/bin/env bash

set -e

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )/.."
BIN_PATH=$PROJECT_DIR/target/release/subsocial-collator

CUSTOM_SPEC_PATH=$PROJECT_DIR/res/custom-subsocial.json
RAW_SPEC_PATH=$PROJECT_DIR/res/subsocial.json

$BIN_PATH build-spec --chain staging --disable-default-bootnode > "$CUSTOM_SPEC_PATH"
$BIN_PATH build-spec --chain "$CUSTOM_SPEC_PATH"  --disable-default-bootnode --raw > "$RAW_SPEC_PATH"

rm "$CUSTOM_SPEC_PATH"
