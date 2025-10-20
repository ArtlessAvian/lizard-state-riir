#!/bin/bash

set -eux

cargo +nightly -Z unstable-options -C rust-workspace build
godot -v --headless --path godot-project/ --import

exit 0
