#!/bin/sh

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)

actual=$(INPUT_DEVICES_PATH="$repo/tests/fixtures/input-devices-shuttle.txt" \
	"$repo/scripts/find-event.sh")

if [ "$actual" != "/dev/input/event26" ]; then
	echo "expected /dev/input/event26, got: $actual" >&2
	exit 1
fi

actual=$(INPUT_DEVICES_PATH="$repo/tests/fixtures/input-devices-no-shuttle.txt" \
	"$repo/scripts/find-event.sh")

if [ -n "$actual" ]; then
	echo "expected no output when ShuttlePro is absent, got: $actual" >&2
	exit 1
fi
