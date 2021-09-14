#!/usr/bin/env bash
set -Eeuo pipefail

cargo run -- \
    -l debug \
    --csi-path /tmp/csi.sock \
    --csi-name metal-csi-debug \
    --metadata-db /tmp/metal-csi.db \
    --node-id dev-node \
    -c dev.yml $@
