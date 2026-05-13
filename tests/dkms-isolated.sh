#!/bin/sh

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)
tmp=${TMPDIR:-/tmp}/hid-shuttlepro-dkms-test.$$
src="$tmp/src"
dkmstree="$tmp/dkms"
installtree="$tmp/modules"
kernelver=${KVER:-$(uname -r)}

cleanup()
{
	rm -rf "$tmp"
}
trap cleanup EXIT HUP INT TERM

mkdir -p "$src/hid-shuttlepro-0.1.0" "$dkmstree" "$installtree"
cp -a \
	"$repo/hid-shuttlepro.c" \
	"$repo/Makefile" \
	"$repo/dkms.conf" \
	"$repo/README.md" \
	"$repo/LICENSE" \
	"$repo/scripts" \
	"$src/hid-shuttlepro-0.1.0/"

dkms add -m hid-shuttlepro -v 0.1.0 \
	--sourcetree "$src" \
	--dkmstree "$dkmstree" \
	--installtree "$installtree"

dkms build -m hid-shuttlepro -v 0.1.0 -k "$kernelver" \
	--dkmstree "$dkmstree" \
	--installtree "$installtree" \
	--kernelsourcedir "${KDIR:-/lib/modules/$kernelver/build}"
