#!/usr/bin/bash

pushd . > /dev/null
set -e

LAUNCH_CMD="up -d"
LAUNCH_VALIDATORS="yes"
LAUNCH_COLLATOR="yes"

if [[ $1 == "down" ]]; then
  LAUNCH_CMD=$1
  [[ $2 == "clean" ]] && LAUNCH_CMD+=" -v"
  shift
fi

[[ $1 == "only-collator" ]] && LAUNCH_VALIDATORS="no"
[[ $1 == "only-validators" ]] && LAUNCH_COLLATOR="no"

SCRIPT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_ROOT"

[[ "$LAUNCH_VALIDATORS" == "yes" ]] && docker-compose -p dfpara -f rco-validators-compose.yml $LAUNCH_CMD
[[ "$LAUNCH_COLLATOR" == "yes" ]] && docker-compose -p dfpara -f subsocial-collator-compose.yml $LAUNCH_CMD

popd > /dev/null
exit 0
