#!/bin/bash

set -eux

if git ls-files -m | read
then
    echo "Refusing to test with unstaged changes!"
    echo "Use \"git stash push --keep-index\""
    echo "Then use \"git stash pop\""
    exit 1
fi

# TODO: Ideally precommit hooks are incremental tests.
# Only dirty files need to be tested.
./tools/precommit/precommit-allow-dirty.sh

exit 0
