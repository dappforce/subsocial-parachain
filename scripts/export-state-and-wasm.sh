#!/usr/bin/env bash

set -e

[[ -z $1 ]] && exit 1

PARACHAIN_ID=$1
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )/.."
BIN_PATH=$PROJECT_DIR/target/release/subsocial-collator

WASM_PATH=$PROJECT_DIR/para-$PARACHAIN_ID-wasm
GENESIS_PATH=$PROJECT_DIR/para-$PARACHAIN_ID-genesis

[[ -f $WASM_PATH ]] && rm -f $WASM_PATH
[[ -f $GENESIS_PATH ]] && rm -f $GENESIS_PATH

$BIN_PATH export-genesis-wasm --chain staging > "$WASM_PATH"
$BIN_PATH export-genesis-state --parachain-id $PARACHAIN_ID > "$GENESIS_PATH"
