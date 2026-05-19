#!/bin/bash
set -e
cargo build --release "$@"
sh "bindings.sh"