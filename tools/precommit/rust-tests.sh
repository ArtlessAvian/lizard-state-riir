#!/bin/bash

set -eux

# if ! git diff --name-only --cached | grep -Fe .rs -e .lock -e .toml
# then
#     exit 0
# fi

cargo +nightly -Z unstable-options -C rust-workspace nextest run --no-fail-fast
cargo +nightly -Z unstable-options -C rust-workspace test --doc

exit 0
