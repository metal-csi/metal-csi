#!/usr/bin/env bash
set -Eeuo pipefail

docker build -t zed-csi-builder -f build/Dockerfile .

function do_run() {
    docker run --rm -t -u $UID:$UID \
        -v "${PWD}/driver:/app/driver" \
        -v "${PWD}/target:/app/target" \
        -v "${PWD}/cache:/toolchain/registry" zed-csi-builder \
        $@
}

function build_target() {
    target=$1
    mkdir -p driver target cache
    do_run cargo build --release --target $target
    do_run rm -f "target/${target}/release/zed-csi.lz"
    do_run lzip -9 -k "target/${target}/release/zed-csi"
}

build_target x86_64-unknown-linux-gnu
build_target aarch64-unknown-linux-gnu
build_target armv7-unknown-linux-gnueabihf
