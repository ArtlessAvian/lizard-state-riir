#!/bin/bash

set -eux

# if ! git diff --name-only --cached | grep -Fe .rs -e .lock -e .toml
# then
#     exit 0
# fi

cargo +nightly -Z unstable-options -C rust-workspace fmt
cargo +nightly -Z unstable-options -C rust-workspace clippy --fix --allow-dirty -- -Dwarnings

if git ls-files -m | read
then
    echo "Formatting errors detected! Please manually review."
fi

cargo +nightly -Z unstable-options -C rust-workspace clippy -- -Dwarnings

if git ls-files -m | read
then
    exit 1
fi

exit 0
