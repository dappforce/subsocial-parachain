#!/usr/bin/env bash

set -e

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )/.."
BIN_PATH=$PROJECT_DIR/target/release/subsocial-collator

WASM_PATH=$PROJECT_DIR/para-888-wasm
GENESIS_PATH=$PROJECT_DIR/para-888-genesis

[[ -f $WASM_PATH ]] && rm -f $WASM_PATH
[[ -f $GENESIS_PATH ]] && rm -f $GENESIS_PATH

$BIN_PATH export-genesis-wasm --chain subsocial-latest > "$WASM_PATH"
$BIN_PATH export-genesis-state --parachain-id 888 > "$GENESIS_PATH"
