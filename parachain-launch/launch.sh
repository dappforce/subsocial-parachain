#!/bin/bash

set -e

SCRIPT_ROOT=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
CONFIG_PATH=$SCRIPT_ROOT/config.yml
OUTPUT_DIR="$SCRIPT_ROOT/output"

function stop_containers {
  local clean_volumes=${1:-false}

  [[ ! -f "$OUTPUT_DIR/docker-compose.yml" ]] && exit 1

  if [ "$clean_volumes" = true ]; then
    docker-compose -f "$SCRIPT_ROOT/output/docker-compose.yml" down -v
  else
    docker-compose -f "$SCRIPT_ROOT/output/docker-compose.yml" down
  fi
}

case "$1" in
  "") ;;
  --stop)
    stop_containers
    exit 0
    ;;
  --clean)
    stop_containers true
    exit 0
    ;;
  --help | *)
    echo "Usage: ./launch.sh [--stop | --clean]"
    echo "  --stop: Clean up the containers keeping volumes"
    echo "  --clean: Clean up the containers and volumes"
    exit 0
esac

if [[ ! $(type parachain-launch 2> /dev/null) ]]; then
  echo "Parachain launch tool is not installed in your system."
  echo -e "Consider installing it with:\n\`sudo yarn global add @open-web3/parachain-launch\`"
  exit 1
fi

if [[ ! $(type docker-compose 2> /dev/null) ]]; then
  echo "Docker-compose is not installed in your system. Consider installing it and try again"
  echo "Details: https://docs.docker.com/compose/install/"
  exit 1
fi

if [[ -f "$OUTPUT_DIR/docker-compose.yml" ]]; then
  stop_containers
  rm -rf "$OUTPUT_DIR"
fi

parachain-launch generate --config="$CONFIG_PATH" --output "$OUTPUT_DIR"
docker-compose -f "$SCRIPT_ROOT/output/docker-compose.yml" up -d --build
