#!/bin/bash

[[ -z $1 ]] && exit 1

mkdir -p "$(dirname "$1")"
wget -O "$1" https://github.com/paritytech/polkadot/releases/download/v0.9.16/polkadot

chmod +x "$1"
