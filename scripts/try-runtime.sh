#!/bin/bash

SCRIPT_DIR=$(dirname "$0")
ROOT_DIR=$SCRIPT_DIR/..

set -e

cargo build --release --locked --features try-runtime --workspace --exclude integration-tests
"$ROOT_DIR"/target/release/subsocial-collator try-runtime \
--runtime "$ROOT_DIR/target/release/wbuild/subsocial-parachain-runtime/subsocial_parachain_runtime.compact.compressed.wasm" \
--chain=dev on-runtime-upgrade --checks=all live --uri wss://para.subsocial.network:443
