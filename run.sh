#!/usr/bin/env bash
set -Eeuo pipefail

cargo run -- \
    -l debug \
    --csi-path /tmp/csi.sock \
    -c dev.yml $@
