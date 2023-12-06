#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

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
