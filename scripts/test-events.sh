#!/bin/sh
# Run evtest against the current ShuttlePro v2 evdev node.

set -eu

dir=$(unset CDPATH; cd -- "$(dirname -- "$0")" && pwd)
event=$("$dir/find-event.sh")

if [ -z "$event" ]; then
	echo "Contour ShuttlePro v2 event node not found" >&2
	exit 1
fi

exec evtest "$event"
