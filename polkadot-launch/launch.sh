#!/bin/bash

POLKADOT_LAUNCH_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
POLKADOT_BIN_PATH=$POLKADOT_LAUNCH_DIR/bin/polkadot
CONFIG_PATH=$POLKADOT_LAUNCH_DIR/config.json

if [[ ! $(type polkadot-launch 2> /dev/null) ]]; then
  echo "Polkadot launch is not installed in your system."
  echo -e "Consider installing it with:\n\`sudo yarn global add polkadot-launch\`"
  exit 1
fi

if [[ ! -f $POLKADOT_BIN_PATH ]]; then
  echo "Downloading polkadot binary..."
  "$POLKADOT_LAUNCH_DIR"/get-polkadot-bin.sh "$POLKADOT_BIN_PATH" &> /dev/null
fi

(cd "$POLKADOT_LAUNCH_DIR" && polkadot-launch "$CONFIG_PATH")
