# Subsocial parachain node

The Subsocial parachain is our custom built Kusama blockchain, based on the Substrate framework.

## Build

For linux, FreeBSD, OpenBSD and macOS:

```sh
git clone https://github.com/dappforce/subsocial-parachain
cd subsocial-parachain/
sh scripts/init.sh
cargo build --release
```

## Run

Take into account that you need to build a binary as described in the previous step.

Simply run and join the network:

```shell
./target/release/subsocial-collator \
--name=your-node-name \
-- \
--execution=wasm \
--chain=kusama
```

Run as an archive node (store all blocks state):

```shell
./target/release/subsocial-collator \
--name=your-node-name \
--pruning=archive \
-- \
--execution=wasm \
--chain=kusama
```

## Using docker

Official docker hub image of the Subsocial parachain: https://hub.docker.com/r/dappforce/subsocial-parachain

Simply run and join the network with docker:

```shell
docker run -d -v node-data:/data dappforce/subsocial-parachain:latest subsocial-collator \
--name=your-node-name \
-- \
--execution=wasm \
--chain=kusama
```
