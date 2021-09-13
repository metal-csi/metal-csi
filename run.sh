#!/usr/bin/env bash
set -Eeuo pipefail

cargo run -- \
    -l debug \
    --csi-path /tmp/csi.sock \
    --csi-name zed-csi-debug \
    --metadata-db /tmp/zedcsi.db \
    --node-id dev-node \
    -c dev.yml $@
