#!/usr/bin/env bash
set -Eeuo pipefail

function do_run() {
    docker run --rm -t -u $UID:$UID \
        -v "${PWD}/driver:/app/driver" \
        -v "${PWD}/target:/app/target" \
        -v "${PWD}/out:/app/out" \
        -v "${PWD}/cache:/toolchain/registry" metal-csi-builder \
        $@
}

function build_target() {
    target=$1
    tag=$2
    mkdir -p driver target cache out
    do_run cargo build --release --target $target
    do_run rm -f "target/${target}/release/metal-csi.lz"
    do_run lzip -9 -k "target/${target}/release/metal-csi"
    do_run cp "target/${target}/release/metal-csi.lz" "out/metal-csi.$tag.lz"
}

docker build -t metal-csi-builder -f build/Dockerfile .
build_target x86_64-unknown-linux-gnu amd64
build_target aarch64-unknown-linux-gnu arm64
build_target armv7-unknown-linux-gnueabihf armv7
