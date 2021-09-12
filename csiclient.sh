#!/usr/bin/env bash
set -Eeuo pipefail

TEST_CMD="$1"
TEST_VOL_NAME="csiclient"
TEST_VOL_PVC_NAME="pvc-12341234123412341234"
TEST_ZFS_PARENT="hoard/csitest/"
TEST_BIND_PATH="/tmp/test-csitest-bind"
TEST_TARGET_PATH="/tmp/test-csitest"
TEST_STORAGECLASS_PARAMS="type=iscsi"
TEST_STORAGECLASS_SECRETS="$(cat secret.params)"

function csc() {
    docker run --rm -it -v "/tmp/csi.sock:/plugin/csi.sock" \
        -e "X_CSI_SECRETS=${TEST_STORAGECLASS_SECRETS}" \
        $(docker build -q -f csiclient.dockerfile .) \
        --endpoint unix:///plugin/csi.sock $@
}

case $TEST_CMD in
probe)
    csc identity probe
    ;;

info)
    csc identity plugin-info
    ;;

id)
    csc identity plugin-capabilities
    ;;

node-info)
    csc node get-info
    ;;

controller-info)
    csc controller get-capabilities
    ;;

controller-publish)
    csc controller publish "${TEST_ZFS_PARENT}${TEST_VOL_NAME}" --vol-context "${TEST_STORAGECLASS_PARAMS}"
    ;;

controller-unpublish)
    csc controller unpublish "${TEST_ZFS_PARENT}${TEST_VOL_NAME}"
    ;;

controller-create)
    csc controller create-volume "${TEST_VOL_PVC_NAME}" --params "${TEST_STORAGECLASS_PARAMS}"
    ;;

controller-delete)
    csc controller delete-volume "${TEST_VOL_NAME}"
    ;;

node-stage)
    csc node stage "${TEST_ZFS_PARENT}${TEST_VOL_NAME}" --staging-target-path "${TEST_BIND_PATH}"
    ;;

node-unstage)
    csc node unstage "${TEST_ZFS_PARENT}${TEST_VOL_NAME}" --staging-target-path "${TEST_BIND_PATH}"
    ;;

node-publish)
    csc node publish "${TEST_ZFS_PARENT}${TEST_VOL_NAME}" --staging-target-path "${TEST_BIND_PATH}" --target-path "${TEST_TARGET_PATH}"
    ;;

node-unpublish)
    csc node unpublish "${TEST_ZFS_PARENT}${TEST_VOL_NAME}" --target-path "${TEST_TARGET_PATH}"
    ;;

*)
    csc $@
    ;;
esac
