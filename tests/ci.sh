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

make -C "$repo" W=1

if [ -x "${CHECKPATCH:-}" ]; then
	"$CHECKPATCH" --strict --no-tree --file "$repo/hid-shuttlepro.c"
elif [ -x /lib/modules/"$(uname -r)"/build/scripts/checkpatch.pl ]; then
	/lib/modules/"$(uname -r)"/build/scripts/checkpatch.pl \
		--strict --no-tree --file "$repo/hid-shuttlepro.c"
else
	echo "checkpatch.pl not found; skipping checkpatch"
fi
