# Subsocial parachain node

The Subsocial parachain is our custom built Kusama blockchain, based on the Substrate framework.

## Build

For Linux, FreeBSD, OpenBSD, and macOS:

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

### Using docker

Official Docker Hub image of the Subsocial parachain: 
https://hub.docker.com/r/dappforce/subsocial-parachain

Simply run and join the network with docker:

```shell
docker run -d -v node-data:/data dappforce/subsocial-parachain:latest subsocial-collator \
--name=your-node-name \
-- \
--execution=wasm \
--chain=kusama
```

### Using parachain-launch

- Install [parachain-launch](https://github.com/open-web3-stack/parachain-launch)
- Install [Docker-compose](https://docs.docker.com/compose/install/)
- Configure and launch with a single command: `./parachain-launch/launch.sh`

**Note:**

- You may need to build docker image if the one [in registry](https://hub.docker.com/r/dappforce/subsocial-parachain) is outdated.
- To build latest docker image, compatible with parachain-launch, run exactly:
    ```shell
    docker build . -f docker/Dockerfile -t dappforce/subsocial-parachain:rococo
    ```
