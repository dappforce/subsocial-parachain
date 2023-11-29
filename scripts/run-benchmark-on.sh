#!/usr/bin/env bash

set -e

SCRIPT_DIR=$(dirname "$0")
ROOT_DIR=$SCRIPT_DIR/..

if [[ -z $1 || -z $2 ]]; then
  echo "You have to specify the pallet name and the output file"
  echo "For example: ./run-benchmark-on.sh pallet_name ./pallets/name/src/weights.rs"
  exit 1
fi

PALLET_NAME="$1"
OUTPUT_FILE="$2"

"$ROOT_DIR"/target/release/subsocial-collator benchmark pallet \
  --chain=dev \
  --steps=50 \
  --repeat=20 \
  --pallet "$PALLET_NAME" \
  --extrinsic '*' \
  --execution=wasm \
  --wasm-execution=Compiled \
  --heap-pages=4096 \
  --output="$OUTPUT_FILE" \
  --template=./.maintain/weight-template.hbs
