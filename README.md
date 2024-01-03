# Subsocial Parachain Node

The Subsocial parachain is our custom-built Polkadot blockchain, based on the Substrate framework.

## Build

For Linux, FreeBSD, OpenBSD, and macOS:

```sh
git clone https://github.com/dappforce/subsocial-parachain
cd subsocial-parachain/
sh scripts/init.sh
cargo build --release
```

## Run

Please note that you need to build a binary as described in the previous step.

Simply run and join the network:

```shell
./target/release/subsocial-collator \
--name=your-node-name \
-- \
--execution=wasm \
--chain=polkadot
```

Run as an archive node (store all block states):

```shell
./target/release/subsocial-collator \
--name=your-node-name \
--pruning=archive \
-- \
--execution=wasm \
--chain=polkadot
```

### Using Docker

Find the complete example in the [docker/docker-compose.yml](docker/docker-compose.yml) file.

Official Docker Hub image of the Subsocial parachain: [Dappforce Docker Hub](https://hub.docker.com/r/dappforce/subsocial-parachain)

Simply run and join the network with Docker:

```shell
docker run -d -v node-data:/data dappforce/subsocial-parachain:latest subsocial-collator \
--name=your-node-name \
-- \
--execution=wasm \
--chain=kusama
```

**Note:**

- You may need to build the Docker image if the one [in the registry](https://hub.docker.com/r/dappforce/subsocial-parachain) is outdated.
- To build the latest Docker image, compatible with parachain-launch, run exactly:

  ```shell
  docker build . -f docker/Dockerfile -t dappforce/subsocial-parachain:rococo
  ```
