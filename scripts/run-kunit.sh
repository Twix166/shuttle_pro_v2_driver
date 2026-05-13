#!/bin/sh
# Build and run the optional ShuttlePro parser KUnit module.

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)

make -C "$repo" BUILD_KUNIT=1

sudo rmmod hid_shuttlepro_kunit 2>/dev/null || true
sudo insmod "$repo/hid-shuttlepro-kunit.ko"
sudo rmmod hid_shuttlepro_kunit

echo "KUnit results are available in dmesg for suite: hid_shuttlepro"
