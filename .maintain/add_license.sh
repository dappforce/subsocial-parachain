#!/bin/bash

set -e

# Find all .go files and process them
find . -type f -name '*.rs' -print0 | while IFS= read -r -d $'\0' file; do
    addlicense $1 -f .maintain/license-header "$file"
done
