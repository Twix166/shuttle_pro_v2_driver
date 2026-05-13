#!/bin/sh
# Print the evdev node for the Contour ShuttlePro v2.

set -eu

awk '
	/^N: Name="Contour ShuttlePro v2"$/ { found = 1; next }
	found && /^H: Handlers=/ {
		for (i = 1; i <= NF; i++) {
			sub(/^Handlers=/, "", $i)
			if ($i ~ /^event[0-9]+$/) {
				print "/dev/input/" $i
				exit 0
			}
		}
	}
	found && /^$/ { found = 0 }
' /proc/bus/input/devices
