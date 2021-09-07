#!/usr/bin/env bash
set -Eeuo pipefail

# Enable qemu exec
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes > /dev/null

# Use cross-builder profile
docker buildx use cross-builder

# Build the image
docker buildx build --platform linux/arm64,linux/amd64,linux/arm/v7 \
    --build-arg CSIVERSION="0.0.1-alpha3" \
    -f dist.dockerfile -t "zed-csi-local" .
