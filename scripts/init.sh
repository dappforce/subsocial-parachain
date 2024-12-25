#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ "$1" == "nosudo" ]; then
   apt-get update && \
   apt-get install -y build-essential clang curl libssl-dev protobuf-compiler
else
   sudo apt-get update && \
   sudo apt-get install -y build-essential clang curl libssl-dev protobuf-compiler
fi

type rustup >/dev/null 2>&1 || {
  echo >&2 "rustup is required, but it's not installed. Installing.";
  curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    . "$HOME/.cargo/env" && \
   rustup show;
}

CDIR=`dirname "$0"`
export RUSTC_VERSION=`cat $CDIR/../RUSTC_VERSION`

if [ -z $CI_PROJECT_NAME ] ; then
   rustup update $RUSTC_VERSION
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain $RUSTC_VERSION

# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
command -v wasm-gc || \
	cargo +$RUSTC_VERSION install --git https://github.com/alexcrichton/wasm-gc --force

rustup override set $RUSTC_VERSION
