#!/usr/bin/env bash

set -e

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )/.."

docker pull parity/rococo:rococo-v1
docker run --rm -d --name rococo-export parity/rococo:rococo-v1 --chain rococo-local > /dev/null

docker exec -it -u root rococo-export sh -c "polkadot build-spec --chain rococo-local > /rococo-local"
docker exec -it -u root rococo-export sh -c "polkadot build-spec --chain rococo > /rococo"
docker exec -it -u root rococo-export sh -c "polkadot build-spec --chain /rococo-local --raw > /rococo-local.json"
docker exec -it -u root rococo-export sh -c "polkadot build-spec --chain /rococo --raw > /rococo.json"

docker cp rococo-export:/rococo-local.json $PROJECT_DIR/res/rococo-local.json
docker cp rococo-export:/rococo.json $PROJECT_DIR/res/rococo.json

docker container stop rococo-export > /dev/null
