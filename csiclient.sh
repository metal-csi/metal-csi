#!/usr/bin/env bash
set -Eeuo pipefail

docker run --rm -it -v "/tmp/csi.sock:/plugin/csi.sock" \
    $(docker build -q -f csiclient.dockerfile .) \
    --endpoint unix:///plugin/csi.sock $@
