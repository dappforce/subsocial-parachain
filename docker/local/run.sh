#!/usr/bin/bash

pushd . > /dev/null

set -e

LAUNCH_CMD="up -d"
[[ $1 == "down" ]] && LAUNCH_CMD="down -v"

SCRIPT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_ROOT"

docker-compose -p dfpara -f rco-validator-compose.yml $LAUNCH_CMD
docker-compose -p dfpara -f subsocial-collator-compose.yml $LAUNCH_CMD

popd > /dev/null
exit 0
