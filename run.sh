#!/usr/bin/env bash
set -Eeuo pipefail

rm -f /tmp/csi.sock
cargo run -- \
    -l debug \
    -c dev.yml $@
