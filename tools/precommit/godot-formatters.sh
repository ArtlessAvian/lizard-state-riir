#!/bin/bash

set -eux

git ls-files | grep .gd$ | xargs --no-run-if-empty gdformat

if git ls-files -m | read
then
    echo "Formatting errors detected! Please manually review."
    exit 1
fi

exit 0
