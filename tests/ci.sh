#!/bin/sh

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)

for file in "$repo"/scripts/*.sh "$repo"/tests/*.sh; do
	sh -n "$file"
done

if command -v shellcheck >/dev/null 2>&1; then
	shellcheck "$repo"/scripts/*.sh "$repo"/tests/*.sh
fi

"$repo/tests/test_find_event.sh"
"$repo/tests/test_metadata.sh"

cc -std=c11 -Wall -Wextra -Werror -I"$repo" \
	"$repo/tests/test_decode.c" -o "$repo/tests/test_decode"
"$repo/tests/test_decode"

make -C "$repo" W=1
"$repo/tests/static-analysis.sh"

if grep -q '^CONFIG_KUNIT=[ym]' "${KDIR:-/lib/modules/$(uname -r)/build}/.config" 2>/dev/null; then
	make -C "$repo" BUILD_KUNIT=1 W=1
else
	echo "KUnit not enabled in kernel config; skipping KUnit module build"
fi

if [ -x "${CHECKPATCH:-}" ]; then
	"$CHECKPATCH" --strict --no-tree --file "$repo/hid-shuttlepro.c"
	"$CHECKPATCH" --strict --no-tree --file "$repo/shuttlepro-report.h"
	"$CHECKPATCH" --strict --no-tree --file "$repo/hid-shuttlepro-kunit.c"
elif [ -x /lib/modules/"$(uname -r)"/build/scripts/checkpatch.pl ]; then
	/lib/modules/"$(uname -r)"/build/scripts/checkpatch.pl \
		--strict --no-tree --file "$repo/hid-shuttlepro.c"
	/lib/modules/"$(uname -r)"/build/scripts/checkpatch.pl \
		--strict --no-tree --file "$repo/shuttlepro-report.h"
	/lib/modules/"$(uname -r)"/build/scripts/checkpatch.pl \
		--strict --no-tree --file "$repo/hid-shuttlepro-kunit.c"
else
	echo "checkpatch.pl not found; skipping checkpatch"
fi
