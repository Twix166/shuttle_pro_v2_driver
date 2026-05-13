#!/bin/sh

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)
kdir=${KDIR:-/lib/modules/$(uname -r)/build}

if command -v sparse >/dev/null 2>&1; then
	make -C "$kdir" M="$repo" C=1 CHECK=sparse modules
else
	echo "sparse not found; skipping sparse"
fi

if command -v spatch >/dev/null 2>&1 &&
	[ -x "$kdir/scripts/coccicheck" ]; then
	make -C "$kdir" M="$repo" C=1 CHECK="$kdir/scripts/coccicheck" \
		MODE=report modules
else
	echo "Coccinelle not found; skipping coccicheck"
fi
