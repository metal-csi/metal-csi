#!/usr/bin/env bash
set -Eeuo pipefail

cargo run -- \
    -l debug \
    --csi-path /tmp/csi.sock \
    --csi-name zed-csi-debug \
    --node-id dev-node \
    -c dev.yml $@
