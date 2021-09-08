#!/usr/bin/env bash
set -Eeuo pipefail

rm -f /tmp/csi.sock
cargo run -- \
    -l debug \
    --csi-path /tmp/csi.sock \
    -c dev.yml $@
