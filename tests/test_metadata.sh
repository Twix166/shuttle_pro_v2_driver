#!/bin/sh

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)

grep -q '^// SPDX-License-Identifier: GPL-2.0-only$' \
	"$repo/hid-shuttlepro.c"
grep -q '^MODULE_LICENSE("GPL");$' "$repo/hid-shuttlepro.c"
grep -q '^PACKAGE_NAME="hid-shuttlepro"$' "$repo/dkms.conf"
grep -q '^PACKAGE_VERSION="0.1.0"$' "$repo/dkms.conf"
grep -q '^AUTOINSTALL="yes"$' "$repo/dkms.conf"
grep -q 'TAG+="uaccess"' "$repo/99-hid-shuttlepro.rules"
grep -q 'Use this software entirely at your own risk' "$repo/README.md"
