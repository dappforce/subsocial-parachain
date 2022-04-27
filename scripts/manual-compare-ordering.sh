#!/usr/bin/env bash

set -e

ROOT=$(git rev-parse --show-toplevel)
OUTPUT_PATH="$ROOT/output.txt"

CHAIN=dev
REF_URL=wss://para.subsocial.network
VERSION=$("$ROOT"/target/release/subsocial-collator --version)

PID=$("$ROOT"/target/release/subsocial-collator --chain=dev --ws-port 8844 --tmp -- --dev > /dev/null 2>&1 & echo $!)

echo "Metadata comparison:" >> "$OUTPUT_PATH"
echo "Date: $(date)" >> "$OUTPUT_PATH"
echo "Reference: $REF_URL" >> "$OUTPUT_PATH"
echo "Target version: $VERSION" >> "$OUTPUT_PATH"
echo "Chain: $CHAIN" >> "$OUTPUT_PATH"
echo "----------------------------------------------------------------------" >> "$OUTPUT_PATH"

docker pull jacogr/polkadot-js-tools

CMD="docker run --pull always --network host jacogr/polkadot-js-tools metadata $REF_URL ws://localhost:8844"

echo -e "Running:\n$CMD"
$CMD >> "$OUTPUT_PATH"
sed -z -i 's/\n\n/\n/g' "$OUTPUT_PATH"
cat "$OUTPUT_PATH" | egrep -n -i ''
SUMMARY=$("$ROOT"/scripts/github/extrinsic-ordering-filter.sh "$OUTPUT_PATH")
echo -e $SUMMARY
echo -e $SUMMARY >> "$OUTPUT_PATH"

kill $PID

cat "$OUTPUT_PATH"
