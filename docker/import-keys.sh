#!/usr/bin/bash

set -e

insert_key(){
  [[ -z $1 || -z $2 ]] && exit 1
  printf "Key \e[33m%s\e[00m insert result: \e[37m" "${2%.json*}"

  until \
    curl http://localhost:"$1" -H "Content-Type:application/json;charset=utf-8" -d "@$SCRIPT_ROOT/keys/$2" 2> /dev/null
  do
    sleep 2
  done
  printf "\e[00m"
}

SCRIPT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

bash -c "UNSAFE_RPC_METHODS=\"--rpc-methods=Unsafe\" $SCRIPT_ROOT/run.sh only-validators" 2> /dev/null

insert_key 9733 "aura-1.json"
insert_key 9733 "gran-1.json"

insert_key 9734 "aura-2.json"
insert_key 9734 "gran-2.json"

bash -c "$SCRIPT_ROOT/run.sh only-validators" 2> /dev/null
