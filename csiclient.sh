#!/usr/bin/env bash
set -Eeuo pipefail

docker run --rm -it -v "/tmp/csi.sock:/tmp/csi.sock" \
    $(docker build -q -f csiclient.dockerfile .) \
    --endpoint unix:///tmp/csi.sock $@
