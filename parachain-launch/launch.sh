#!/bin/bash

set -e

SCRIPT_ROOT=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
CONFIG_PATH=$SCRIPT_ROOT/config.yml

if [[ ! $(type parachain-launch 2> /dev/null) ]]; then
  echo "Parachain launch tool is not installed in your system."
  echo -e "Consider installing it with:\n\`sudo yarn global add parachain-launch\`"
  exit 1
fi

if [[ ! $(type docker-compose 2> /dev/null) ]]; then
  echo "Docker-compose is not installed in your system. Consider installing it and try again"
  echo "Details: https://docs.docker.com/compose/install/"
  exit 1
fi

parachain-launch generate --config="$CONFIG_PATH" --output "$SCRIPT_ROOT/output"
docker-compose -f "$SCRIPT_ROOT/output/docker-compose.yml" up -d --build
